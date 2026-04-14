//! Field-level mutation functions for GTFS records.

use crate::crud::common::CrudError;
use crate::crud::query::Filterable;
use crate::models::{
    Agency, AgencyId, Attribution, BikesAllowed, Calendar, CalendarDate, Color, ContinuousDropOff,
    ContinuousPickup, CurrencyCode, DirectionId, DropOffType, Email, ExactTimes, ExceptionType,
    FareAttribute, FareId, FareRule, FeedInfo, Frequency, IsBidirectional, LanguageCode, Latitude,
    Level, LevelId, LocationType, Longitude, Pathway, PathwayId, PathwayMode, Phone, PickupType,
    Route, RouteId, RouteType, ServiceId, Shape, ShapeId, Stop, StopId, StopTime, Timepoint,
    Timezone, Transfer, TransferType, Translation, Trip, TripId, Url, WheelchairAccessible,
};

/// Uniform field-setter API: each GTFS record type forwards to its
/// corresponding `set_*_field` free function. Enables generic dispatch over
/// GTFS types in `update.rs`.
pub trait FieldSetter {
    /// Parses `value` and assigns it to `field` on `self`.
    ///
    /// # Errors
    ///
    /// Returns [`CrudError`] on parse failure or unknown field.
    fn set_field(&mut self, field: &str, value: &str) -> Result<(), CrudError>;
}

macro_rules! impl_field_setter {
    ($($ty:ident => $setter:ident),* $(,)?) => {
        $(
            impl FieldSetter for $ty {
                fn set_field(&mut self, field: &str, value: &str) -> Result<(), CrudError> {
                    $setter(self, field, value)
                }
            }
        )*
    };
}

impl_field_setter! {
    Agency => set_agency_field,
    Stop => set_stop_field,
    Route => set_route_field,
    Trip => set_trip_field,
    StopTime => set_stop_time_field,
    Calendar => set_calendar_field,
    CalendarDate => set_calendar_date_field,
    Shape => set_shape_field,
    Frequency => set_frequency_field,
    Transfer => set_transfer_field,
    Pathway => set_pathway_field,
    Level => set_level_field,
    FeedInfo => set_feed_info_field,
    FareAttribute => set_fare_attribute_field,
    FareRule => set_fare_rule_field,
    Translation => set_translation_field,
    Attribution => set_attribution_field,
}

fn parse_value<T: std::str::FromStr>(
    value: &str,
    field: &str,
    expected: &str,
) -> Result<T, CrudError> {
    value
        .parse::<T>()
        .map_err(|_| CrudError::InvalidFieldValue {
            field: field.to_string(),
            value: value.to_string(),
            expected: expected.to_string(),
        })
}

fn parse_enum<T>(
    value: &str,
    field: &str,
    from_i32: fn(i32) -> Option<T>,
    expected: &str,
) -> Result<T, CrudError> {
    let i = parse_value::<i32>(value, field, expected)?;
    from_i32(i).ok_or_else(|| CrudError::InvalidFieldValue {
        field: field.to_string(),
        value: value.to_string(),
        expected: expected.to_string(),
    })
}

fn unknown(field: &str, valid: &[&str]) -> CrudError {
    CrudError::UnknownField {
        field: field.to_string(),
        valid: valid.join(", "),
    }
}

/// Generates a `pub fn $name(record, field, value)` that matches on `field`
/// and assigns an expression to `record.<field>`. The assignment expression
/// has access to the `field` and `value` idents named in the invocation.
macro_rules! define_setter {
    (
        $name:ident ( $record:ident : $ty:ty, $field:ident, $value:ident ) {
            $( $col:ident => $assign:expr ),* $(,)?
        }
    ) => {
        /// # Errors
        ///
        /// Returns [`CrudError`] on unknown field or parse failure.
        pub fn $name($record: &mut $ty, $field: &str, $value: &str) -> Result<(), CrudError> {
            match $field {
                $( stringify!($col) => $record.$col = $assign, )*
                _ => return Err(unknown($field, <$ty>::valid_fields())),
            }
            Ok(())
        }
    };
}

define_setter!(set_agency_field(agency: Agency, field, value) {
    agency_id => Some(AgencyId::from(value.to_string())),
    agency_name => value.to_string(),
    agency_url => Url::from(value.to_string()),
    agency_timezone => Timezone::from(value.to_string()),
    agency_lang => Some(LanguageCode::from(value.to_string())),
    agency_phone => Some(Phone::from(value.to_string())),
    agency_fare_url => Some(Url::from(value.to_string())),
    agency_email => Some(Email::from(value.to_string())),
});

define_setter!(set_stop_field(stop: Stop, field, value) {
    stop_id => StopId::from(value.to_string()),
    stop_code => Some(value.to_string()),
    stop_name => Some(value.to_string()),
    tts_stop_name => Some(value.to_string()),
    stop_desc => Some(value.to_string()),
    stop_lat => Some(Latitude(parse_value(value, field, "number")?)),
    stop_lon => Some(Longitude(parse_value(value, field, "number")?)),
    zone_id => Some(value.to_string()),
    stop_url => Some(Url::from(value.to_string())),
    location_type => Some(parse_enum(value, field, LocationType::from_i32, "0-4")?),
    parent_station => Some(StopId::from(value.to_string())),
    stop_timezone => Some(Timezone::from(value.to_string())),
    wheelchair_boarding => Some(parse_enum(value, field, WheelchairAccessible::from_i32, "0-2")?),
    level_id => Some(LevelId::from(value.to_string())),
    platform_code => Some(value.to_string()),
});

define_setter!(set_route_field(route: Route, field, value) {
    route_id => RouteId::from(value.to_string()),
    agency_id => Some(AgencyId::from(value.to_string())),
    route_short_name => Some(value.to_string()),
    route_long_name => Some(value.to_string()),
    route_desc => Some(value.to_string()),
    route_type => parse_enum(value, field, RouteType::from_i32, "route type integer")?,
    route_url => Some(Url::from(value.to_string())),
    route_color => Some(Color::from(value.to_string())),
    route_text_color => Some(Color::from(value.to_string())),
    route_sort_order => Some(parse_value(value, field, "integer")?),
    continuous_pickup => Some(parse_enum(value, field, ContinuousPickup::from_i32, "0-3")?),
    continuous_drop_off => Some(parse_enum(value, field, ContinuousDropOff::from_i32, "0-3")?),
    network_id => Some(value.to_string()),
});

define_setter!(set_trip_field(trip: Trip, field, value) {
    route_id => RouteId::from(value.to_string()),
    service_id => ServiceId::from(value.to_string()),
    trip_id => TripId::from(value.to_string()),
    trip_headsign => Some(value.to_string()),
    trip_short_name => Some(value.to_string()),
    direction_id => Some(parse_enum(value, field, DirectionId::from_i32, "0-1")?),
    block_id => Some(value.to_string()),
    shape_id => Some(ShapeId::from(value.to_string())),
    wheelchair_accessible => Some(parse_enum(value, field, WheelchairAccessible::from_i32, "0-2")?),
    bikes_allowed => Some(parse_enum(value, field, BikesAllowed::from_i32, "0-2")?),
});

define_setter!(set_stop_time_field(st: StopTime, field, value) {
    trip_id => TripId::from(value.to_string()),
    arrival_time => Some(parse_value(value, field, "time HH:MM:SS")?),
    departure_time => Some(parse_value(value, field, "time HH:MM:SS")?),
    stop_id => StopId::from(value.to_string()),
    stop_sequence => parse_value(value, field, "integer")?,
    stop_headsign => Some(value.to_string()),
    pickup_type => Some(parse_enum(value, field, PickupType::from_i32, "0-3")?),
    drop_off_type => Some(parse_enum(value, field, DropOffType::from_i32, "0-3")?),
    continuous_pickup => Some(parse_enum(value, field, ContinuousPickup::from_i32, "0-3")?),
    continuous_drop_off => Some(parse_enum(value, field, ContinuousDropOff::from_i32, "0-3")?),
    shape_dist_traveled => Some(parse_value(value, field, "number")?),
    timepoint => Some(parse_enum(value, field, Timepoint::from_i32, "0-1")?),
});

define_setter!(set_calendar_field(cal: Calendar, field, value) {
    service_id => ServiceId::from(value.to_string()),
    monday => value == "1",
    tuesday => value == "1",
    wednesday => value == "1",
    thursday => value == "1",
    friday => value == "1",
    saturday => value == "1",
    sunday => value == "1",
    start_date => parse_value(value, field, "date YYYYMMDD")?,
    end_date => parse_value(value, field, "date YYYYMMDD")?,
});

define_setter!(set_calendar_date_field(cd: CalendarDate, field, value) {
    service_id => ServiceId::from(value.to_string()),
    date => parse_value(value, field, "date YYYYMMDD")?,
    exception_type => parse_enum(value, field, ExceptionType::from_i32, "1 or 2")?,
});

define_setter!(set_shape_field(shape: Shape, field, value) {
    shape_id => ShapeId::from(value.to_string()),
    shape_pt_lat => Latitude(parse_value(value, field, "number")?),
    shape_pt_lon => Longitude(parse_value(value, field, "number")?),
    shape_pt_sequence => parse_value(value, field, "integer")?,
    shape_dist_traveled => Some(parse_value(value, field, "number")?),
});

define_setter!(set_frequency_field(freq: Frequency, field, value) {
    trip_id => TripId::from(value.to_string()),
    start_time => parse_value(value, field, "time HH:MM:SS")?,
    end_time => parse_value(value, field, "time HH:MM:SS")?,
    headway_secs => parse_value(value, field, "integer")?,
    exact_times => Some(parse_enum(value, field, ExactTimes::from_i32, "0-1")?),
});

define_setter!(set_transfer_field(tr: Transfer, field, value) {
    from_stop_id => Some(StopId::from(value.to_string())),
    to_stop_id => Some(StopId::from(value.to_string())),
    from_route_id => Some(RouteId::from(value.to_string())),
    to_route_id => Some(RouteId::from(value.to_string())),
    from_trip_id => Some(TripId::from(value.to_string())),
    to_trip_id => Some(TripId::from(value.to_string())),
    transfer_type => parse_enum(value, field, TransferType::from_i32, "0-3")?,
    min_transfer_time => Some(parse_value(value, field, "integer")?),
});

define_setter!(set_pathway_field(pw: Pathway, field, value) {
    pathway_id => PathwayId::from(value.to_string()),
    from_stop_id => StopId::from(value.to_string()),
    to_stop_id => StopId::from(value.to_string()),
    pathway_mode => parse_enum(value, field, PathwayMode::from_i32, "1-7")?,
    is_bidirectional => parse_enum(value, field, IsBidirectional::from_i32, "0-1")?,
    length => Some(parse_value(value, field, "number")?),
    traversal_time => Some(parse_value(value, field, "integer")?),
    stair_count => Some(parse_value(value, field, "integer")?),
    max_slope => Some(parse_value(value, field, "number")?),
    min_width => Some(parse_value(value, field, "number")?),
    signposted_as => Some(value.to_string()),
    reversed_signposted_as => Some(value.to_string()),
});

define_setter!(set_level_field(level: Level, field, value) {
    level_id => LevelId::from(value.to_string()),
    level_index => parse_value(value, field, "number")?,
    level_name => Some(value.to_string()),
});

define_setter!(set_feed_info_field(fi: FeedInfo, field, value) {
    feed_publisher_name => value.to_string(),
    feed_publisher_url => Url::from(value.to_string()),
    feed_lang => LanguageCode::from(value.to_string()),
    default_lang => Some(LanguageCode::from(value.to_string())),
    feed_start_date => Some(parse_value(value, field, "date YYYYMMDD")?),
    feed_end_date => Some(parse_value(value, field, "date YYYYMMDD")?),
    feed_version => Some(value.to_string()),
    feed_contact_email => Some(Email::from(value.to_string())),
    feed_contact_url => Some(Url::from(value.to_string())),
});

define_setter!(set_fare_attribute_field(fa: FareAttribute, field, value) {
    fare_id => FareId::from(value.to_string()),
    price => parse_value(value, field, "number")?,
    currency_type => CurrencyCode::from(value.to_string()),
    payment_method => parse_value(value, field, "0 or 1")?,
    transfers => Some(parse_value(value, field, "integer")?),
    agency_id => Some(AgencyId::from(value.to_string())),
    transfer_duration => Some(parse_value(value, field, "integer")?),
});

define_setter!(set_fare_rule_field(fr: FareRule, field, value) {
    fare_id => FareId::from(value.to_string()),
    route_id => Some(RouteId::from(value.to_string())),
    origin_id => Some(value.to_string()),
    destination_id => Some(value.to_string()),
    contains_id => Some(value.to_string()),
});

define_setter!(set_translation_field(tr: Translation, field, value) {
    table_name => value.to_string(),
    field_name => value.to_string(),
    language => LanguageCode::from(value.to_string()),
    translation => value.to_string(),
    record_id => Some(value.to_string()),
    record_sub_id => Some(value.to_string()),
    field_value => Some(value.to_string()),
});

define_setter!(set_attribution_field(attr: Attribution, field, value) {
    attribution_id => Some(value.to_string()),
    agency_id => Some(AgencyId::from(value.to_string())),
    route_id => Some(RouteId::from(value.to_string())),
    trip_id => Some(TripId::from(value.to_string())),
    organization_name => value.to_string(),
    is_producer => Some(parse_value(value, field, "0 or 1")?),
    is_operator => Some(parse_value(value, field, "0 or 1")?),
    is_authority => Some(parse_value(value, field, "0 or 1")?),
    attribution_url => Some(Url::from(value.to_string())),
    attribution_email => Some(Email::from(value.to_string())),
    attribution_phone => Some(Phone::from(value.to_string())),
});
