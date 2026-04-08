//! Record deletion for GTFS feeds.
//!
//! Provides a two-phase API: [`validate_delete`] builds a [`DeletePlan`]
//! describing what will be removed (including cascade dependents), then
//! [`apply_delete`] mutates the feed.

use std::collections::{HashSet, VecDeque};

use thiserror::Error;

use crate::crud::common::{
    CascadeEntry, find_matching_indices, get_pk_value, make_entity_ref, primary_key_fields,
};
use crate::crud::query::{Query, QueryError};
use crate::crud::read::GtfsTarget;
use crate::integrity::{EntityRef, IntegrityIndex};
use crate::models::{
    Agency, Attribution, Calendar, CalendarDate, FareAttribute, FareRule, FeedInfo, Frequency,
    GtfsDate, GtfsFeed, Level, Pathway, Route, Shape, Stop, StopTime, Transfer, Translation, Trip,
};
use crate::parser::feed_source::GtfsFiles;

/// Errors that can occur during record deletion.
#[derive(Debug, Error)]
pub enum DeleteError {
    #[error("Missing --where filter. Refusing to delete without filter.")]
    MissingWhereFilter,

    #[error("{0}")]
    QueryError(#[from] QueryError),
}

/// The validated deletion plan ready to be applied to the feed.
#[derive(Debug)]
pub struct DeletePlan {
    pub target: GtfsTarget,
    pub file_name: &'static str,
    pub matched_count: usize,
    pub matched_pks: Vec<String>,
    pub(crate) matched_indices: Vec<usize>,
    pub cascade: Option<DeleteCascadePlan>,
}

/// Cascade plan: lists all transitive dependents that will be removed.
#[derive(Debug)]
pub struct DeleteCascadePlan {
    pub entries: Vec<CascadeEntry>,
    pub(crate) dependents: HashSet<EntityRef>,
}

/// Result of applying a deletion.
#[derive(Debug)]
pub struct DeleteResult {
    pub primary_count: usize,
    pub cascade_counts: Vec<(GtfsTarget, usize)>,
    pub modified_targets: Vec<GtfsTarget>,
}

/// Returns `true` if `target` can have records that reference it (i.e. it has
/// a PK that other files use as FK). Leaf targets never trigger cascade.
#[must_use]
pub fn can_have_dependents(target: GtfsTarget) -> bool {
    !direct_dependent_files(target).is_empty()
}

/// Returns the GTFS files that must be loaded to validate a delete on `target`.
///
/// For leaf targets (no dependents), only the target file itself is needed.
/// For targets with dependents, transitively expands the dependency chain so
/// that `find_dependents_recursive` can discover the full cascade tree.
#[must_use]
pub fn required_files(target: GtfsTarget) -> Vec<GtfsFiles> {
    let target_file = target_to_gtfs_file(target);

    if !can_have_dependents(target) {
        return vec![target_file];
    }

    // BFS: transitively collect all files reachable through the dependency graph
    let mut files = HashSet::new();
    files.insert(target_file);

    let mut queue = VecDeque::new();
    queue.push_back(target);

    while let Some(t) = queue.pop_front() {
        for dep_file in direct_dependent_files(t) {
            if files.insert(dep_file)
                && let Some(dep_target) = gtfs_file_to_target(dep_file)
            {
                queue.push_back(dep_target);
            }
        }
    }

    files.into_iter().collect()
}

/// Direct dependent files (files that reference this target's PKs).
fn direct_dependent_files(target: GtfsTarget) -> Vec<GtfsFiles> {
    use GtfsFiles as F;
    match target {
        GtfsTarget::Agency => vec![F::Routes, F::FareAttributes, F::Attributions],
        GtfsTarget::Stops => vec![F::Stops, F::StopTimes, F::Transfers, F::Pathways],
        GtfsTarget::Routes => vec![F::Trips, F::FareRules, F::Transfers, F::Attributions],
        GtfsTarget::Trips => vec![F::StopTimes, F::Frequencies, F::Transfers, F::Attributions],
        GtfsTarget::Calendar | GtfsTarget::CalendarDates => vec![F::CalendarDates, F::Trips],
        GtfsTarget::Shapes => vec![F::Trips],
        GtfsTarget::Levels => vec![F::Stops],
        GtfsTarget::FareAttributes => vec![F::FareRules],
        _ => vec![],
    }
}

fn target_to_gtfs_file(target: GtfsTarget) -> GtfsFiles {
    match target {
        GtfsTarget::Agency => GtfsFiles::Agency,
        GtfsTarget::Stops => GtfsFiles::Stops,
        GtfsTarget::Routes => GtfsFiles::Routes,
        GtfsTarget::Trips => GtfsFiles::Trips,
        GtfsTarget::StopTimes => GtfsFiles::StopTimes,
        GtfsTarget::Calendar => GtfsFiles::Calendar,
        GtfsTarget::CalendarDates => GtfsFiles::CalendarDates,
        GtfsTarget::Shapes => GtfsFiles::Shapes,
        GtfsTarget::Frequencies => GtfsFiles::Frequencies,
        GtfsTarget::Transfers => GtfsFiles::Transfers,
        GtfsTarget::Pathways => GtfsFiles::Pathways,
        GtfsTarget::Levels => GtfsFiles::Levels,
        GtfsTarget::FeedInfo => GtfsFiles::FeedInfo,
        GtfsTarget::FareAttributes => GtfsFiles::FareAttributes,
        GtfsTarget::FareRules => GtfsFiles::FareRules,
        GtfsTarget::Translations => GtfsFiles::Translations,
        GtfsTarget::Attributions => GtfsFiles::Attributions,
    }
}

fn gtfs_file_to_target(file: GtfsFiles) -> Option<GtfsTarget> {
    match file {
        GtfsFiles::Agency => Some(GtfsTarget::Agency),
        GtfsFiles::Stops => Some(GtfsTarget::Stops),
        GtfsFiles::Routes => Some(GtfsTarget::Routes),
        GtfsFiles::Trips => Some(GtfsTarget::Trips),
        GtfsFiles::StopTimes => Some(GtfsTarget::StopTimes),
        GtfsFiles::Calendar => Some(GtfsTarget::Calendar),
        GtfsFiles::CalendarDates => Some(GtfsTarget::CalendarDates),
        GtfsFiles::Shapes => Some(GtfsTarget::Shapes),
        GtfsFiles::Frequencies => Some(GtfsTarget::Frequencies),
        GtfsFiles::Transfers => Some(GtfsTarget::Transfers),
        GtfsFiles::Pathways => Some(GtfsTarget::Pathways),
        GtfsFiles::Levels => Some(GtfsTarget::Levels),
        GtfsFiles::FeedInfo => Some(GtfsTarget::FeedInfo),
        GtfsFiles::FareAttributes => Some(GtfsTarget::FareAttributes),
        GtfsFiles::FareRules => Some(GtfsTarget::FareRules),
        GtfsFiles::Translations => Some(GtfsTarget::Translations),
        GtfsFiles::Attributions => Some(GtfsTarget::Attributions),
        _ => None, // Fares v2 files not yet in CrudTarget
    }
}

/// Validates a delete operation and builds a [`DeletePlan`] without mutating
/// the feed.
///
/// # Errors
///
/// Returns [`DeleteError`] on query validation failure.
pub fn validate_delete(
    feed: &GtfsFeed,
    target: GtfsTarget,
    query: &Query,
) -> Result<DeletePlan, DeleteError> {
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

    if matched_indices.is_empty() {
        return Ok(DeletePlan {
            target,
            file_name: target.file_name(),
            matched_count: 0,
            matched_pks: Vec::new(),
            matched_indices,
            cascade: None,
        });
    }

    let matched_pks = extract_pk_display(feed, target, &matched_indices);

    let cascade = if can_have_dependents(target) {
        let integrity = IntegrityIndex::build_from_feed(feed);
        let mut all_dependents: HashSet<EntityRef> = HashSet::new();

        for &idx in &matched_indices {
            let Some(pk) = get_pk_value(feed, target, idx) else {
                continue;
            };
            let Some(entity) = make_entity_ref(target, &pk) else {
                continue;
            };

            for (dep, _) in integrity.find_dependents_recursive(&entity) {
                all_dependents.insert(dep);
            }
        }

        if all_dependents.is_empty() {
            None
        } else {
            let entries = group_dependents_by_target(&all_dependents);
            Some(DeleteCascadePlan {
                entries,
                dependents: all_dependents,
            })
        }
    } else {
        None
    };

    Ok(DeletePlan {
        target,
        file_name: target.file_name(),
        matched_count,
        matched_pks,
        matched_indices,
        cascade,
    })
}

/// Applies a validated [`DeletePlan`], removing matched records and cascade
/// dependents.
pub fn apply_delete(feed: &mut GtfsFeed, plan: &DeletePlan) -> DeleteResult {
    let matched_set: HashSet<usize> = plan.matched_indices.iter().copied().collect();

    remove_by_indices(feed, plan.target, &matched_set);

    let mut modified_targets = vec![plan.target];
    let mut cascade_counts = Vec::new();

    if let Some(ref cascade) = plan.cascade {
        for entry in &cascade.entries {
            let removed = remove_by_entity_refs(feed, entry.dependent, &cascade.dependents);
            cascade_counts.push((entry.dependent, removed));
            if !modified_targets.contains(&entry.dependent) {
                modified_targets.push(entry.dependent);
            }
        }
    }

    DeleteResult {
        primary_count: plan.matched_count,
        cascade_counts,
        modified_targets,
    }
}

fn extract_pk_display(feed: &GtfsFeed, target: GtfsTarget, indices: &[usize]) -> Vec<String> {
    let pk_fields = primary_key_fields(target);

    indices
        .iter()
        .map(|&idx| {
            if let Some(pk) = get_pk_value(feed, target, idx) {
                if pk_fields.len() == 1 {
                    return format!("{}={}", pk_fields[0], pk);
                }
                return format!("{}={}", pk_fields[0], pk);
            }

            match target {
                GtfsTarget::StopTimes => {
                    let st = &feed.stop_times[idx];
                    format!(
                        "trip_id={}, stop_sequence={}",
                        st.trip_id.as_ref(),
                        st.stop_sequence
                    )
                }
                GtfsTarget::CalendarDates => {
                    let cd = &feed.calendar_dates[idx];
                    format!("service_id={}, date={}", cd.service_id.as_ref(), cd.date)
                }
                GtfsTarget::Shapes => {
                    let s = &feed.shapes[idx];
                    format!(
                        "shape_id={}, shape_pt_sequence={}",
                        s.shape_id.as_ref(),
                        s.shape_pt_sequence
                    )
                }
                GtfsTarget::Frequencies => {
                    let f = &feed.frequencies[idx];
                    format!(
                        "trip_id={}, start_time={}",
                        f.trip_id.as_ref(),
                        f.start_time
                    )
                }
                _ => format!("#{idx}"),
            }
        })
        .collect()
}

fn group_dependents_by_target(dependents: &HashSet<EntityRef>) -> Vec<CascadeEntry> {
    let mut counts: std::collections::HashMap<GtfsTarget, usize> = std::collections::HashMap::new();

    for dep in dependents {
        *counts.entry(dep.target()).or_default() += 1;
    }

    counts
        .into_iter()
        .map(|(target, count)| CascadeEntry {
            dependent: target,
            fk_fields: Vec::new(),
            count,
        })
        .collect()
}

fn remove_by_indices(feed: &mut GtfsFeed, target: GtfsTarget, indices: &HashSet<usize>) {
    macro_rules! retain_by_idx {
        ($records:expr) => {{
            let mut idx = 0usize;
            $records.retain(|_| {
                let keep = !indices.contains(&idx);
                idx += 1;
                keep
            });
        }};
    }

    match target {
        GtfsTarget::Agency => retain_by_idx!(feed.agencies),
        GtfsTarget::Stops => retain_by_idx!(feed.stops),
        GtfsTarget::Routes => retain_by_idx!(feed.routes),
        GtfsTarget::Trips => retain_by_idx!(feed.trips),
        GtfsTarget::StopTimes => retain_by_idx!(feed.stop_times),
        GtfsTarget::Calendar => retain_by_idx!(feed.calendars),
        GtfsTarget::CalendarDates => retain_by_idx!(feed.calendar_dates),
        GtfsTarget::Shapes => retain_by_idx!(feed.shapes),
        GtfsTarget::Frequencies => retain_by_idx!(feed.frequencies),
        GtfsTarget::Transfers => retain_by_idx!(feed.transfers),
        GtfsTarget::Pathways => retain_by_idx!(feed.pathways),
        GtfsTarget::Levels => retain_by_idx!(feed.levels),
        GtfsTarget::FeedInfo => {
            if indices.contains(&0) {
                feed.feed_info = None;
            }
        }
        GtfsTarget::FareAttributes => retain_by_idx!(feed.fare_attributes),
        GtfsTarget::FareRules => retain_by_idx!(feed.fare_rules),
        GtfsTarget::Translations => retain_by_idx!(feed.translations),
        GtfsTarget::Attributions => retain_by_idx!(feed.attributions),
    }
}

macro_rules! remove_by_pk {
    ($records:expr, $refs:expr, $variant:pat => $id:ident, |$r:ident| $key:expr) => {{
        let ids: HashSet<&str> = $refs
            .iter()
            .filter_map(|e| match e {
                $variant => Some($id.as_ref()),
                _ => None,
            })
            .collect();
        let before = $records.len();
        $records.retain(|$r| !ids.contains($key));
        before - $records.len()
    }};
}

macro_rules! remove_by_composite {
    ($records:expr, $refs:expr, $K:ty, $variant:pat => $key:expr, |$r:ident| $rec_key:expr) => {{
        let keys: HashSet<$K> = $refs
            .iter()
            .filter_map(|e| match e {
                $variant => Some($key),
                _ => None,
            })
            .collect();
        let before = $records.len();
        $records.retain(|$r| !keys.contains(&$rec_key));
        before - $records.len()
    }};
}

macro_rules! remove_by_index {
    ($records:expr, $refs:expr, $variant:pat => $i:ident) => {{
        let indices: HashSet<usize> = $refs
            .iter()
            .filter_map(|e| match e {
                $variant => Some(*$i),
                _ => None,
            })
            .collect();
        let before = $records.len();
        let mut idx = 0usize;
        $records.retain(|_| {
            let keep = !indices.contains(&idx);
            idx += 1;
            keep
        });
        before - $records.len()
    }};
}

/// Removes records from a target whose [`EntityRef`] is in the given set.
/// Returns the number of records removed.
fn remove_by_entity_refs(
    feed: &mut GtfsFeed,
    target: GtfsTarget,
    refs: &HashSet<EntityRef>,
) -> usize {
    match target {
        GtfsTarget::Agency => {
            let ids: HashSet<&str> = refs
                .iter()
                .filter_map(|e| match e {
                    EntityRef::Agency(id) => Some(id.as_ref()),
                    _ => None,
                })
                .collect();
            let before = feed.agencies.len();
            feed.agencies.retain(|a| {
                a.agency_id
                    .as_ref()
                    .is_none_or(|id| !ids.contains(id.as_ref()))
            });
            before - feed.agencies.len()
        }
        GtfsTarget::Stops => {
            remove_by_pk!(feed.stops, refs, EntityRef::Stop(id) => id, |s| s.stop_id.as_ref())
        }
        GtfsTarget::Routes => {
            remove_by_pk!(feed.routes, refs, EntityRef::Route(id) => id, |r| r.route_id.as_ref())
        }
        GtfsTarget::Trips => {
            remove_by_pk!(feed.trips, refs, EntityRef::Trip(id) => id, |t| t.trip_id.as_ref())
        }
        GtfsTarget::Calendar => {
            remove_by_pk!(feed.calendars, refs, EntityRef::Service(id) => id, |c| c.service_id.as_ref())
        }
        GtfsTarget::Pathways => {
            remove_by_pk!(feed.pathways, refs, EntityRef::Pathway(id) => id, |p| p.pathway_id.as_ref())
        }
        GtfsTarget::Levels => {
            remove_by_pk!(feed.levels, refs, EntityRef::Level(id) => id, |l| l.level_id.as_ref())
        }
        GtfsTarget::FareAttributes => {
            remove_by_pk!(feed.fare_attributes, refs, EntityRef::Fare(id) => id, |fa| fa.fare_id.as_ref())
        }
        GtfsTarget::StopTimes => remove_by_composite!(
            feed.stop_times, refs, (&str, u32),
            EntityRef::StopTime(tid, seq) => (tid.as_ref(), *seq),
            |st| (st.trip_id.as_ref(), st.stop_sequence)
        ),
        GtfsTarget::CalendarDates => remove_by_composite!(
            feed.calendar_dates, refs, (&str, GtfsDate),
            EntityRef::CalendarDate(svc, d) => (svc.as_ref(), *d),
            |cd| (cd.service_id.as_ref(), cd.date)
        ),
        GtfsTarget::Shapes => remove_by_composite!(
            feed.shapes, refs, (&str, u32),
            EntityRef::ShapePoint(id, seq) => (id.as_ref(), *seq),
            |s| (s.shape_id.as_ref(), s.shape_pt_sequence)
        ),
        GtfsTarget::Frequencies => remove_by_composite!(
            feed.frequencies, refs, (&str, u32),
            EntityRef::Frequency(tid, secs) => (tid.as_ref(), *secs),
            |f| (f.trip_id.as_ref(), f.start_time.total_seconds)
        ),
        GtfsTarget::Transfers => {
            remove_by_index!(feed.transfers, refs, EntityRef::Transfer(i) => i)
        }
        GtfsTarget::FareRules => {
            remove_by_index!(feed.fare_rules, refs, EntityRef::FareRule(i) => i)
        }
        GtfsTarget::Attributions => {
            remove_by_index!(feed.attributions, refs, EntityRef::Attribution(i) => i)
        }
        GtfsTarget::FeedInfo => {
            if feed.feed_info.is_some() {
                feed.feed_info = None;
                1
            } else {
                0
            }
        }
        GtfsTarget::Translations => 0,
    }
}
