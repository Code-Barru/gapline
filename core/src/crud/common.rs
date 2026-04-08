//! Shared types and helpers for CRUD operations on GTFS feeds.

use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::crud::query::Filterable;
use crate::crud::query::Query;
use crate::crud::read::GtfsTarget;
use crate::integrity::EntityRef;
use crate::models::{
    Agency, Attribution, Calendar, CalendarDate, FareAttribute, FareRule, FeedInfo, Frequency,
    GtfsDate, GtfsFeed, GtfsTime, Level, Pathway, Route, Shape, Stop, StopTime, Transfer,
    Translation, Trip,
};

/// Errors shared across CRUD operations (create, update, delete).
#[derive(Debug, Error)]
pub enum CrudError {
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
}

/// A parsed `field=value` pair from `--set`.
#[derive(Debug, Clone)]
pub struct FieldAssignment {
    pub field: String,
    pub value: String,
}

/// Convenience alias for a field-name → value map built from assignments.
pub type Fields<'a> = HashMap<&'a str, &'a str>;

/// Parses raw `"field=value"` strings into [`FieldAssignment`]s.
///
/// # Errors
///
/// Returns an error on invalid input.
pub fn parse_assignments(raw: &[String]) -> Result<Vec<FieldAssignment>, CrudError> {
    if raw.is_empty() {
        return Err(CrudError::EmptyAssignments);
    }

    raw.iter()
        .map(|s| {
            let (field, value) = s
                .split_once('=')
                .ok_or_else(|| CrudError::InvalidAssignment(s.clone()))?;
            if field.is_empty() {
                return Err(CrudError::InvalidAssignment(s.clone()));
            }
            Ok(FieldAssignment {
                field: field.to_string(),
                value: value.to_string(),
            })
        })
        .collect()
}

/// Validates field names against the target's known fields and builds a
/// lookup map. Rejects unknown fields and duplicate assignments.
///
/// # Errors
///
/// Returns an error on invalid input.
pub fn to_field_map(
    assignments: &[FieldAssignment],
    target: GtfsTarget,
) -> Result<Fields<'_>, CrudError> {
    let valid = valid_fields_for(target);
    let mut map = HashMap::new();

    for a in assignments {
        if !valid.contains(&a.field.as_str()) {
            return Err(CrudError::UnknownField {
                field: a.field.clone(),
                valid: valid.join(", "),
            });
        }
        if map.insert(a.field.as_str(), a.value.as_str()).is_some() {
            return Err(CrudError::DuplicateAssignment(a.field.clone()));
        }
    }

    Ok(map)
}

/// Returns the list of recognized field names for the given target.
#[must_use]
pub fn valid_fields_for(target: GtfsTarget) -> &'static [&'static str] {
    match target {
        GtfsTarget::Agency => Agency::valid_fields(),
        GtfsTarget::Stops => Stop::valid_fields(),
        GtfsTarget::Routes => Route::valid_fields(),
        GtfsTarget::Trips => Trip::valid_fields(),
        GtfsTarget::StopTimes => StopTime::valid_fields(),
        GtfsTarget::Calendar => Calendar::valid_fields(),
        GtfsTarget::CalendarDates => CalendarDate::valid_fields(),
        GtfsTarget::Shapes => Shape::valid_fields(),
        GtfsTarget::Frequencies => Frequency::valid_fields(),
        GtfsTarget::Transfers => Transfer::valid_fields(),
        GtfsTarget::Pathways => Pathway::valid_fields(),
        GtfsTarget::Levels => Level::valid_fields(),
        GtfsTarget::FeedInfo => FeedInfo::valid_fields(),
        GtfsTarget::FareAttributes => FareAttribute::valid_fields(),
        GtfsTarget::FareRules => FareRule::valid_fields(),
        GtfsTarget::Translations => Translation::valid_fields(),
        GtfsTarget::Attributions => Attribution::valid_fields(),
    }
}

/// Pre-built lookup sets for PK/FK validation.
/// Only the sets relevant to the target being validated are populated.
pub struct FeedIndex<'a> {
    pub agency_ids: HashSet<&'a str>,
    pub stop_ids: HashSet<&'a str>,
    pub route_ids: HashSet<&'a str>,
    pub trip_ids: HashSet<&'a str>,
    pub service_ids: HashSet<&'a str>, // union of calendar + calendar_dates
    pub pathway_ids: HashSet<&'a str>,
    pub level_ids: HashSet<&'a str>,
    pub fare_ids: HashSet<&'a str>,
    pub stop_time_pks: HashSet<(&'a str, u32)>,
    pub calendar_date_pks: HashSet<(&'a str, GtfsDate)>,
    pub shape_pks: HashSet<(&'a str, u32)>,
    pub frequency_pks: HashSet<(&'a str, GtfsTime)>,
    pub has_feed_info: bool,
}

impl<'a> FeedIndex<'a> {
    /// Builds only the index sets required for the given `target`.
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn build(feed: &'a GtfsFeed, target: GtfsTarget) -> Self {
        let mut idx = Self {
            agency_ids: HashSet::new(),
            stop_ids: HashSet::new(),
            route_ids: HashSet::new(),
            trip_ids: HashSet::new(),
            service_ids: HashSet::new(),
            pathway_ids: HashSet::new(),
            level_ids: HashSet::new(),
            fare_ids: HashSet::new(),
            stop_time_pks: HashSet::new(),
            calendar_date_pks: HashSet::new(),
            shape_pks: HashSet::new(),
            frequency_pks: HashSet::new(),
            has_feed_info: false,
        };

        match target {
            GtfsTarget::Agency => {
                idx.agency_ids = feed
                    .agencies
                    .iter()
                    .filter_map(|a| a.agency_id.as_ref().map(std::convert::AsRef::as_ref))
                    .collect();
            }
            GtfsTarget::Stops => {
                idx.stop_ids = feed.stops.iter().map(|s| s.stop_id.as_ref()).collect();
                idx.level_ids = feed.levels.iter().map(|l| l.level_id.as_ref()).collect();
            }
            GtfsTarget::Routes => {
                idx.route_ids = feed.routes.iter().map(|r| r.route_id.as_ref()).collect();
                idx.agency_ids = feed
                    .agencies
                    .iter()
                    .filter_map(|a| a.agency_id.as_ref().map(std::convert::AsRef::as_ref))
                    .collect();
            }
            GtfsTarget::Trips => {
                idx.trip_ids = feed.trips.iter().map(|t| t.trip_id.as_ref()).collect();
                idx.route_ids = feed.routes.iter().map(|r| r.route_id.as_ref()).collect();
                idx.service_ids = feed
                    .calendars
                    .iter()
                    .map(|c| c.service_id.as_ref())
                    .chain(feed.calendar_dates.iter().map(|cd| cd.service_id.as_ref()))
                    .collect();
            }
            GtfsTarget::StopTimes => {
                idx.stop_time_pks = feed
                    .stop_times
                    .iter()
                    .map(|st| (st.trip_id.as_ref(), st.stop_sequence))
                    .collect();
                idx.trip_ids = feed.trips.iter().map(|t| t.trip_id.as_ref()).collect();
                idx.stop_ids = feed.stops.iter().map(|s| s.stop_id.as_ref()).collect();
            }
            GtfsTarget::Calendar => {
                idx.service_ids = feed
                    .calendars
                    .iter()
                    .map(|c| c.service_id.as_ref())
                    .collect();
            }
            GtfsTarget::CalendarDates => {
                idx.calendar_date_pks = feed
                    .calendar_dates
                    .iter()
                    .map(|cd| (cd.service_id.as_ref(), cd.date))
                    .collect();
                idx.service_ids = feed
                    .calendars
                    .iter()
                    .map(|c| c.service_id.as_ref())
                    .chain(feed.calendar_dates.iter().map(|cd| cd.service_id.as_ref()))
                    .collect();
            }
            GtfsTarget::Shapes => {
                idx.shape_pks = feed
                    .shapes
                    .iter()
                    .map(|s| (s.shape_id.as_ref(), s.shape_pt_sequence))
                    .collect();
            }
            GtfsTarget::Frequencies => {
                idx.frequency_pks = feed
                    .frequencies
                    .iter()
                    .map(|f| (f.trip_id.as_ref(), f.start_time))
                    .collect();
                idx.trip_ids = feed.trips.iter().map(|t| t.trip_id.as_ref()).collect();
            }
            GtfsTarget::Transfers => {
                idx.stop_ids = feed.stops.iter().map(|s| s.stop_id.as_ref()).collect();
                idx.route_ids = feed.routes.iter().map(|r| r.route_id.as_ref()).collect();
                idx.trip_ids = feed.trips.iter().map(|t| t.trip_id.as_ref()).collect();
            }
            GtfsTarget::Pathways => {
                idx.pathway_ids = feed
                    .pathways
                    .iter()
                    .map(|p| p.pathway_id.as_ref())
                    .collect();
                idx.stop_ids = feed.stops.iter().map(|s| s.stop_id.as_ref()).collect();
            }
            GtfsTarget::Levels => {
                idx.level_ids = feed.levels.iter().map(|l| l.level_id.as_ref()).collect();
            }
            GtfsTarget::FeedInfo => {
                idx.has_feed_info = feed.feed_info.is_some();
            }
            GtfsTarget::FareAttributes => {
                idx.fare_ids = feed
                    .fare_attributes
                    .iter()
                    .map(|fa| fa.fare_id.as_ref())
                    .collect();
                idx.agency_ids = feed
                    .agencies
                    .iter()
                    .filter_map(|a| a.agency_id.as_ref().map(std::convert::AsRef::as_ref))
                    .collect();
            }
            GtfsTarget::FareRules => {
                idx.fare_ids = feed
                    .fare_attributes
                    .iter()
                    .map(|fa| fa.fare_id.as_ref())
                    .collect();
                idx.route_ids = feed.routes.iter().map(|r| r.route_id.as_ref()).collect();
            }
            GtfsTarget::Translations => {}
            GtfsTarget::Attributions => {
                idx.agency_ids = feed
                    .agencies
                    .iter()
                    .filter_map(|a| a.agency_id.as_ref().map(std::convert::AsRef::as_ref))
                    .collect();
                idx.route_ids = feed.routes.iter().map(|r| r.route_id.as_ref()).collect();
                idx.trip_ids = feed.trips.iter().map(|t| t.trip_id.as_ref()).collect();
            }
        }

        idx
    }
}

/// Validates that all FK field values in `fields` reference existing entities.
///
/// Uses the pre-built [`FeedIndex`] for O(1) lookups.
///
/// # Errors
///
/// Returns an error on invalid input.
pub fn validate_foreign_keys(
    idx: &FeedIndex,
    target: GtfsTarget,
    fields: &Fields,
) -> Result<(), CrudError> {
    match target {
        GtfsTarget::Routes | GtfsTarget::FareAttributes => {
            fk_check(fields, "agency_id", "agency.txt", &idx.agency_ids)?;
        }
        GtfsTarget::Trips => {
            fk_check(fields, "route_id", "routes.txt", &idx.route_ids)?;
            fk_check(
                fields,
                "service_id",
                "calendar.txt / calendar_dates.txt",
                &idx.service_ids,
            )?;
        }
        GtfsTarget::StopTimes => {
            fk_check(fields, "trip_id", "trips.txt", &idx.trip_ids)?;
            fk_check(fields, "stop_id", "stops.txt", &idx.stop_ids)?;
        }
        GtfsTarget::Stops => {
            fk_check(fields, "parent_station", "stops.txt", &idx.stop_ids)?;
            fk_check(fields, "level_id", "levels.txt", &idx.level_ids)?;
        }
        GtfsTarget::CalendarDates => {
            fk_check(
                fields,
                "service_id",
                "calendar.txt / calendar_dates.txt",
                &idx.service_ids,
            )?;
        }
        GtfsTarget::Frequencies => {
            fk_check(fields, "trip_id", "trips.txt", &idx.trip_ids)?;
        }
        GtfsTarget::Transfers => {
            fk_check(fields, "from_stop_id", "stops.txt", &idx.stop_ids)?;
            fk_check(fields, "to_stop_id", "stops.txt", &idx.stop_ids)?;
            fk_check(fields, "from_route_id", "routes.txt", &idx.route_ids)?;
            fk_check(fields, "to_route_id", "routes.txt", &idx.route_ids)?;
            fk_check(fields, "from_trip_id", "trips.txt", &idx.trip_ids)?;
            fk_check(fields, "to_trip_id", "trips.txt", &idx.trip_ids)?;
        }
        GtfsTarget::Pathways => {
            fk_check(fields, "from_stop_id", "stops.txt", &idx.stop_ids)?;
            fk_check(fields, "to_stop_id", "stops.txt", &idx.stop_ids)?;
        }
        GtfsTarget::FareRules => {
            fk_check(fields, "fare_id", "fare_attributes.txt", &idx.fare_ids)?;
            fk_check(fields, "route_id", "routes.txt", &idx.route_ids)?;
        }
        GtfsTarget::Attributions => {
            fk_check(fields, "agency_id", "agency.txt", &idx.agency_ids)?;
            fk_check(fields, "route_id", "routes.txt", &idx.route_ids)?;
            fk_check(fields, "trip_id", "trips.txt", &idx.trip_ids)?;
        }
        _ => {}
    }
    Ok(())
}

/// Check that a field value (if present) exists in the given set. O(1).
///
/// # Errors
///
/// Returns an error on invalid input.
pub fn fk_check<S: ::std::hash::BuildHasher>(
    fields: &Fields,
    field: &str,
    referenced_file: &str,
    valid: &HashSet<&str, S>,
) -> Result<(), CrudError> {
    if let Some(&val) = fields.get(field)
        && !valid.contains(val)
    {
        return Err(CrudError::ForeignKeyViolation {
            field: field.to_string(),
            value: val.to_string(),
            referenced_file: referenced_file.to_string(),
        });
    }
    Ok(())
}

/// Checks that a simple (single-field) PK value does not already exist.
///
/// # Errors
///
/// Returns an error on invalid input.
pub fn pk_check_simple<S: ::std::hash::BuildHasher>(
    fields: &Fields,
    key: &str,
    set: &HashSet<&str, S>,
    file: &str,
) -> Result<(), CrudError> {
    if let Some(&id) = fields.get(key)
        && set.contains(id)
    {
        return Err(pk_err(key, id, file));
    }
    Ok(())
}

/// Builds a [`CrudError::DuplicatePrimaryKey`].
#[must_use]
pub fn pk_err(field: &str, value: &str, file: &str) -> CrudError {
    CrudError::DuplicatePrimaryKey {
        field: field.to_string(),
        value: value.to_string(),
        file: file.to_string(),
    }
}

/// A group of FK references in a single dependent file affected by a cascade.
#[derive(Debug)]
pub struct CascadeEntry {
    pub dependent: GtfsTarget,
    pub fk_fields: Vec<&'static str>,
    pub count: usize,
}

pub fn find_matching_indices<T: Filterable>(records: &[T], query: &Query) -> Vec<usize> {
    records
        .iter()
        .enumerate()
        .filter(|(_, r)| query.matches(*r))
        .map(|(i, _)| i)
        .collect()
}

/// Builds an [`EntityRef`] from a simple (single-field) primary key value.
///
/// Returns `None` for composite-PK or PK-less targets.
#[must_use]
pub fn make_entity_ref(target: GtfsTarget, pk_value: &str) -> Option<EntityRef> {
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
        _ => None,
    }
}

/// Extracts the simple primary key value of a record by its index in the feed.
///
/// Returns `None` for composite-PK or PK-less targets.
#[must_use]
pub fn get_pk_value(feed: &GtfsFeed, target: GtfsTarget, idx: usize) -> Option<String> {
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
        _ => None,
    }
}

/// Returns the primary key field names for the given target.
#[must_use]
pub fn primary_key_fields(target: GtfsTarget) -> &'static [&'static str] {
    match target {
        GtfsTarget::Agency => &["agency_id"],
        GtfsTarget::Stops => &["stop_id"],
        GtfsTarget::Routes => &["route_id"],
        GtfsTarget::Trips => &["trip_id"],
        GtfsTarget::StopTimes => &["trip_id", "stop_sequence"],
        GtfsTarget::Calendar => &["service_id"],
        GtfsTarget::CalendarDates => &["service_id", "date"],
        GtfsTarget::Shapes => &["shape_id", "shape_pt_sequence"],
        GtfsTarget::Frequencies => &["trip_id", "start_time"],
        GtfsTarget::Transfers
        | GtfsTarget::FeedInfo
        | GtfsTarget::FareRules
        | GtfsTarget::Translations
        | GtfsTarget::Attributions => &[],
        GtfsTarget::Pathways => &["pathway_id"],
        GtfsTarget::Levels => &["level_id"],
        GtfsTarget::FareAttributes => &["fare_id"],
    }
}
