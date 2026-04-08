//! Record creation for GTFS feeds.
//!
//! Provides a two-phase API: [`validate_create`] validates field assignments and
//! builds a [`CreatePlan`], then [`apply_create`] mutates the feed.

use thiserror::Error;

use crate::crud::common::{
    self, CrudError, FeedIndex, Fields, parse_assignments, to_field_map, validate_foreign_keys,
};
use crate::crud::read::GtfsTarget;
use crate::models::{
    Agency, AgencyId, Attribution, BikesAllowed, Calendar, CalendarDate, Color, ContinuousDropOff,
    ContinuousPickup, CurrencyCode, DirectionId, DropOffType, Email, ExactTimes, ExceptionType,
    FareAttribute, FareId, FareRule, FeedInfo, Frequency, GtfsDate, GtfsFeed, GtfsTime,
    IsBidirectional, LanguageCode, Latitude, Level, LevelId, LocationType, Longitude, Pathway,
    PathwayId, PathwayMode, Phone, PickupType, Route, RouteId, RouteType, ServiceId, Shape,
    ShapeId, Stop, StopId, StopTime, Timepoint, Timezone, Transfer, TransferType, Translation,
    Trip, TripId, Url, WheelchairAccessible,
};
use crate::parser::feed_source::GtfsFiles;

// Re-export so that existing callers (`use headway_core::crud::create::FieldAssignment`) keep working.
pub use crate::crud::common::FieldAssignment;

/// Errors that can occur during record creation.
#[derive(Debug, Error)]
pub enum CreateError {
    #[error("Invalid assignment \"{0}\": expected field=value")]
    InvalidAssignment(String),

    #[error("Duplicate assignment for field \"{0}\"")]
    DuplicateAssignment(String),

    #[error("Unknown field \"{field}\" (valid fields: {valid})")]
    UnknownField { field: String, valid: String },

    #[error("Missing required field: {0}")]
    MissingRequiredField(String),

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

    #[error("{field} is forbidden: {reason}")]
    ForbiddenField { field: String, reason: String },

    #[error("No field assignments provided")]
    EmptyAssignments,

    #[error("feed_info already exists (at most one row allowed)")]
    FeedInfoAlreadyExists,
}

impl From<CrudError> for CreateError {
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

/// The validated plan ready to be applied to the feed.
#[derive(Debug)]
pub struct CreatePlan {
    pub target: GtfsTarget,
    pub file_name: &'static str,
    pub record: CreatedRecord,
    pub display_fields: Vec<(String, String)>,
}

/// A built record, ready to be inserted into the feed.
#[derive(Debug)]
pub enum CreatedRecord {
    Agency(Agency),
    Stop(Stop),
    Route(Route),
    Trip(Trip),
    StopTime(StopTime),
    Calendar(Calendar),
    CalendarDate(CalendarDate),
    Shape(Shape),
    Frequency(Frequency),
    Transfer(Transfer),
    Pathway(Pathway),
    Level(Level),
    FeedInfo(FeedInfo),
    FareAttribute(FareAttribute),
    FareRule(FareRule),
    Translation(Translation),
    Attribution(Attribution),
}

/// Validates field assignments and builds a [`CreatePlan`] without mutating the feed.
///
/// Performs all checks: field name validity, required fields, type parsing,
/// primary key uniqueness, and foreign key integrity.
///
/// # Errors
///
/// Returns [`CreateError`] on any validation failure.
pub fn validate_create(
    feed: &GtfsFeed,
    target: GtfsTarget,
    raw_assignments: &[String],
) -> Result<CreatePlan, CreateError> {
    let assignments = parse_assignments(raw_assignments)?;
    let fields = to_field_map(&assignments, target)?;
    let display_fields: Vec<(String, String)> = assignments
        .iter()
        .map(|a| (a.field.clone(), a.value.clone()))
        .collect();

    // Required fields + conditional checks (including forbidden fields) run first
    check_required_for(feed, target, &fields)?;

    let index = FeedIndex::build(feed, target);
    validate_primary_key(&index, target, &fields)?;
    validate_foreign_keys(&index, target, &fields)?;

    let record = build_record(target, &fields)?;
    let file_name = target.file_name();

    Ok(CreatePlan {
        target,
        file_name,
        record,
        display_fields,
    })
}

/// Applies a validated [`CreatePlan`] by inserting the record into the feed.
pub fn apply_create(feed: &mut GtfsFeed, plan: CreatePlan) {
    match plan.record {
        CreatedRecord::Agency(r) => feed.agencies.push(r),
        CreatedRecord::Stop(r) => feed.stops.push(r),
        CreatedRecord::Route(r) => feed.routes.push(r),
        CreatedRecord::Trip(r) => feed.trips.push(r),
        CreatedRecord::StopTime(r) => feed.stop_times.push(r),
        CreatedRecord::Calendar(r) => feed.calendars.push(r),
        CreatedRecord::CalendarDate(r) => feed.calendar_dates.push(r),
        CreatedRecord::Shape(r) => feed.shapes.push(r),
        CreatedRecord::Frequency(r) => feed.frequencies.push(r),
        CreatedRecord::Transfer(r) => feed.transfers.push(r),
        CreatedRecord::Pathway(r) => feed.pathways.push(r),
        CreatedRecord::Level(r) => feed.levels.push(r),
        CreatedRecord::FeedInfo(r) => feed.feed_info = Some(r),
        CreatedRecord::FareAttribute(r) => feed.fare_attributes.push(r),
        CreatedRecord::FareRule(r) => feed.fare_rules.push(r),
        CreatedRecord::Translation(r) => feed.translations.push(r),
        CreatedRecord::Attribution(r) => feed.attributions.push(r),
    }
}

/// Returns the GTFS files that must be loaded to validate a create on `target`.
///
/// Includes the target file itself (for PK checks and the writer) plus any
/// files needed for FK validation.
#[must_use]
pub fn required_files(target: GtfsTarget) -> Vec<GtfsFiles> {
    use GtfsFiles as F;
    match target {
        GtfsTarget::Agency => vec![F::Agency],
        GtfsTarget::Stops => vec![F::Stops, F::Levels],
        GtfsTarget::Routes => vec![F::Routes, F::Agency],
        GtfsTarget::Trips => vec![F::Trips, F::Routes, F::Calendar, F::CalendarDates],
        GtfsTarget::StopTimes => vec![F::StopTimes, F::Trips, F::Stops],
        GtfsTarget::Calendar => vec![F::Calendar],
        GtfsTarget::CalendarDates => vec![F::CalendarDates, F::Calendar],
        GtfsTarget::Shapes => vec![F::Shapes],
        GtfsTarget::Frequencies => vec![F::Frequencies, F::Trips],
        GtfsTarget::Transfers => vec![F::Stops, F::Routes, F::Trips],
        GtfsTarget::Pathways => vec![F::Pathways, F::Stops],
        GtfsTarget::Levels => vec![F::Levels],
        GtfsTarget::FeedInfo => vec![F::FeedInfo],
        GtfsTarget::FareAttributes => vec![F::FareAttributes, F::Agency],
        GtfsTarget::FareRules => vec![F::FareAttributes, F::Routes],
        GtfsTarget::Translations => vec![F::Translations],
        GtfsTarget::Attributions => vec![F::Agency, F::Routes, F::Trips],
    }
}

fn check_required(fields: &Fields, required: &[&str]) -> Result<(), CreateError> {
    for &name in required {
        if !fields.contains_key(name) {
            return Err(CreateError::MissingRequiredField(name.to_string()));
        }
    }
    Ok(())
}

fn check_required_for(
    feed: &GtfsFeed,
    target: GtfsTarget,
    fields: &Fields,
) -> Result<(), CreateError> {
    use crate::parser::feed_source::GtfsFiles;

    let gtfs_file = match target {
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
    };

    check_required(fields, gtfs_file.required_columns())?;

    match target {
        GtfsTarget::Stops => check_stops_conditional(fields)?,
        GtfsTarget::Trips => check_trips_conditional(feed, fields)?,
        GtfsTarget::StopTimes => check_stop_times_conditional(fields)?,
        _ => {}
    }

    Ok(())
}

fn check_stops_conditional(fields: &Fields) -> Result<(), CreateError> {
    let loc_type: Option<LocationType> = fields
        .get("location_type")
        .and_then(|v| v.parse::<i32>().ok())
        .and_then(LocationType::from_i32);

    // Default is StopOrPlatform (0)
    let loc = loc_type.unwrap_or(LocationType::StopOrPlatform);

    match loc {
        LocationType::StopOrPlatform | LocationType::Station => {
            check_required(fields, &["stop_name", "stop_lat", "stop_lon"])?;
        }
        LocationType::EntranceExit | LocationType::GenericNode | LocationType::BoardingArea => {
            check_required(fields, &["parent_station"])?;
        }
    }

    if loc == LocationType::Station && fields.contains_key("parent_station") {
        return Err(CreateError::ForbiddenField {
            field: "parent_station".to_string(),
            reason: "parent_station should not be set for location_type 1 (Station)".to_string(),
        });
    }

    Ok(())
}

fn check_trips_conditional(feed: &GtfsFeed, fields: &Fields) -> Result<(), CreateError> {
    if feed.has_file("shapes.txt") {
        check_required(fields, &["shape_id"])?;
    }
    Ok(())
}

fn check_stop_times_conditional(fields: &Fields) -> Result<(), CreateError> {
    let is_exact = fields
        .get("timepoint")
        .and_then(|v| v.parse::<i32>().ok())
        .and_then(Timepoint::from_i32)
        == Some(Timepoint::Exact);

    if is_exact {
        check_required(fields, &["arrival_time", "departure_time"])?;
    }
    Ok(())
}

fn validate_primary_key(
    idx: &FeedIndex,
    target: GtfsTarget,
    fields: &Fields,
) -> Result<(), CreateError> {
    match target {
        GtfsTarget::Agency => {
            if let Some(&id) = fields.get("agency_id")
                && idx.agency_ids.contains(id)
            {
                return Err(common::pk_err("agency_id", id, "agency.txt").into());
            }
        }
        GtfsTarget::Stops => {
            common::pk_check_simple(fields, "stop_id", &idx.stop_ids, "stops.txt")?;
        }
        GtfsTarget::Routes => {
            common::pk_check_simple(fields, "route_id", &idx.route_ids, "routes.txt")?;
        }
        GtfsTarget::Trips => {
            common::pk_check_simple(fields, "trip_id", &idx.trip_ids, "trips.txt")?;
        }
        GtfsTarget::StopTimes => {
            if let (Some(&tid), Some(&seq_s)) = (fields.get("trip_id"), fields.get("stop_sequence"))
                && let Ok(seq) = seq_s.parse::<u32>()
                && idx.stop_time_pks.contains(&(tid, seq))
            {
                return Err(common::pk_err(
                    "(trip_id, stop_sequence)",
                    &format!("({tid}, {seq})"),
                    "stop_times.txt",
                )
                .into());
            }
        }
        GtfsTarget::Calendar => {
            common::pk_check_simple(fields, "service_id", &idx.service_ids, "calendar.txt")?;
        }
        GtfsTarget::CalendarDates => {
            if let (Some(&sid), Some(&date_s)) = (fields.get("service_id"), fields.get("date"))
                && let Ok(date) = date_s.parse::<GtfsDate>()
                && idx.calendar_date_pks.contains(&(sid, date))
            {
                return Err(common::pk_err(
                    "(service_id, date)",
                    &format!("({sid}, {date_s})"),
                    "calendar_dates.txt",
                )
                .into());
            }
        }
        GtfsTarget::Shapes => {
            if let (Some(&sid), Some(&seq_s)) =
                (fields.get("shape_id"), fields.get("shape_pt_sequence"))
                && let Ok(seq) = seq_s.parse::<u32>()
                && idx.shape_pks.contains(&(sid, seq))
            {
                return Err(common::pk_err(
                    "(shape_id, shape_pt_sequence)",
                    &format!("({sid}, {seq})"),
                    "shapes.txt",
                )
                .into());
            }
        }
        GtfsTarget::Frequencies => {
            if let (Some(&tid), Some(&st_s)) = (fields.get("trip_id"), fields.get("start_time"))
                && let Ok(st) = st_s.parse::<GtfsTime>()
                && idx.frequency_pks.contains(&(tid, st))
            {
                return Err(common::pk_err(
                    "(trip_id, start_time)",
                    &format!("({tid}, {st_s})"),
                    "frequencies.txt",
                )
                .into());
            }
        }
        GtfsTarget::Pathways => {
            common::pk_check_simple(fields, "pathway_id", &idx.pathway_ids, "pathways.txt")?;
        }
        GtfsTarget::Levels => {
            common::pk_check_simple(fields, "level_id", &idx.level_ids, "levels.txt")?;
        }
        GtfsTarget::FeedInfo => {
            if idx.has_feed_info {
                return Err(CreateError::FeedInfoAlreadyExists);
            }
        }
        GtfsTarget::FareAttributes => {
            common::pk_check_simple(fields, "fare_id", &idx.fare_ids, "fare_attributes.txt")?;
        }
        // fare_rules, transfers, translations, attributions: no strict PK
        _ => {}
    }
    Ok(())
}

fn build_record(target: GtfsTarget, fields: &Fields) -> Result<CreatedRecord, CreateError> {
    match target {
        GtfsTarget::Agency => build_agency(fields).map(CreatedRecord::Agency),
        GtfsTarget::Stops => build_stop(fields).map(CreatedRecord::Stop),
        GtfsTarget::Routes => build_route(fields).map(CreatedRecord::Route),
        GtfsTarget::Trips => build_trip(fields).map(CreatedRecord::Trip),
        GtfsTarget::StopTimes => build_stop_time(fields).map(CreatedRecord::StopTime),
        GtfsTarget::Calendar => build_calendar(fields).map(CreatedRecord::Calendar),
        GtfsTarget::CalendarDates => build_calendar_date(fields).map(CreatedRecord::CalendarDate),
        GtfsTarget::Shapes => build_shape(fields).map(CreatedRecord::Shape),
        GtfsTarget::Frequencies => build_frequency(fields).map(CreatedRecord::Frequency),
        GtfsTarget::Transfers => build_transfer(fields).map(CreatedRecord::Transfer),
        GtfsTarget::Pathways => build_pathway(fields).map(CreatedRecord::Pathway),
        GtfsTarget::Levels => build_level(fields).map(CreatedRecord::Level),
        GtfsTarget::FeedInfo => build_feed_info(fields).map(CreatedRecord::FeedInfo),
        GtfsTarget::FareAttributes => {
            build_fare_attribute(fields).map(CreatedRecord::FareAttribute)
        }
        GtfsTarget::FareRules => build_fare_rule(fields).map(CreatedRecord::FareRule),
        GtfsTarget::Translations => build_translation(fields).map(CreatedRecord::Translation),
        GtfsTarget::Attributions => build_attribution(fields).map(CreatedRecord::Attribution),
    }
}

fn req_str(fields: &Fields, name: &str) -> String {
    fields.get(name).unwrap_or(&"").to_string()
}

fn opt_str(fields: &Fields, name: &str) -> Option<String> {
    fields.get(name).map(|v| (*v).to_string())
}

fn req_id<T: From<String>>(fields: &Fields, name: &str) -> T {
    T::from(req_str(fields, name))
}

fn opt_id<T: From<String>>(fields: &Fields, name: &str) -> Option<T> {
    fields.get(name).map(|v| T::from((*v).to_string()))
}

fn req_parse<T: std::str::FromStr>(
    fields: &Fields,
    name: &str,
    expected: &str,
) -> Result<T, CreateError> {
    let val = req_str(fields, name);
    val.parse::<T>()
        .map_err(|_| CreateError::InvalidFieldValue {
            field: name.to_string(),
            value: val,
            expected: expected.to_string(),
        })
}

fn opt_parse<T: std::str::FromStr>(
    fields: &Fields,
    name: &str,
    expected: &str,
) -> Result<Option<T>, CreateError> {
    match fields.get(name) {
        None => Ok(None),
        Some(val) => val
            .parse::<T>()
            .map(Some)
            .map_err(|_| CreateError::InvalidFieldValue {
                field: name.to_string(),
                value: (*val).to_string(),
                expected: expected.to_string(),
            }),
    }
}

fn req_enum<T>(
    fields: &Fields,
    name: &str,
    from_i32: fn(i32) -> Option<T>,
    expected: &str,
) -> Result<T, CreateError> {
    let val = req_str(fields, name);
    let i = val
        .parse::<i32>()
        .map_err(|_| CreateError::InvalidFieldValue {
            field: name.to_string(),
            value: val.clone(),
            expected: expected.to_string(),
        })?;
    from_i32(i).ok_or_else(|| CreateError::InvalidFieldValue {
        field: name.to_string(),
        value: val,
        expected: expected.to_string(),
    })
}

fn opt_enum<T>(
    fields: &Fields,
    name: &str,
    from_i32: fn(i32) -> Option<T>,
    expected: &str,
) -> Result<Option<T>, CreateError> {
    match fields.get(name) {
        None => Ok(None),
        Some(val) => {
            let i = val
                .parse::<i32>()
                .map_err(|_| CreateError::InvalidFieldValue {
                    field: name.to_string(),
                    value: (*val).to_string(),
                    expected: expected.to_string(),
                })?;
            from_i32(i)
                .map(Some)
                .ok_or_else(|| CreateError::InvalidFieldValue {
                    field: name.to_string(),
                    value: (*val).to_string(),
                    expected: expected.to_string(),
                })
        }
    }
}

fn req_bool(fields: &Fields, name: &str) -> bool {
    fields.get(name).is_some_and(|v| *v == "1")
}

#[allow(clippy::unnecessary_wraps)]
fn build_agency(f: &Fields) -> Result<Agency, CreateError> {
    Ok(Agency {
        agency_id: opt_id::<AgencyId>(f, "agency_id"),
        agency_name: req_str(f, "agency_name"),
        agency_url: req_id::<Url>(f, "agency_url"),
        agency_timezone: req_id::<Timezone>(f, "agency_timezone"),
        agency_lang: opt_id::<LanguageCode>(f, "agency_lang"),
        agency_phone: opt_id::<Phone>(f, "agency_phone"),
        agency_fare_url: opt_id::<Url>(f, "agency_fare_url"),
        agency_email: opt_id::<Email>(f, "agency_email"),
    })
}

fn build_stop(f: &Fields) -> Result<Stop, CreateError> {
    Ok(Stop {
        stop_id: req_id::<StopId>(f, "stop_id"),
        stop_code: opt_str(f, "stop_code"),
        stop_name: opt_str(f, "stop_name"),
        tts_stop_name: opt_str(f, "tts_stop_name"),
        stop_desc: opt_str(f, "stop_desc"),
        stop_lat: opt_parse::<f64>(f, "stop_lat", "number")?.map(Latitude),
        stop_lon: opt_parse::<f64>(f, "stop_lon", "number")?.map(Longitude),
        zone_id: opt_str(f, "zone_id"),
        stop_url: opt_id::<Url>(f, "stop_url"),
        location_type: opt_enum(f, "location_type", LocationType::from_i32, "0-4")?,
        parent_station: opt_id::<StopId>(f, "parent_station"),
        stop_timezone: opt_id::<Timezone>(f, "stop_timezone"),
        wheelchair_boarding: opt_enum(
            f,
            "wheelchair_boarding",
            WheelchairAccessible::from_i32,
            "0-2",
        )?,
        level_id: opt_id::<LevelId>(f, "level_id"),
        platform_code: opt_str(f, "platform_code"),
    })
}

fn build_route(f: &Fields) -> Result<Route, CreateError> {
    Ok(Route {
        route_id: req_id::<RouteId>(f, "route_id"),
        agency_id: opt_id::<AgencyId>(f, "agency_id"),
        route_short_name: opt_str(f, "route_short_name"),
        route_long_name: opt_str(f, "route_long_name"),
        route_desc: opt_str(f, "route_desc"),
        route_type: req_enum(f, "route_type", RouteType::from_i32, "route type integer")?,
        route_url: opt_id::<Url>(f, "route_url"),
        route_color: opt_id::<Color>(f, "route_color"),
        route_text_color: opt_id::<Color>(f, "route_text_color"),
        route_sort_order: opt_parse(f, "route_sort_order", "integer")?,
        continuous_pickup: opt_enum(f, "continuous_pickup", ContinuousPickup::from_i32, "0-3")?,
        continuous_drop_off: opt_enum(
            f,
            "continuous_drop_off",
            ContinuousDropOff::from_i32,
            "0-3",
        )?,
        network_id: opt_str(f, "network_id"),
    })
}

fn build_trip(f: &Fields) -> Result<Trip, CreateError> {
    Ok(Trip {
        route_id: req_id::<RouteId>(f, "route_id"),
        service_id: req_id::<ServiceId>(f, "service_id"),
        trip_id: req_id::<TripId>(f, "trip_id"),
        trip_headsign: opt_str(f, "trip_headsign"),
        trip_short_name: opt_str(f, "trip_short_name"),
        direction_id: opt_enum(f, "direction_id", DirectionId::from_i32, "0-1")?,
        block_id: opt_str(f, "block_id"),
        shape_id: opt_id::<ShapeId>(f, "shape_id"),
        wheelchair_accessible: opt_enum(
            f,
            "wheelchair_accessible",
            WheelchairAccessible::from_i32,
            "0-2",
        )?,
        bikes_allowed: opt_enum(f, "bikes_allowed", BikesAllowed::from_i32, "0-2")?,
    })
}

fn build_stop_time(f: &Fields) -> Result<StopTime, CreateError> {
    Ok(StopTime {
        trip_id: req_id::<TripId>(f, "trip_id"),
        arrival_time: opt_parse(f, "arrival_time", "time HH:MM:SS")?,
        departure_time: opt_parse(f, "departure_time", "time HH:MM:SS")?,
        stop_id: req_id::<StopId>(f, "stop_id"),
        stop_sequence: req_parse(f, "stop_sequence", "integer")?,
        stop_headsign: opt_str(f, "stop_headsign"),
        pickup_type: opt_enum(f, "pickup_type", PickupType::from_i32, "0-3")?,
        drop_off_type: opt_enum(f, "drop_off_type", DropOffType::from_i32, "0-3")?,
        continuous_pickup: opt_enum(f, "continuous_pickup", ContinuousPickup::from_i32, "0-3")?,
        continuous_drop_off: opt_enum(
            f,
            "continuous_drop_off",
            ContinuousDropOff::from_i32,
            "0-3",
        )?,
        shape_dist_traveled: opt_parse(f, "shape_dist_traveled", "number")?,
        timepoint: opt_enum(f, "timepoint", Timepoint::from_i32, "0-1")?,
    })
}

fn build_calendar(f: &Fields) -> Result<Calendar, CreateError> {
    Ok(Calendar {
        service_id: req_id::<ServiceId>(f, "service_id"),
        monday: req_bool(f, "monday"),
        tuesday: req_bool(f, "tuesday"),
        wednesday: req_bool(f, "wednesday"),
        thursday: req_bool(f, "thursday"),
        friday: req_bool(f, "friday"),
        saturday: req_bool(f, "saturday"),
        sunday: req_bool(f, "sunday"),
        start_date: req_parse(f, "start_date", "date YYYYMMDD")?,
        end_date: req_parse(f, "end_date", "date YYYYMMDD")?,
    })
}

fn build_calendar_date(f: &Fields) -> Result<CalendarDate, CreateError> {
    Ok(CalendarDate {
        service_id: req_id::<ServiceId>(f, "service_id"),
        date: req_parse(f, "date", "date YYYYMMDD")?,
        exception_type: req_enum(f, "exception_type", ExceptionType::from_i32, "1 or 2")?,
    })
}

fn build_shape(f: &Fields) -> Result<Shape, CreateError> {
    Ok(Shape {
        shape_id: req_id::<ShapeId>(f, "shape_id"),
        shape_pt_lat: Latitude(req_parse(f, "shape_pt_lat", "number")?),
        shape_pt_lon: Longitude(req_parse(f, "shape_pt_lon", "number")?),
        shape_pt_sequence: req_parse(f, "shape_pt_sequence", "integer")?,
        shape_dist_traveled: opt_parse(f, "shape_dist_traveled", "number")?,
    })
}

fn build_frequency(f: &Fields) -> Result<Frequency, CreateError> {
    Ok(Frequency {
        trip_id: req_id::<TripId>(f, "trip_id"),
        start_time: req_parse(f, "start_time", "time HH:MM:SS")?,
        end_time: req_parse(f, "end_time", "time HH:MM:SS")?,
        headway_secs: req_parse(f, "headway_secs", "integer")?,
        exact_times: opt_enum(f, "exact_times", ExactTimes::from_i32, "0-1")?,
    })
}

fn build_transfer(f: &Fields) -> Result<Transfer, CreateError> {
    Ok(Transfer {
        from_stop_id: opt_id::<StopId>(f, "from_stop_id"),
        to_stop_id: opt_id::<StopId>(f, "to_stop_id"),
        from_route_id: opt_id::<RouteId>(f, "from_route_id"),
        to_route_id: opt_id::<RouteId>(f, "to_route_id"),
        from_trip_id: opt_id::<TripId>(f, "from_trip_id"),
        to_trip_id: opt_id::<TripId>(f, "to_trip_id"),
        transfer_type: req_enum(f, "transfer_type", TransferType::from_i32, "0-3")?,
        min_transfer_time: opt_parse(f, "min_transfer_time", "integer")?,
    })
}

fn build_pathway(f: &Fields) -> Result<Pathway, CreateError> {
    Ok(Pathway {
        pathway_id: req_id::<PathwayId>(f, "pathway_id"),
        from_stop_id: req_id::<StopId>(f, "from_stop_id"),
        to_stop_id: req_id::<StopId>(f, "to_stop_id"),
        pathway_mode: req_enum(f, "pathway_mode", PathwayMode::from_i32, "1-7")?,
        is_bidirectional: req_enum(f, "is_bidirectional", IsBidirectional::from_i32, "0-1")?,
        length: opt_parse(f, "length", "number")?,
        traversal_time: opt_parse(f, "traversal_time", "integer")?,
        stair_count: opt_parse(f, "stair_count", "integer")?,
        max_slope: opt_parse(f, "max_slope", "number")?,
        min_width: opt_parse(f, "min_width", "number")?,
        signposted_as: opt_str(f, "signposted_as"),
        reversed_signposted_as: opt_str(f, "reversed_signposted_as"),
    })
}

fn build_level(f: &Fields) -> Result<Level, CreateError> {
    Ok(Level {
        level_id: req_id::<LevelId>(f, "level_id"),
        level_index: req_parse(f, "level_index", "number")?,
        level_name: opt_str(f, "level_name"),
    })
}

fn build_feed_info(f: &Fields) -> Result<FeedInfo, CreateError> {
    Ok(FeedInfo {
        feed_publisher_name: req_str(f, "feed_publisher_name"),
        feed_publisher_url: req_id::<Url>(f, "feed_publisher_url"),
        feed_lang: req_id::<LanguageCode>(f, "feed_lang"),
        default_lang: opt_id::<LanguageCode>(f, "default_lang"),
        feed_start_date: opt_parse(f, "feed_start_date", "date YYYYMMDD")?,
        feed_end_date: opt_parse(f, "feed_end_date", "date YYYYMMDD")?,
        feed_version: opt_str(f, "feed_version"),
        feed_contact_email: opt_id::<Email>(f, "feed_contact_email"),
        feed_contact_url: opt_id::<Url>(f, "feed_contact_url"),
    })
}

fn build_fare_attribute(f: &Fields) -> Result<FareAttribute, CreateError> {
    Ok(FareAttribute {
        fare_id: req_id::<FareId>(f, "fare_id"),
        price: req_parse(f, "price", "number")?,
        currency_type: req_id::<CurrencyCode>(f, "currency_type"),
        payment_method: req_parse(f, "payment_method", "0 or 1")?,
        transfers: opt_parse(f, "transfers", "integer")?,
        agency_id: opt_id::<AgencyId>(f, "agency_id"),
        transfer_duration: opt_parse(f, "transfer_duration", "integer")?,
    })
}

#[allow(clippy::unnecessary_wraps)]
fn build_fare_rule(f: &Fields) -> Result<FareRule, CreateError> {
    Ok(FareRule {
        fare_id: req_id::<FareId>(f, "fare_id"),
        route_id: opt_id::<RouteId>(f, "route_id"),
        origin_id: opt_str(f, "origin_id"),
        destination_id: opt_str(f, "destination_id"),
        contains_id: opt_str(f, "contains_id"),
    })
}

#[allow(clippy::unnecessary_wraps)]
fn build_translation(f: &Fields) -> Result<Translation, CreateError> {
    Ok(Translation {
        table_name: req_str(f, "table_name"),
        field_name: req_str(f, "field_name"),
        language: req_id::<LanguageCode>(f, "language"),
        translation: req_str(f, "translation"),
        record_id: opt_str(f, "record_id"),
        record_sub_id: opt_str(f, "record_sub_id"),
        field_value: opt_str(f, "field_value"),
    })
}

fn build_attribution(f: &Fields) -> Result<Attribution, CreateError> {
    Ok(Attribution {
        attribution_id: opt_str(f, "attribution_id"),
        agency_id: opt_id::<AgencyId>(f, "agency_id"),
        route_id: opt_id::<RouteId>(f, "route_id"),
        trip_id: opt_id::<TripId>(f, "trip_id"),
        organization_name: req_str(f, "organization_name"),
        is_producer: opt_parse(f, "is_producer", "0 or 1")?,
        is_operator: opt_parse(f, "is_operator", "0 or 1")?,
        is_authority: opt_parse(f, "is_authority", "0 or 1")?,
        attribution_url: opt_id::<Url>(f, "attribution_url"),
        attribution_email: opt_id::<Email>(f, "attribution_email"),
        attribution_phone: opt_id::<Phone>(f, "attribution_phone"),
    })
}
