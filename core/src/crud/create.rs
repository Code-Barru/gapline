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
    /// The original field assignments, kept for confirmation-prompt display.
    pub assignments: Vec<FieldAssignment>,
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
        assignments,
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
        GtfsTarget::Agency => Ok(CreatedRecord::Agency(build_agency(fields))),
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
        GtfsTarget::FareRules => Ok(CreatedRecord::FareRule(build_fare_rule(fields))),
        GtfsTarget::Translations => Ok(CreatedRecord::Translation(build_translation(fields))),
        GtfsTarget::Attributions => build_attribution(fields).map(CreatedRecord::Attribution),
    }
}

/// Generates `fn $fn(f: &Fields) -> Result<$ty, CreateError>` via incremental
/// TT munching. Each field line must end with a trailing comma. Field names
/// are derived from the struct field identifier via `stringify!`.
///
/// Macros cannot expand to a single struct field in Rust, so we accumulate
/// the full struct-literal body in `[$($acc:tt)*]` and emit it in the
/// terminal arm.
macro_rules! build_record {
    ($fn:ident -> $ty:ident { $($body:tt)* }) => {
        build_record!(@walk fallible, $fn, $ty, f, [] $($body)*);
    };

    (@plain $fn:ident -> $ty:ident { $($body:tt)* }) => {
        build_record!(@walk plain, $fn, $ty, f, [] $($body)*);
    };

    // --- terminal arms ---
    (@walk fallible, $fn:ident, $ty:ident, $f:ident, [$($acc:tt)*]) => {
        fn $fn($f: &Fields) -> Result<$ty, CreateError> {
            Ok($ty { $($acc)* })
        }
    };
    (@walk plain, $fn:ident, $ty:ident, $f:ident, [$($acc:tt)*]) => {
        fn $fn($f: &Fields) -> $ty {
            $ty { $($acc)* }
        }
    };

    // --- infallible kinds (valid in both modes) ---
    (@walk $mode:ident, $fn:ident, $ty:ident, $f:ident, [$($acc:tt)*]
        $name:ident : req_str, $($rest:tt)*) => {
        build_record!(@walk $mode, $fn, $ty, $f,
            [$($acc)* $name: req_str($f, stringify!($name)),] $($rest)*);
    };
    (@walk $mode:ident, $fn:ident, $ty:ident, $f:ident, [$($acc:tt)*]
        $name:ident : opt_str, $($rest:tt)*) => {
        build_record!(@walk $mode, $fn, $ty, $f,
            [$($acc)* $name: opt_str($f, stringify!($name)),] $($rest)*);
    };
    (@walk $mode:ident, $fn:ident, $ty:ident, $f:ident, [$($acc:tt)*]
        $name:ident : req_bool, $($rest:tt)*) => {
        build_record!(@walk $mode, $fn, $ty, $f,
            [$($acc)* $name: req_bool($f, stringify!($name)),] $($rest)*);
    };
    (@walk $mode:ident, $fn:ident, $ty:ident, $f:ident, [$($acc:tt)*]
        $name:ident : req_id<$t:ty>, $($rest:tt)*) => {
        build_record!(@walk $mode, $fn, $ty, $f,
            [$($acc)* $name: req_id::<$t>($f, stringify!($name)),] $($rest)*);
    };
    (@walk $mode:ident, $fn:ident, $ty:ident, $f:ident, [$($acc:tt)*]
        $name:ident : opt_id<$t:ty>, $($rest:tt)*) => {
        build_record!(@walk $mode, $fn, $ty, $f,
            [$($acc)* $name: opt_id::<$t>($f, stringify!($name)),] $($rest)*);
    };

    // --- fallible kinds (only valid in fallible mode) ---
    (@walk fallible, $fn:ident, $ty:ident, $f:ident, [$($acc:tt)*]
        $name:ident : req_parse($expected:literal), $($rest:tt)*) => {
        build_record!(@walk fallible, $fn, $ty, $f,
            [$($acc)* $name: req_parse($f, stringify!($name), $expected)?,] $($rest)*);
    };
    (@walk fallible, $fn:ident, $ty:ident, $f:ident, [$($acc:tt)*]
        $name:ident : opt_parse($expected:literal), $($rest:tt)*) => {
        build_record!(@walk fallible, $fn, $ty, $f,
            [$($acc)* $name: opt_parse($f, stringify!($name), $expected)?,] $($rest)*);
    };
    (@walk fallible, $fn:ident, $ty:ident, $f:ident, [$($acc:tt)*]
        $name:ident : req_parse<$t:ty>($expected:literal) as $wrap:path, $($rest:tt)*) => {
        build_record!(@walk fallible, $fn, $ty, $f,
            [$($acc)* $name: $wrap(req_parse::<$t>($f, stringify!($name), $expected)?),]
            $($rest)*);
    };
    (@walk fallible, $fn:ident, $ty:ident, $f:ident, [$($acc:tt)*]
        $name:ident : opt_parse<$t:ty>($expected:literal) as $wrap:path, $($rest:tt)*) => {
        build_record!(@walk fallible, $fn, $ty, $f,
            [$($acc)* $name: opt_parse::<$t>($f, stringify!($name), $expected)?.map($wrap),]
            $($rest)*);
    };
    (@walk fallible, $fn:ident, $ty:ident, $f:ident, [$($acc:tt)*]
        $name:ident : req_enum($from:path, $expected:literal), $($rest:tt)*) => {
        build_record!(@walk fallible, $fn, $ty, $f,
            [$($acc)* $name: req_enum($f, stringify!($name), $from, $expected)?,] $($rest)*);
    };
    (@walk fallible, $fn:ident, $ty:ident, $f:ident, [$($acc:tt)*]
        $name:ident : opt_enum($from:path, $expected:literal), $($rest:tt)*) => {
        build_record!(@walk fallible, $fn, $ty, $f,
            [$($acc)* $name: opt_enum($f, stringify!($name), $from, $expected)?,] $($rest)*);
    };
}

/// Infallible variant — delegates to `build_record!(@plain ...)`.
macro_rules! build_record_plain {
    ($fn:ident -> $ty:ident { $($body:tt)* }) => {
        build_record!(@plain $fn -> $ty { $($body)* });
    };
}

fn req_str(fields: &Fields, name: &str) -> String {
    fields.get(name).copied().unwrap_or("").to_owned()
}

fn opt_str(fields: &Fields, name: &str) -> Option<String> {
    fields.get(name).copied().map(str::to_owned)
}

fn req_id<T: From<String>>(fields: &Fields, name: &str) -> T {
    T::from(req_str(fields, name))
}

fn opt_id<T: From<String>>(fields: &Fields, name: &str) -> Option<T> {
    fields.get(name).copied().map(|v| T::from(v.to_owned()))
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

build_record_plain! {
    build_agency -> Agency {
        agency_id: opt_id<AgencyId>,
        agency_name: req_str,
        agency_url: req_id<Url>,
        agency_timezone: req_id<Timezone>,
        agency_lang: opt_id<LanguageCode>,
        agency_phone: opt_id<Phone>,
        agency_fare_url: opt_id<Url>,
        agency_email: opt_id<Email>,
    }
}

build_record! {
    build_stop -> Stop {
        stop_id: req_id<StopId>,
        stop_code: opt_str,
        stop_name: opt_str,
        tts_stop_name: opt_str,
        stop_desc: opt_str,
        stop_lat: opt_parse<f64>("number") as Latitude,
        stop_lon: opt_parse<f64>("number") as Longitude,
        zone_id: opt_str,
        stop_url: opt_id<Url>,
        location_type: opt_enum(LocationType::from_i32, "0-4"),
        parent_station: opt_id<StopId>,
        stop_timezone: opt_id<Timezone>,
        wheelchair_boarding: opt_enum(WheelchairAccessible::from_i32, "0-2"),
        level_id: opt_id<LevelId>,
        platform_code: opt_str,
    }
}

build_record! {
    build_route -> Route {
        route_id: req_id<RouteId>,
        agency_id: opt_id<AgencyId>,
        route_short_name: opt_str,
        route_long_name: opt_str,
        route_desc: opt_str,
        route_type: req_enum(RouteType::from_i32, "route type integer"),
        route_url: opt_id<Url>,
        route_color: opt_id<Color>,
        route_text_color: opt_id<Color>,
        route_sort_order: opt_parse("integer"),
        continuous_pickup: opt_enum(ContinuousPickup::from_i32, "0-3"),
        continuous_drop_off: opt_enum(ContinuousDropOff::from_i32, "0-3"),
        network_id: opt_str,
    }
}

build_record! {
    build_trip -> Trip {
        route_id: req_id<RouteId>,
        service_id: req_id<ServiceId>,
        trip_id: req_id<TripId>,
        trip_headsign: opt_str,
        trip_short_name: opt_str,
        direction_id: opt_enum(DirectionId::from_i32, "0-1"),
        block_id: opt_str,
        shape_id: opt_id<ShapeId>,
        wheelchair_accessible: opt_enum(WheelchairAccessible::from_i32, "0-2"),
        bikes_allowed: opt_enum(BikesAllowed::from_i32, "0-2"),
    }
}

build_record! {
    build_stop_time -> StopTime {
        trip_id: req_id<TripId>,
        arrival_time: opt_parse("time HH:MM:SS"),
        departure_time: opt_parse("time HH:MM:SS"),
        stop_id: req_id<StopId>,
        stop_sequence: req_parse("integer"),
        stop_headsign: opt_str,
        pickup_type: opt_enum(PickupType::from_i32, "0-3"),
        drop_off_type: opt_enum(DropOffType::from_i32, "0-3"),
        continuous_pickup: opt_enum(ContinuousPickup::from_i32, "0-3"),
        continuous_drop_off: opt_enum(ContinuousDropOff::from_i32, "0-3"),
        shape_dist_traveled: opt_parse("number"),
        timepoint: opt_enum(Timepoint::from_i32, "0-1"),
    }
}

build_record! {
    build_calendar -> Calendar {
        service_id: req_id<ServiceId>,
        monday: req_bool,
        tuesday: req_bool,
        wednesday: req_bool,
        thursday: req_bool,
        friday: req_bool,
        saturday: req_bool,
        sunday: req_bool,
        start_date: req_parse("date YYYYMMDD"),
        end_date: req_parse("date YYYYMMDD"),
    }
}

build_record! {
    build_calendar_date -> CalendarDate {
        service_id: req_id<ServiceId>,
        date: req_parse("date YYYYMMDD"),
        exception_type: req_enum(ExceptionType::from_i32, "1 or 2"),
    }
}

build_record! {
    build_shape -> Shape {
        shape_id: req_id<ShapeId>,
        shape_pt_lat: req_parse<f64>("number") as Latitude,
        shape_pt_lon: req_parse<f64>("number") as Longitude,
        shape_pt_sequence: req_parse("integer"),
        shape_dist_traveled: opt_parse("number"),
    }
}

build_record! {
    build_frequency -> Frequency {
        trip_id: req_id<TripId>,
        start_time: req_parse("time HH:MM:SS"),
        end_time: req_parse("time HH:MM:SS"),
        headway_secs: req_parse("integer"),
        exact_times: opt_enum(ExactTimes::from_i32, "0-1"),
    }
}

build_record! {
    build_transfer -> Transfer {
        from_stop_id: opt_id<StopId>,
        to_stop_id: opt_id<StopId>,
        from_route_id: opt_id<RouteId>,
        to_route_id: opt_id<RouteId>,
        from_trip_id: opt_id<TripId>,
        to_trip_id: opt_id<TripId>,
        transfer_type: req_enum(TransferType::from_i32, "0-3"),
        min_transfer_time: opt_parse("integer"),
    }
}

build_record! {
    build_pathway -> Pathway {
        pathway_id: req_id<PathwayId>,
        from_stop_id: req_id<StopId>,
        to_stop_id: req_id<StopId>,
        pathway_mode: req_enum(PathwayMode::from_i32, "1-7"),
        is_bidirectional: req_enum(IsBidirectional::from_i32, "0-1"),
        length: opt_parse("number"),
        traversal_time: opt_parse("integer"),
        stair_count: opt_parse("integer"),
        max_slope: opt_parse("number"),
        min_width: opt_parse("number"),
        signposted_as: opt_str,
        reversed_signposted_as: opt_str,
    }
}

build_record! {
    build_level -> Level {
        level_id: req_id<LevelId>,
        level_index: req_parse("number"),
        level_name: opt_str,
    }
}

build_record! {
    build_feed_info -> FeedInfo {
        feed_publisher_name: req_str,
        feed_publisher_url: req_id<Url>,
        feed_lang: req_id<LanguageCode>,
        default_lang: opt_id<LanguageCode>,
        feed_start_date: opt_parse("date YYYYMMDD"),
        feed_end_date: opt_parse("date YYYYMMDD"),
        feed_version: opt_str,
        feed_contact_email: opt_id<Email>,
        feed_contact_url: opt_id<Url>,
    }
}

build_record! {
    build_fare_attribute -> FareAttribute {
        fare_id: req_id<FareId>,
        price: req_parse("number"),
        currency_type: req_id<CurrencyCode>,
        payment_method: req_parse("0 or 1"),
        transfers: opt_parse("integer"),
        agency_id: opt_id<AgencyId>,
        transfer_duration: opt_parse("integer"),
    }
}

build_record_plain! {
    build_fare_rule -> FareRule {
        fare_id: req_id<FareId>,
        route_id: opt_id<RouteId>,
        origin_id: opt_str,
        destination_id: opt_str,
        contains_id: opt_str,
    }
}

build_record_plain! {
    build_translation -> Translation {
        table_name: req_str,
        field_name: req_str,
        language: req_id<LanguageCode>,
        translation: req_str,
        record_id: opt_str,
        record_sub_id: opt_str,
        field_value: opt_str,
    }
}

build_record! {
    build_attribution -> Attribution {
        attribution_id: opt_str,
        agency_id: opt_id<AgencyId>,
        route_id: opt_id<RouteId>,
        trip_id: opt_id<TripId>,
        organization_name: req_str,
        is_producer: opt_parse("0 or 1"),
        is_operator: opt_parse("0 or 1"),
        is_authority: opt_parse("0 or 1"),
        attribution_url: opt_id<Url>,
        attribution_email: opt_id<Email>,
        attribution_phone: opt_id<Phone>,
    }
}
