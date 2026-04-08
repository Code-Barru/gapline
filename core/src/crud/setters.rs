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

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_agency_field(agency: &mut Agency, field: &str, value: &str) -> Result<(), CrudError> {
    match field {
        "agency_id" => agency.agency_id = Some(AgencyId::from(value.to_string())),
        "agency_name" => agency.agency_name = value.to_string(),
        "agency_url" => agency.agency_url = Url::from(value.to_string()),
        "agency_timezone" => agency.agency_timezone = Timezone::from(value.to_string()),
        "agency_lang" => agency.agency_lang = Some(LanguageCode::from(value.to_string())),
        "agency_phone" => agency.agency_phone = Some(Phone::from(value.to_string())),
        "agency_fare_url" => agency.agency_fare_url = Some(Url::from(value.to_string())),
        "agency_email" => agency.agency_email = Some(Email::from(value.to_string())),
        _ => return Err(unknown(field, Agency::valid_fields())),
    }
    Ok(())
}

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_stop_field(stop: &mut Stop, field: &str, value: &str) -> Result<(), CrudError> {
    match field {
        "stop_id" => stop.stop_id = StopId::from(value.to_string()),
        "stop_code" => stop.stop_code = Some(value.to_string()),
        "stop_name" => stop.stop_name = Some(value.to_string()),
        "tts_stop_name" => stop.tts_stop_name = Some(value.to_string()),
        "stop_desc" => stop.stop_desc = Some(value.to_string()),
        "stop_lat" => stop.stop_lat = Some(Latitude(parse_value(value, field, "number")?)),
        "stop_lon" => stop.stop_lon = Some(Longitude(parse_value(value, field, "number")?)),
        "zone_id" => stop.zone_id = Some(value.to_string()),
        "stop_url" => stop.stop_url = Some(Url::from(value.to_string())),
        "location_type" => {
            stop.location_type = Some(parse_enum(value, field, LocationType::from_i32, "0-4")?);
        }
        "parent_station" => stop.parent_station = Some(StopId::from(value.to_string())),
        "stop_timezone" => stop.stop_timezone = Some(Timezone::from(value.to_string())),
        "wheelchair_boarding" => {
            stop.wheelchair_boarding = Some(parse_enum(
                value,
                field,
                WheelchairAccessible::from_i32,
                "0-2",
            )?);
        }
        "level_id" => stop.level_id = Some(LevelId::from(value.to_string())),
        "platform_code" => stop.platform_code = Some(value.to_string()),
        _ => return Err(unknown(field, Stop::valid_fields())),
    }
    Ok(())
}

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_route_field(route: &mut Route, field: &str, value: &str) -> Result<(), CrudError> {
    match field {
        "route_id" => route.route_id = RouteId::from(value.to_string()),
        "agency_id" => route.agency_id = Some(AgencyId::from(value.to_string())),
        "route_short_name" => route.route_short_name = Some(value.to_string()),
        "route_long_name" => route.route_long_name = Some(value.to_string()),
        "route_desc" => route.route_desc = Some(value.to_string()),
        "route_type" => {
            route.route_type = parse_enum(value, field, RouteType::from_i32, "route type integer")?;
        }
        "route_url" => route.route_url = Some(Url::from(value.to_string())),
        "route_color" => route.route_color = Some(Color::from(value.to_string())),
        "route_text_color" => route.route_text_color = Some(Color::from(value.to_string())),
        "route_sort_order" => route.route_sort_order = Some(parse_value(value, field, "integer")?),
        "continuous_pickup" => {
            route.continuous_pickup =
                Some(parse_enum(value, field, ContinuousPickup::from_i32, "0-3")?);
        }
        "continuous_drop_off" => {
            route.continuous_drop_off = Some(parse_enum(
                value,
                field,
                ContinuousDropOff::from_i32,
                "0-3",
            )?);
        }
        "network_id" => route.network_id = Some(value.to_string()),
        _ => return Err(unknown(field, Route::valid_fields())),
    }
    Ok(())
}

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_trip_field(trip: &mut Trip, field: &str, value: &str) -> Result<(), CrudError> {
    match field {
        "route_id" => trip.route_id = RouteId::from(value.to_string()),
        "service_id" => trip.service_id = ServiceId::from(value.to_string()),
        "trip_id" => trip.trip_id = TripId::from(value.to_string()),
        "trip_headsign" => trip.trip_headsign = Some(value.to_string()),
        "trip_short_name" => trip.trip_short_name = Some(value.to_string()),
        "direction_id" => {
            trip.direction_id = Some(parse_enum(value, field, DirectionId::from_i32, "0-1")?);
        }
        "block_id" => trip.block_id = Some(value.to_string()),
        "shape_id" => trip.shape_id = Some(ShapeId::from(value.to_string())),
        "wheelchair_accessible" => {
            trip.wheelchair_accessible = Some(parse_enum(
                value,
                field,
                WheelchairAccessible::from_i32,
                "0-2",
            )?);
        }
        "bikes_allowed" => {
            trip.bikes_allowed = Some(parse_enum(value, field, BikesAllowed::from_i32, "0-2")?);
        }
        _ => return Err(unknown(field, Trip::valid_fields())),
    }
    Ok(())
}

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_stop_time_field(st: &mut StopTime, field: &str, value: &str) -> Result<(), CrudError> {
    match field {
        "trip_id" => st.trip_id = TripId::from(value.to_string()),
        "arrival_time" => st.arrival_time = Some(parse_value(value, field, "time HH:MM:SS")?),
        "departure_time" => st.departure_time = Some(parse_value(value, field, "time HH:MM:SS")?),
        "stop_id" => st.stop_id = StopId::from(value.to_string()),
        "stop_sequence" => st.stop_sequence = parse_value(value, field, "integer")?,
        "stop_headsign" => st.stop_headsign = Some(value.to_string()),
        "pickup_type" => {
            st.pickup_type = Some(parse_enum(value, field, PickupType::from_i32, "0-3")?);
        }
        "drop_off_type" => {
            st.drop_off_type = Some(parse_enum(value, field, DropOffType::from_i32, "0-3")?);
        }
        "continuous_pickup" => {
            st.continuous_pickup =
                Some(parse_enum(value, field, ContinuousPickup::from_i32, "0-3")?);
        }
        "continuous_drop_off" => {
            st.continuous_drop_off = Some(parse_enum(
                value,
                field,
                ContinuousDropOff::from_i32,
                "0-3",
            )?);
        }
        "shape_dist_traveled" => {
            st.shape_dist_traveled = Some(parse_value(value, field, "number")?);
        }
        "timepoint" => {
            st.timepoint = Some(parse_enum(value, field, Timepoint::from_i32, "0-1")?);
        }
        _ => return Err(unknown(field, StopTime::valid_fields())),
    }
    Ok(())
}

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_calendar_field(cal: &mut Calendar, field: &str, value: &str) -> Result<(), CrudError> {
    match field {
        "service_id" => cal.service_id = ServiceId::from(value.to_string()),
        "monday" => cal.monday = value == "1",
        "tuesday" => cal.tuesday = value == "1",
        "wednesday" => cal.wednesday = value == "1",
        "thursday" => cal.thursday = value == "1",
        "friday" => cal.friday = value == "1",
        "saturday" => cal.saturday = value == "1",
        "sunday" => cal.sunday = value == "1",
        "start_date" => cal.start_date = parse_value(value, field, "date YYYYMMDD")?,
        "end_date" => cal.end_date = parse_value(value, field, "date YYYYMMDD")?,
        _ => return Err(unknown(field, Calendar::valid_fields())),
    }
    Ok(())
}

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_calendar_date_field(
    cd: &mut CalendarDate,
    field: &str,
    value: &str,
) -> Result<(), CrudError> {
    match field {
        "service_id" => cd.service_id = ServiceId::from(value.to_string()),
        "date" => cd.date = parse_value(value, field, "date YYYYMMDD")?,
        "exception_type" => {
            cd.exception_type = parse_enum(value, field, ExceptionType::from_i32, "1 or 2")?;
        }
        _ => return Err(unknown(field, CalendarDate::valid_fields())),
    }
    Ok(())
}

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_shape_field(shape: &mut Shape, field: &str, value: &str) -> Result<(), CrudError> {
    match field {
        "shape_id" => shape.shape_id = ShapeId::from(value.to_string()),
        "shape_pt_lat" => shape.shape_pt_lat = Latitude(parse_value(value, field, "number")?),
        "shape_pt_lon" => shape.shape_pt_lon = Longitude(parse_value(value, field, "number")?),
        "shape_pt_sequence" => shape.shape_pt_sequence = parse_value(value, field, "integer")?,
        "shape_dist_traveled" => {
            shape.shape_dist_traveled = Some(parse_value(value, field, "number")?);
        }
        _ => return Err(unknown(field, Shape::valid_fields())),
    }
    Ok(())
}

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_frequency_field(
    freq: &mut Frequency,
    field: &str,
    value: &str,
) -> Result<(), CrudError> {
    match field {
        "trip_id" => freq.trip_id = TripId::from(value.to_string()),
        "start_time" => freq.start_time = parse_value(value, field, "time HH:MM:SS")?,
        "end_time" => freq.end_time = parse_value(value, field, "time HH:MM:SS")?,
        "headway_secs" => freq.headway_secs = parse_value(value, field, "integer")?,
        "exact_times" => {
            freq.exact_times = Some(parse_enum(value, field, ExactTimes::from_i32, "0-1")?);
        }
        _ => return Err(unknown(field, Frequency::valid_fields())),
    }
    Ok(())
}

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_transfer_field(tr: &mut Transfer, field: &str, value: &str) -> Result<(), CrudError> {
    match field {
        "from_stop_id" => tr.from_stop_id = Some(StopId::from(value.to_string())),
        "to_stop_id" => tr.to_stop_id = Some(StopId::from(value.to_string())),
        "from_route_id" => tr.from_route_id = Some(RouteId::from(value.to_string())),
        "to_route_id" => tr.to_route_id = Some(RouteId::from(value.to_string())),
        "from_trip_id" => tr.from_trip_id = Some(TripId::from(value.to_string())),
        "to_trip_id" => tr.to_trip_id = Some(TripId::from(value.to_string())),
        "transfer_type" => {
            tr.transfer_type = parse_enum(value, field, TransferType::from_i32, "0-3")?;
        }
        "min_transfer_time" => {
            tr.min_transfer_time = Some(parse_value(value, field, "integer")?);
        }
        _ => return Err(unknown(field, Transfer::valid_fields())),
    }
    Ok(())
}

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_pathway_field(pw: &mut Pathway, field: &str, value: &str) -> Result<(), CrudError> {
    match field {
        "pathway_id" => pw.pathway_id = PathwayId::from(value.to_string()),
        "from_stop_id" => pw.from_stop_id = StopId::from(value.to_string()),
        "to_stop_id" => pw.to_stop_id = StopId::from(value.to_string()),
        "pathway_mode" => {
            pw.pathway_mode = parse_enum(value, field, PathwayMode::from_i32, "1-7")?;
        }
        "is_bidirectional" => {
            pw.is_bidirectional = parse_enum(value, field, IsBidirectional::from_i32, "0-1")?;
        }
        "length" => pw.length = Some(parse_value(value, field, "number")?),
        "traversal_time" => pw.traversal_time = Some(parse_value(value, field, "integer")?),
        "stair_count" => pw.stair_count = Some(parse_value(value, field, "integer")?),
        "max_slope" => pw.max_slope = Some(parse_value(value, field, "number")?),
        "min_width" => pw.min_width = Some(parse_value(value, field, "number")?),
        "signposted_as" => pw.signposted_as = Some(value.to_string()),
        "reversed_signposted_as" => pw.reversed_signposted_as = Some(value.to_string()),
        _ => return Err(unknown(field, Pathway::valid_fields())),
    }
    Ok(())
}

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_level_field(level: &mut Level, field: &str, value: &str) -> Result<(), CrudError> {
    match field {
        "level_id" => level.level_id = LevelId::from(value.to_string()),
        "level_index" => level.level_index = parse_value(value, field, "number")?,
        "level_name" => level.level_name = Some(value.to_string()),
        _ => return Err(unknown(field, Level::valid_fields())),
    }
    Ok(())
}

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_feed_info_field(fi: &mut FeedInfo, field: &str, value: &str) -> Result<(), CrudError> {
    match field {
        "feed_publisher_name" => fi.feed_publisher_name = value.to_string(),
        "feed_publisher_url" => fi.feed_publisher_url = Url::from(value.to_string()),
        "feed_lang" => fi.feed_lang = LanguageCode::from(value.to_string()),
        "default_lang" => fi.default_lang = Some(LanguageCode::from(value.to_string())),
        "feed_start_date" => {
            fi.feed_start_date = Some(parse_value(value, field, "date YYYYMMDD")?);
        }
        "feed_end_date" => fi.feed_end_date = Some(parse_value(value, field, "date YYYYMMDD")?),
        "feed_version" => fi.feed_version = Some(value.to_string()),
        "feed_contact_email" => fi.feed_contact_email = Some(Email::from(value.to_string())),
        "feed_contact_url" => fi.feed_contact_url = Some(Url::from(value.to_string())),
        _ => return Err(unknown(field, FeedInfo::valid_fields())),
    }
    Ok(())
}

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_fare_attribute_field(
    fa: &mut FareAttribute,
    field: &str,
    value: &str,
) -> Result<(), CrudError> {
    match field {
        "fare_id" => fa.fare_id = FareId::from(value.to_string()),
        "price" => fa.price = parse_value(value, field, "number")?,
        "currency_type" => fa.currency_type = CurrencyCode::from(value.to_string()),
        "payment_method" => fa.payment_method = parse_value(value, field, "0 or 1")?,
        "transfers" => fa.transfers = Some(parse_value(value, field, "integer")?),
        "agency_id" => fa.agency_id = Some(AgencyId::from(value.to_string())),
        "transfer_duration" => fa.transfer_duration = Some(parse_value(value, field, "integer")?),
        _ => return Err(unknown(field, FareAttribute::valid_fields())),
    }
    Ok(())
}

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_fare_rule_field(fr: &mut FareRule, field: &str, value: &str) -> Result<(), CrudError> {
    match field {
        "fare_id" => fr.fare_id = FareId::from(value.to_string()),
        "route_id" => fr.route_id = Some(RouteId::from(value.to_string())),
        "origin_id" => fr.origin_id = Some(value.to_string()),
        "destination_id" => fr.destination_id = Some(value.to_string()),
        "contains_id" => fr.contains_id = Some(value.to_string()),
        _ => return Err(unknown(field, FareRule::valid_fields())),
    }
    Ok(())
}

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_translation_field(
    tr: &mut Translation,
    field: &str,
    value: &str,
) -> Result<(), CrudError> {
    match field {
        "table_name" => tr.table_name = value.to_string(),
        "field_name" => tr.field_name = value.to_string(),
        "language" => tr.language = LanguageCode::from(value.to_string()),
        "translation" => tr.translation = value.to_string(),
        "record_id" => tr.record_id = Some(value.to_string()),
        "record_sub_id" => tr.record_sub_id = Some(value.to_string()),
        "field_value" => tr.field_value = Some(value.to_string()),
        _ => return Err(unknown(field, Translation::valid_fields())),
    }
    Ok(())
}

/// # Errors
///
/// Returns an error on invalid input.
pub fn set_attribution_field(
    attr: &mut Attribution,
    field: &str,
    value: &str,
) -> Result<(), CrudError> {
    match field {
        "attribution_id" => attr.attribution_id = Some(value.to_string()),
        "agency_id" => attr.agency_id = Some(AgencyId::from(value.to_string())),
        "route_id" => attr.route_id = Some(RouteId::from(value.to_string())),
        "trip_id" => attr.trip_id = Some(TripId::from(value.to_string())),
        "organization_name" => attr.organization_name = value.to_string(),
        "is_producer" => attr.is_producer = Some(parse_value(value, field, "0 or 1")?),
        "is_operator" => attr.is_operator = Some(parse_value(value, field, "0 or 1")?),
        "is_authority" => attr.is_authority = Some(parse_value(value, field, "0 or 1")?),
        "attribution_url" => attr.attribution_url = Some(Url::from(value.to_string())),
        "attribution_email" => attr.attribution_email = Some(Email::from(value.to_string())),
        "attribution_phone" => attr.attribution_phone = Some(Phone::from(value.to_string())),
        _ => return Err(unknown(field, Attribution::valid_fields())),
    }
    Ok(())
}
