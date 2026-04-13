//! Record update for GTFS feeds.
//!
//! Provides a two-phase API: [`validate_update`] validates field assignments and
//! builds an [`UpdatePlan`], then [`apply_update`] mutates the feed.

use std::collections::HashSet;

use thiserror::Error;

use crate::crud::common::{
    CascadeEntry, CrudError, FeedIndex, FieldAssignment, find_matching_indices, get_pk_value,
    make_entity_ref, parse_assignments, primary_key_fields, to_field_map, validate_foreign_keys,
};
use crate::crud::query::Filterable;
use crate::crud::query::{Query, QueryError};
use crate::crud::read::GtfsTarget;
use crate::crud::setters::FieldSetter;
use crate::integrity::{EntityRef, IntegrityIndex};
use crate::models::GtfsFeed;
use crate::parser::feed_source::GtfsFiles;

/// Errors that can occur during record update.
#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Invalid assignment \"{0}\": expected field=value")]
    InvalidAssignment(String),

    #[error("Duplicate assignment for field \"{0}\"")]
    DuplicateAssignment(String),

    #[error("Unknown field \"{field}\" (valid fields: {valid})")]
    UnknownField { field: String, valid: String },

    #[error("Invalid value for {field}: expected {expected}, got \"{value}\"")]
    InvalidFieldValue {
        field: String,
        value: String,
        expected: String,
    },

    #[error("{field} '{value}' already exists in {file}")]
    DuplicatePrimaryKey {
        field: String,
        value: String,
        file: String,
    },

    #[error("{field} '{value}' does not exist in {referenced_file}")]
    ForeignKeyViolation {
        field: String,
        value: String,
        referenced_file: String,
    },

    #[error("No field assignments provided")]
    EmptyAssignments,

    #[error(
        "Cannot update {field} '{value}': referenced by {count} {dependent_file} records. \
         Delete referencing records first or use cascade."
    )]
    PrimaryKeyReferenced {
        field: String,
        value: String,
        dependent_file: String,
        count: usize,
    },

    #[error("{0}")]
    QueryError(#[from] QueryError),
}

impl From<CrudError> for UpdateError {
    fn from(e: CrudError) -> Self {
        match e {
            CrudError::InvalidAssignment(s) => Self::InvalidAssignment(s),
            CrudError::DuplicateAssignment(s) => Self::DuplicateAssignment(s),
            CrudError::UnknownField { field, valid } => Self::UnknownField { field, valid },
            CrudError::InvalidFieldValue {
                field,
                value,
                expected,
            } => Self::InvalidFieldValue {
                field,
                value,
                expected,
            },
            CrudError::DuplicatePrimaryKey { field, value, file } => {
                Self::DuplicatePrimaryKey { field, value, file }
            }
            CrudError::ForeignKeyViolation {
                field,
                value,
                referenced_file,
            } => Self::ForeignKeyViolation {
                field,
                value,
                referenced_file,
            },
            CrudError::EmptyAssignments => Self::EmptyAssignments,
        }
    }
}

/// The validated update plan ready to be applied to the feed.
#[derive(Debug)]
pub struct UpdatePlan {
    pub target: GtfsTarget,
    pub file_name: &'static str,
    pub matched_count: usize,
    pub assignments: Vec<FieldAssignment>,
    pub(crate) matched_indices: Vec<usize>,
    /// When `--cascade` propagates a PK change to dependent files.
    pub cascade: Option<CascadePlan>,
}

/// Cascade plan: propagates a PK rename to all FK references in dependent files.
#[derive(Debug)]
pub struct CascadePlan {
    pub pk_field: String,
    pub old_value: String,
    pub new_value: String,
    pub entries: Vec<CascadeEntry>,
}

/// Result of applying an update (possibly with cascade).
#[derive(Debug)]
pub struct UpdateResult {
    pub count: usize,
    pub modified_targets: Vec<GtfsTarget>,
}

/// Returns `true` if any of the `--set` assignments targets a primary key field.
///
/// Call this **before** loading the feed to decide whether dependent files
/// need to be loaded (see [`required_files`]).
#[must_use]
pub fn has_pk_assignments(target: GtfsTarget, raw_set: &[String]) -> bool {
    let pk = primary_key_fields(target);
    if pk.is_empty() {
        return false;
    }
    raw_set
        .iter()
        .filter_map(|s| s.split_once('=').map(|(f, _)| f))
        .any(|field| pk.contains(&field))
}

/// Returns the GTFS files that must be loaded to validate an update on `target`.
///
/// When `include_dependents` is `true`, the result also includes files that
/// reference this target's primary keys — needed only when `--set` modifies a
/// PK field (see [`has_pk_assignments`]).  When `false`, only the target file
/// and its FK dependencies are returned (same set as `create::required_files`).
#[must_use]
pub fn required_files(target: GtfsTarget, include_dependents: bool) -> Vec<GtfsFiles> {
    use GtfsFiles as F;

    let fk_deps = crate::crud::create::required_files(target);

    if !include_dependents {
        return fk_deps;
    }

    // Dependent files (files that reference this target's PKs)
    let dependents: Vec<GtfsFiles> = match target {
        GtfsTarget::Agency => vec![F::Routes, F::FareAttributes, F::Attributions],
        GtfsTarget::Stops => vec![F::Stops, F::StopTimes, F::Transfers, F::Pathways],
        GtfsTarget::Routes => vec![F::Trips, F::FareRules, F::Transfers, F::Attributions],
        GtfsTarget::Trips => vec![F::StopTimes, F::Frequencies, F::Transfers, F::Attributions],
        GtfsTarget::Calendar | GtfsTarget::CalendarDates => {
            vec![F::CalendarDates, F::Trips]
        }
        GtfsTarget::Shapes => vec![F::Trips],
        GtfsTarget::Levels => vec![F::Stops],
        GtfsTarget::FareAttributes => vec![F::FareRules],
        GtfsTarget::StopTimes
        | GtfsTarget::Frequencies
        | GtfsTarget::Transfers
        | GtfsTarget::Pathways
        | GtfsTarget::FeedInfo
        | GtfsTarget::FareRules
        | GtfsTarget::Translations
        | GtfsTarget::Attributions => vec![],
    };

    let mut set = HashSet::new();
    let mut result = Vec::new();
    for f in fk_deps.into_iter().chain(dependents) {
        if set.insert(f) {
            result.push(f);
        }
    }
    result
}

fn build_cascade_from_index(integrity: &IntegrityIndex, entity: &EntityRef) -> Vec<CascadeEntry> {
    let dependents = integrity.find_dependents(entity);
    if dependents.is_empty() {
        return Vec::new();
    }

    let mut counts: std::collections::HashMap<(GtfsTarget, &'static str), usize> =
        std::collections::HashMap::new();
    for (dep_entity, relation) in dependents {
        *counts
            .entry((dep_entity.target(), relation.fk_field_name()))
            .or_default() += 1;
    }

    let mut by_target: std::collections::HashMap<GtfsTarget, (Vec<&'static str>, usize)> =
        std::collections::HashMap::new();
    for ((dep_target, fk_field), count) in counts {
        let entry = by_target.entry(dep_target).or_default();
        if !entry.0.contains(&fk_field) {
            entry.0.push(fk_field);
        }
        entry.1 += count;
    }

    by_target
        .into_iter()
        .map(|(dependent, (fk_fields, count))| CascadeEntry {
            dependent,
            fk_fields,
            count,
        })
        .collect()
}

/// Validates field assignments against a query and builds an [`UpdatePlan`]
/// without mutating the feed.
///
/// Performs all checks: field name validity, type parsing, query field
/// validation, PK referential integrity, FK integrity, and type validation
/// on a cloned record.
///
/// # Errors
///
/// Returns [`UpdateError`] on any validation failure.
pub fn validate_update(
    feed: &GtfsFeed,
    target: GtfsTarget,
    query: &Query,
    raw_assignments: &[String],
    cascade: bool,
) -> Result<UpdatePlan, UpdateError> {
    // 1. Parse and validate assignments
    let assignments = parse_assignments(raw_assignments)?;
    let fields = to_field_map(&assignments, target)?;

    // 2. Build index for PK/FK lookups
    let index = FeedIndex::build(feed, target);

    // 3. Validate query fields
    for_each_target_type!(target, |T| query.validate_fields::<T>()?);

    // 4. Find matching record indices
    let matched_indices = dispatch_slice!(target, feed, |records| find_matching_indices(
        records, query
    ));

    let matched_count = matched_indices.len();

    // 5. Early return if no matches
    if matched_indices.is_empty() {
        return Ok(UpdatePlan {
            target,
            file_name: target.file_name(),
            matched_count: 0,
            assignments,
            matched_indices,
            cascade: None,
        });
    }

    // 6. Check PK fields — only for targets with simple PKs
    let pk_fields = primary_key_fields(target);
    let pk_assignments: Vec<_> = assignments
        .iter()
        .filter(|a| pk_fields.contains(&a.field.as_str()))
        .collect();

    let mut cascade_plan = None;

    if !pk_assignments.is_empty() {
        cascade_plan = check_pk_constraints(
            feed,
            target,
            &matched_indices,
            &pk_assignments,
            &index,
            cascade,
        )?;
    }

    // 7. Validate FK fields
    validate_foreign_keys(&index, target, &fields)?;

    // 8. Type validation — apply assignments to a cloned first-matched record
    validate_types(feed, target, &matched_indices, &assignments)?;

    Ok(UpdatePlan {
        target,
        file_name: target.file_name(),
        matched_count,
        assignments,
        matched_indices,
        cascade: cascade_plan,
    })
}

fn check_pk_constraints(
    feed: &GtfsFeed,
    target: GtfsTarget,
    matched_indices: &[usize],
    pk_assignments: &[&FieldAssignment],
    index: &FeedIndex,
    cascade: bool,
) -> Result<Option<CascadePlan>, UpdateError> {
    let matched_set: HashSet<usize> = matched_indices.iter().copied().collect();

    // Check new PK doesn't already exist (excluding matched records)
    for pk_a in pk_assignments {
        check_new_pk_unique(target, &pk_a.field, &pk_a.value, index, &matched_set)?;
    }

    // Build integrity index to discover dependents
    let integrity = IntegrityIndex::build_from_feed(feed);

    let pk_field = pk_assignments.first().map_or("id", |a| a.field.as_str());
    let new_value = pk_assignments.first().map_or("", |a| a.value.as_str());

    for &idx in matched_indices {
        let Some(old_pk) = get_pk_value(feed, target, idx) else {
            continue;
        };
        let Some(entity) = make_entity_ref(target, &old_pk) else {
            continue;
        };

        let entries = build_cascade_from_index(&integrity, &entity);
        if entries.is_empty() {
            continue;
        }

        if cascade {
            return Ok(Some(CascadePlan {
                pk_field: pk_field.to_string(),
                old_value: old_pk,
                new_value: new_value.to_string(),
                entries,
            }));
        }

        // Not cascading — report the first dependent as an error
        let first = &entries[0];
        return Err(UpdateError::PrimaryKeyReferenced {
            field: pk_field.to_string(),
            value: old_pk,
            dependent_file: first.dependent.file_name().to_string(),
            count: first.count,
        });
    }

    Ok(None)
}

fn check_new_pk_unique(
    target: GtfsTarget,
    field: &str,
    value: &str,
    index: &FeedIndex,
    _matched_set: &HashSet<usize>,
) -> Result<(), UpdateError> {
    // For simple PKs, check the index set. If the new value already exists
    // in the set it is a conflict (the set includes all records, but matched
    // records will have their PK changed so a collision means a different
    // record already owns this value).
    let (set, file) = match target {
        GtfsTarget::Agency => (&index.agency_ids, "agency.txt"),
        GtfsTarget::Stops => (&index.stop_ids, "stops.txt"),
        GtfsTarget::Routes => (&index.route_ids, "routes.txt"),
        GtfsTarget::Trips => (&index.trip_ids, "trips.txt"),
        GtfsTarget::Calendar => (&index.service_ids, "calendar.txt"),
        GtfsTarget::Pathways => (&index.pathway_ids, "pathways.txt"),
        GtfsTarget::Levels => (&index.level_ids, "levels.txt"),
        GtfsTarget::FareAttributes => (&index.fare_ids, "fare_attributes.txt"),
        _ => return Ok(()),
    };

    if set.contains(value) {
        return Err(UpdateError::DuplicatePrimaryKey {
            field: field.to_string(),
            value: value.to_string(),
            file: file.to_string(),
        });
    }

    Ok(())
}

fn validate_types(
    feed: &GtfsFeed,
    target: GtfsTarget,
    matched_indices: &[usize],
    assignments: &[FieldAssignment],
) -> Result<(), UpdateError> {
    dispatch_slice!(target, feed, |records| {
        if records.is_empty() {
            return Ok(());
        }
        let mut clone = records[matched_indices[0]].clone();
        for a in assignments {
            clone.set_field(&a.field, &a.value)?;
        }
        Ok(())
    })
}

fn write_assignments<T>(records: &mut [T], plan: &UpdatePlan) -> Result<(), UpdateError>
where
    T: FieldSetter,
{
    for &idx in &plan.matched_indices {
        for a in &plan.assignments {
            records[idx].set_field(&a.field, &a.value)?;
        }
    }
    Ok(())
}

fn rewrite_fk<T>(
    records: &mut [T],
    fk_field: &str,
    old_value: &str,
    new_value: &str,
) -> Result<(), UpdateError>
where
    T: Filterable + FieldSetter,
{
    for record in records.iter_mut() {
        if record.field_value(fk_field).as_deref() == Some(old_value) {
            record.set_field(fk_field, new_value)?;
        }
    }
    Ok(())
}

/// Applies a validated [`UpdatePlan`] by mutating the matched records in-place,
/// including cascade updates to dependent files if present.
///
/// # Errors
///
/// Returns [`UpdateError`] if a setter call fails. This should not happen in
/// practice since the plan is pre-validated by [`validate_update`], but
/// errors are surfaced rather than panicked on so the caller can decide.
pub fn apply_update(feed: &mut GtfsFeed, plan: &UpdatePlan) -> Result<UpdateResult, UpdateError> {
    dispatch_slice_mut!(plan.target, feed, |records| write_assignments(
        records, plan
    ))?;

    let mut modified_targets = vec![plan.target];

    if let Some(ref cascade) = plan.cascade {
        apply_cascade(feed, cascade)?;
        for entry in &cascade.entries {
            if !modified_targets.contains(&entry.dependent) {
                modified_targets.push(entry.dependent);
            }
        }
    }

    Ok(UpdateResult {
        count: plan.matched_count,
        modified_targets,
    })
}

fn apply_cascade(feed: &mut GtfsFeed, cascade: &CascadePlan) -> Result<(), UpdateError> {
    let old = &cascade.old_value;
    let new = &cascade.new_value;

    for entry in &cascade.entries {
        for &fk_field in &entry.fk_fields {
            apply_cascade_to_target(feed, entry.dependent, fk_field, old, new)?;
        }
    }
    Ok(())
}

fn apply_cascade_to_target(
    feed: &mut GtfsFeed,
    target: GtfsTarget,
    fk_field: &str,
    old_value: &str,
    new_value: &str,
) -> Result<(), UpdateError> {
    dispatch_slice_mut!(target, feed, |records| rewrite_fk(
        records, fk_field, old_value, new_value
    ))
}
