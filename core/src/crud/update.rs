//! Record update for GTFS feeds.
//!
//! Provides a two-phase API: [`validate_update`] validates field assignments and
//! builds an [`UpdatePlan`], then [`apply_update`] mutates the feed.

use std::collections::HashSet;

use thiserror::Error;

use crate::crud::common::{
    CrudError, FeedIndex, FieldAssignment, parse_assignments, primary_key_fields, to_field_map,
    validate_foreign_keys,
};
use crate::crud::query::Filterable;
use crate::crud::query::{Query, QueryError};
use crate::crud::read::GtfsTarget;
use crate::crud::setters::{
    set_agency_field, set_attribution_field, set_calendar_date_field, set_calendar_field,
    set_fare_attribute_field, set_fare_rule_field, set_feed_info_field, set_frequency_field,
    set_level_field, set_pathway_field, set_route_field, set_shape_field, set_stop_field,
    set_stop_time_field, set_transfer_field, set_translation_field, set_trip_field,
};
use crate::integrity::{EntityRef, IntegrityIndex};
use crate::models::{
    Agency, Attribution, Calendar, CalendarDate, FareAttribute, FareRule, FeedInfo, Frequency,
    GtfsFeed, Level, Pathway, Route, Shape, Stop, StopTime, Transfer, Translation, Trip,
};
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

/// A group of FK references in a single dependent file that will be updated.
#[derive(Debug)]
pub struct CascadeEntry {
    pub dependent: GtfsTarget,
    pub fk_fields: Vec<&'static str>,
    pub count: usize,
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

fn make_entity_ref(target: GtfsTarget, pk_value: &str) -> Option<EntityRef> {
    match target {
        GtfsTarget::Agency => Some(EntityRef::Agency(pk_value.into())),
        GtfsTarget::Stops => Some(EntityRef::Stop(pk_value.into())),
        GtfsTarget::Routes => Some(EntityRef::Route(pk_value.into())),
        GtfsTarget::Trips => Some(EntityRef::Trip(pk_value.into())),
        GtfsTarget::Calendar | GtfsTarget::CalendarDates => {
            Some(EntityRef::Service(pk_value.into()))
        }
        GtfsTarget::Pathways => Some(EntityRef::Pathway(pk_value.into())),
        GtfsTarget::Levels => Some(EntityRef::Level(pk_value.into())),
        GtfsTarget::FareAttributes => Some(EntityRef::Fare(pk_value.into())),
        // Composite or no PK — not supported for cascade
        _ => None,
    }
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

fn find_matching_indices<T: Filterable>(records: &[T], query: &Query) -> Vec<usize> {
    records
        .iter()
        .enumerate()
        .filter(|(_, r)| query.matches(*r))
        .map(|(i, _)| i)
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
#[allow(clippy::too_many_lines)]
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
    match target {
        GtfsTarget::Agency => query.validate_fields::<Agency>()?,
        GtfsTarget::Stops => query.validate_fields::<Stop>()?,
        GtfsTarget::Routes => query.validate_fields::<Route>()?,
        GtfsTarget::Trips => query.validate_fields::<Trip>()?,
        GtfsTarget::StopTimes => query.validate_fields::<StopTime>()?,
        GtfsTarget::Calendar => query.validate_fields::<Calendar>()?,
        GtfsTarget::CalendarDates => query.validate_fields::<CalendarDate>()?,
        GtfsTarget::Shapes => query.validate_fields::<Shape>()?,
        GtfsTarget::Frequencies => query.validate_fields::<Frequency>()?,
        GtfsTarget::Transfers => query.validate_fields::<Transfer>()?,
        GtfsTarget::Pathways => query.validate_fields::<Pathway>()?,
        GtfsTarget::Levels => query.validate_fields::<Level>()?,
        GtfsTarget::FeedInfo => query.validate_fields::<FeedInfo>()?,
        GtfsTarget::FareAttributes => query.validate_fields::<FareAttribute>()?,
        GtfsTarget::FareRules => query.validate_fields::<FareRule>()?,
        GtfsTarget::Translations => query.validate_fields::<Translation>()?,
        GtfsTarget::Attributions => query.validate_fields::<Attribution>()?,
    }

    // 4. Find matching record indices
    let matched_indices = match target {
        GtfsTarget::Agency => find_matching_indices(&feed.agencies, query),
        GtfsTarget::Stops => find_matching_indices(&feed.stops, query),
        GtfsTarget::Routes => find_matching_indices(&feed.routes, query),
        GtfsTarget::Trips => find_matching_indices(&feed.trips, query),
        GtfsTarget::StopTimes => find_matching_indices(&feed.stop_times, query),
        GtfsTarget::Calendar => find_matching_indices(&feed.calendars, query),
        GtfsTarget::CalendarDates => find_matching_indices(&feed.calendar_dates, query),
        GtfsTarget::Shapes => find_matching_indices(&feed.shapes, query),
        GtfsTarget::Frequencies => find_matching_indices(&feed.frequencies, query),
        GtfsTarget::Transfers => find_matching_indices(&feed.transfers, query),
        GtfsTarget::Pathways => find_matching_indices(&feed.pathways, query),
        GtfsTarget::Levels => find_matching_indices(&feed.levels, query),
        GtfsTarget::FeedInfo => find_matching_indices(feed.feed_info.as_slice(), query),
        GtfsTarget::FareAttributes => find_matching_indices(&feed.fare_attributes, query),
        GtfsTarget::FareRules => find_matching_indices(&feed.fare_rules, query),
        GtfsTarget::Translations => find_matching_indices(&feed.translations, query),
        GtfsTarget::Attributions => find_matching_indices(&feed.attributions, query),
    };

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
        let Some(old_pk) = get_old_pk_value(feed, target, idx) else {
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

fn get_old_pk_value(feed: &GtfsFeed, target: GtfsTarget, idx: usize) -> Option<String> {
    match target {
        GtfsTarget::Agency => feed.agencies[idx]
            .agency_id
            .as_ref()
            .map(|id| id.as_ref().to_string()),
        GtfsTarget::Stops => Some(feed.stops[idx].stop_id.as_ref().to_string()),
        GtfsTarget::Routes => Some(feed.routes[idx].route_id.as_ref().to_string()),
        GtfsTarget::Trips => Some(feed.trips[idx].trip_id.as_ref().to_string()),
        GtfsTarget::Calendar => Some(feed.calendars[idx].service_id.as_ref().to_string()),
        GtfsTarget::Pathways => Some(feed.pathways[idx].pathway_id.as_ref().to_string()),
        GtfsTarget::Levels => Some(feed.levels[idx].level_id.as_ref().to_string()),
        GtfsTarget::FareAttributes => Some(feed.fare_attributes[idx].fare_id.as_ref().to_string()),
        // Composite or no PK — not handled here
        _ => None,
    }
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

#[allow(clippy::too_many_lines)]
fn validate_types(
    feed: &GtfsFeed,
    target: GtfsTarget,
    matched_indices: &[usize],
    assignments: &[FieldAssignment],
) -> Result<(), UpdateError> {
    macro_rules! validate_on_clone {
        ($records:expr, $setter:ident) => {{
            let mut clone = $records[matched_indices[0]].clone();
            for a in assignments {
                $setter(&mut clone, &a.field, &a.value)?;
            }
        }};
    }

    match target {
        GtfsTarget::Agency => validate_on_clone!(feed.agencies, set_agency_field),
        GtfsTarget::Stops => validate_on_clone!(feed.stops, set_stop_field),
        GtfsTarget::Routes => validate_on_clone!(feed.routes, set_route_field),
        GtfsTarget::Trips => validate_on_clone!(feed.trips, set_trip_field),
        GtfsTarget::StopTimes => validate_on_clone!(feed.stop_times, set_stop_time_field),
        GtfsTarget::Calendar => validate_on_clone!(feed.calendars, set_calendar_field),
        GtfsTarget::CalendarDates => {
            validate_on_clone!(feed.calendar_dates, set_calendar_date_field);
        }
        GtfsTarget::Shapes => validate_on_clone!(feed.shapes, set_shape_field),
        GtfsTarget::Frequencies => validate_on_clone!(feed.frequencies, set_frequency_field),
        GtfsTarget::Transfers => validate_on_clone!(feed.transfers, set_transfer_field),
        GtfsTarget::Pathways => validate_on_clone!(feed.pathways, set_pathway_field),
        GtfsTarget::Levels => validate_on_clone!(feed.levels, set_level_field),
        GtfsTarget::FeedInfo => {
            if let Some(ref fi) = feed.feed_info {
                let mut clone = fi.clone();
                for a in assignments {
                    set_feed_info_field(&mut clone, &a.field, &a.value)?;
                }
            }
        }
        GtfsTarget::FareAttributes => {
            validate_on_clone!(feed.fare_attributes, set_fare_attribute_field);
        }
        GtfsTarget::FareRules => validate_on_clone!(feed.fare_rules, set_fare_rule_field),
        GtfsTarget::Translations => validate_on_clone!(feed.translations, set_translation_field),
        GtfsTarget::Attributions => validate_on_clone!(feed.attributions, set_attribution_field),
    }

    Ok(())
}

macro_rules! apply_updates {
    ($records:expr, $plan:expr, $setter:ident) => {{
        for &idx in &$plan.matched_indices {
            for a in &$plan.assignments {
                $setter(&mut $records[idx], &a.field, &a.value).unwrap();
            }
        }
        $plan.matched_count
    }};
}

/// Applies a validated [`UpdatePlan`] by mutating the matched records in-place,
/// including cascade updates to dependent files if present.
///
/// # Panics
///
/// Panics if any setter call fails, which should never happen since the plan
/// was already validated by [`validate_update`].
#[allow(clippy::too_many_lines)]
pub fn apply_update(feed: &mut GtfsFeed, plan: &UpdatePlan) -> UpdateResult {
    let count = match plan.target {
        GtfsTarget::Agency => apply_updates!(feed.agencies, plan, set_agency_field),
        GtfsTarget::Stops => apply_updates!(feed.stops, plan, set_stop_field),
        GtfsTarget::Routes => apply_updates!(feed.routes, plan, set_route_field),
        GtfsTarget::Trips => apply_updates!(feed.trips, plan, set_trip_field),
        GtfsTarget::StopTimes => apply_updates!(feed.stop_times, plan, set_stop_time_field),
        GtfsTarget::Calendar => apply_updates!(feed.calendars, plan, set_calendar_field),
        GtfsTarget::CalendarDates => {
            apply_updates!(feed.calendar_dates, plan, set_calendar_date_field)
        }
        GtfsTarget::Shapes => apply_updates!(feed.shapes, plan, set_shape_field),
        GtfsTarget::Frequencies => apply_updates!(feed.frequencies, plan, set_frequency_field),
        GtfsTarget::Transfers => apply_updates!(feed.transfers, plan, set_transfer_field),
        GtfsTarget::Pathways => apply_updates!(feed.pathways, plan, set_pathway_field),
        GtfsTarget::Levels => apply_updates!(feed.levels, plan, set_level_field),
        GtfsTarget::FeedInfo => {
            if let Some(ref mut fi) = feed.feed_info {
                for a in &plan.assignments {
                    set_feed_info_field(fi, &a.field, &a.value).unwrap();
                }
            }
            plan.matched_count
        }
        GtfsTarget::FareAttributes => {
            apply_updates!(feed.fare_attributes, plan, set_fare_attribute_field)
        }
        GtfsTarget::FareRules => apply_updates!(feed.fare_rules, plan, set_fare_rule_field),
        GtfsTarget::Translations => {
            apply_updates!(feed.translations, plan, set_translation_field)
        }
        GtfsTarget::Attributions => {
            apply_updates!(feed.attributions, plan, set_attribution_field)
        }
    };

    let mut modified_targets = vec![plan.target];

    if let Some(ref cascade) = plan.cascade {
        apply_cascade(feed, cascade);
        for entry in &cascade.entries {
            if !modified_targets.contains(&entry.dependent) {
                modified_targets.push(entry.dependent);
            }
        }
    }

    UpdateResult {
        count,
        modified_targets,
    }
}

fn apply_cascade(feed: &mut GtfsFeed, cascade: &CascadePlan) {
    let old = &cascade.old_value;
    let new = &cascade.new_value;

    for entry in &cascade.entries {
        for &fk_field in &entry.fk_fields {
            apply_cascade_to_target(feed, entry.dependent, fk_field, old, new);
        }
    }
}

#[allow(clippy::too_many_lines)]
fn apply_cascade_to_target(
    feed: &mut GtfsFeed,
    target: GtfsTarget,
    fk_field: &str,
    old_value: &str,
    new_value: &str,
) {
    macro_rules! cascade_field {
        ($records:expr, $setter:ident) => {
            for record in &mut $records {
                if record.field_value(fk_field).as_deref() == Some(old_value) {
                    $setter(record, fk_field, new_value).unwrap();
                }
            }
        };
    }

    match target {
        GtfsTarget::Agency => cascade_field!(feed.agencies, set_agency_field),
        GtfsTarget::Stops => cascade_field!(feed.stops, set_stop_field),
        GtfsTarget::Routes => cascade_field!(feed.routes, set_route_field),
        GtfsTarget::Trips => cascade_field!(feed.trips, set_trip_field),
        GtfsTarget::StopTimes => cascade_field!(feed.stop_times, set_stop_time_field),
        GtfsTarget::Calendar => cascade_field!(feed.calendars, set_calendar_field),
        GtfsTarget::CalendarDates => cascade_field!(feed.calendar_dates, set_calendar_date_field),
        GtfsTarget::Shapes => cascade_field!(feed.shapes, set_shape_field),
        GtfsTarget::Frequencies => cascade_field!(feed.frequencies, set_frequency_field),
        GtfsTarget::Transfers => cascade_field!(feed.transfers, set_transfer_field),
        GtfsTarget::Pathways => cascade_field!(feed.pathways, set_pathway_field),
        GtfsTarget::Levels => cascade_field!(feed.levels, set_level_field),
        GtfsTarget::FeedInfo => {
            if let Some(ref mut fi) = feed.feed_info
                && fi.field_value(fk_field).as_deref() == Some(old_value)
            {
                set_feed_info_field(fi, fk_field, new_value).unwrap();
            }
        }
        GtfsTarget::FareAttributes => {
            cascade_field!(feed.fare_attributes, set_fare_attribute_field);
        }
        GtfsTarget::FareRules => cascade_field!(feed.fare_rules, set_fare_rule_field),
        GtfsTarget::Translations => cascade_field!(feed.translations, set_translation_field),
        GtfsTarget::Attributions => cascade_field!(feed.attributions, set_attribution_field),
    }
}
