use crate::models::{
    Agency, Attribution, Calendar, CalendarDate, FareAttribute, FareRule, FeedInfo, Frequency,
    Level, Pathway, Route, Shape, Stop, StopTime, Transfer, Translation, Trip,
};

/// Trait for GTFS records that can be filtered by the query engine.
///
/// Each implementor maps GTFS field names (as they appear in the CSV spec)
/// to their string representation.
pub trait Filterable {
    /// Returns the string value of `field`, or `None` if the field is unset (`Option::None`).
    ///
    /// Unknown field names also return `None` — use [`valid_fields`](Self::valid_fields)
    /// to check if a field name is recognized.
    fn field_value(&self, field: &str) -> Option<String>;

    /// Returns the list of recognized field names for this record type.
    fn valid_fields() -> &'static [&'static str];
}

// ---------------------------------------------------------------------------
// Helper: convert Option<T: Display> to Option<String>
// ---------------------------------------------------------------------------
fn opt_display<T: std::fmt::Display>(opt: Option<&T>) -> Option<String> {
    opt.map(std::string::ToString::to_string)
}

fn bool_str(val: bool) -> String {
    if val { "1".into() } else { "0".into() }
}

// ---------------------------------------------------------------------------
// Macro for enum → i32 → String conversion on Option<Enum>
// ---------------------------------------------------------------------------
macro_rules! opt_enum_i32 {
    ($opt:expr) => {
        $opt.as_ref().map(|e| (*e as i32).to_string())
    };
}

// ===========================================================================
// Agency
// ===========================================================================
impl Filterable for Agency {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "agency_id" => opt_display(self.agency_id.as_ref()),
            "agency_name" => Some(self.agency_name.clone()),
            "agency_url" => Some(self.agency_url.to_string()),
            "agency_timezone" => Some(self.agency_timezone.to_string()),
            "agency_lang" => opt_display(self.agency_lang.as_ref()),
            "agency_phone" => opt_display(self.agency_phone.as_ref()),
            "agency_fare_url" => opt_display(self.agency_fare_url.as_ref()),
            "agency_email" => opt_display(self.agency_email.as_ref()),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &[
            "agency_id",
            "agency_name",
            "agency_url",
            "agency_timezone",
            "agency_lang",
            "agency_phone",
            "agency_fare_url",
            "agency_email",
        ]
    }
}

// ===========================================================================
// Stop
// ===========================================================================
impl Filterable for Stop {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "stop_id" => Some(self.stop_id.to_string()),
            "stop_code" => self.stop_code.clone(),
            "stop_name" => self.stop_name.clone(),
            "tts_stop_name" => self.tts_stop_name.clone(),
            "stop_desc" => self.stop_desc.clone(),
            "stop_lat" => self.stop_lat.map(|v| v.0.to_string()),
            "stop_lon" => self.stop_lon.map(|v| v.0.to_string()),
            "zone_id" => self.zone_id.clone(),
            "stop_url" => opt_display(self.stop_url.as_ref()),
            "location_type" => opt_enum_i32!(self.location_type),
            "parent_station" => opt_display(self.parent_station.as_ref()),
            "stop_timezone" => opt_display(self.stop_timezone.as_ref()),
            "wheelchair_boarding" => opt_enum_i32!(self.wheelchair_boarding),
            "level_id" => opt_display(self.level_id.as_ref()),
            "platform_code" => self.platform_code.clone(),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &[
            "stop_id",
            "stop_code",
            "stop_name",
            "tts_stop_name",
            "stop_desc",
            "stop_lat",
            "stop_lon",
            "zone_id",
            "stop_url",
            "location_type",
            "parent_station",
            "stop_timezone",
            "wheelchair_boarding",
            "level_id",
            "platform_code",
        ]
    }
}

// ===========================================================================
// Route
// ===========================================================================
impl Filterable for Route {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "route_id" => Some(self.route_id.to_string()),
            "agency_id" => opt_display(self.agency_id.as_ref()),
            "route_short_name" => self.route_short_name.clone(),
            "route_long_name" => self.route_long_name.clone(),
            "route_desc" => self.route_desc.clone(),
            "route_type" => Some(self.route_type.to_i32().to_string()),
            "route_url" => opt_display(self.route_url.as_ref()),
            "route_color" => opt_display(self.route_color.as_ref()),
            "route_text_color" => opt_display(self.route_text_color.as_ref()),
            "route_sort_order" => self.route_sort_order.map(|v| v.to_string()),
            "continuous_pickup" => opt_enum_i32!(self.continuous_pickup),
            "continuous_drop_off" => opt_enum_i32!(self.continuous_drop_off),
            "network_id" => self.network_id.clone(),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &[
            "route_id",
            "agency_id",
            "route_short_name",
            "route_long_name",
            "route_desc",
            "route_type",
            "route_url",
            "route_color",
            "route_text_color",
            "route_sort_order",
            "continuous_pickup",
            "continuous_drop_off",
            "network_id",
        ]
    }
}

// ===========================================================================
// Trip
// ===========================================================================
impl Filterable for Trip {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "route_id" => Some(self.route_id.to_string()),
            "service_id" => Some(self.service_id.to_string()),
            "trip_id" => Some(self.trip_id.to_string()),
            "trip_headsign" => self.trip_headsign.clone(),
            "trip_short_name" => self.trip_short_name.clone(),
            "direction_id" => opt_enum_i32!(self.direction_id),
            "block_id" => self.block_id.clone(),
            "shape_id" => opt_display(self.shape_id.as_ref()),
            "wheelchair_accessible" => opt_enum_i32!(self.wheelchair_accessible),
            "bikes_allowed" => opt_enum_i32!(self.bikes_allowed),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &[
            "route_id",
            "service_id",
            "trip_id",
            "trip_headsign",
            "trip_short_name",
            "direction_id",
            "block_id",
            "shape_id",
            "wheelchair_accessible",
            "bikes_allowed",
        ]
    }
}

// ===========================================================================
// StopTime
// ===========================================================================
impl Filterable for StopTime {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "trip_id" => Some(self.trip_id.to_string()),
            "arrival_time" => opt_display(self.arrival_time.as_ref()),
            "departure_time" => opt_display(self.departure_time.as_ref()),
            "stop_id" => Some(self.stop_id.to_string()),
            "stop_sequence" => Some(self.stop_sequence.to_string()),
            "stop_headsign" => self.stop_headsign.clone(),
            "pickup_type" => opt_enum_i32!(self.pickup_type),
            "drop_off_type" => opt_enum_i32!(self.drop_off_type),
            "continuous_pickup" => opt_enum_i32!(self.continuous_pickup),
            "continuous_drop_off" => opt_enum_i32!(self.continuous_drop_off),
            "shape_dist_traveled" => self.shape_dist_traveled.map(|v| v.to_string()),
            "timepoint" => opt_enum_i32!(self.timepoint),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &[
            "trip_id",
            "arrival_time",
            "departure_time",
            "stop_id",
            "stop_sequence",
            "stop_headsign",
            "pickup_type",
            "drop_off_type",
            "continuous_pickup",
            "continuous_drop_off",
            "shape_dist_traveled",
            "timepoint",
        ]
    }
}

// ===========================================================================
// Calendar
// ===========================================================================
impl Filterable for Calendar {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "service_id" => Some(self.service_id.to_string()),
            "monday" => Some(bool_str(self.monday)),
            "tuesday" => Some(bool_str(self.tuesday)),
            "wednesday" => Some(bool_str(self.wednesday)),
            "thursday" => Some(bool_str(self.thursday)),
            "friday" => Some(bool_str(self.friday)),
            "saturday" => Some(bool_str(self.saturday)),
            "sunday" => Some(bool_str(self.sunday)),
            "start_date" => Some(self.start_date.to_string()),
            "end_date" => Some(self.end_date.to_string()),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &[
            "service_id",
            "monday",
            "tuesday",
            "wednesday",
            "thursday",
            "friday",
            "saturday",
            "sunday",
            "start_date",
            "end_date",
        ]
    }
}

// ===========================================================================
// CalendarDate
// ===========================================================================
impl Filterable for CalendarDate {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "service_id" => Some(self.service_id.to_string()),
            "date" => Some(self.date.to_string()),
            "exception_type" => Some((self.exception_type as i32).to_string()),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &["service_id", "date", "exception_type"]
    }
}

// ===========================================================================
// Shape
// ===========================================================================
impl Filterable for Shape {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "shape_id" => Some(self.shape_id.to_string()),
            "shape_pt_lat" => Some(self.shape_pt_lat.0.to_string()),
            "shape_pt_lon" => Some(self.shape_pt_lon.0.to_string()),
            "shape_pt_sequence" => Some(self.shape_pt_sequence.to_string()),
            "shape_dist_traveled" => self.shape_dist_traveled.map(|v| v.to_string()),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &[
            "shape_id",
            "shape_pt_lat",
            "shape_pt_lon",
            "shape_pt_sequence",
            "shape_dist_traveled",
        ]
    }
}

// ===========================================================================
// Frequency
// ===========================================================================
impl Filterable for Frequency {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "trip_id" => Some(self.trip_id.to_string()),
            "start_time" => Some(self.start_time.to_string()),
            "end_time" => Some(self.end_time.to_string()),
            "headway_secs" => Some(self.headway_secs.to_string()),
            "exact_times" => opt_enum_i32!(self.exact_times),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &[
            "trip_id",
            "start_time",
            "end_time",
            "headway_secs",
            "exact_times",
        ]
    }
}

// ===========================================================================
// Transfer
// ===========================================================================
impl Filterable for Transfer {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "from_stop_id" => opt_display(self.from_stop_id.as_ref()),
            "to_stop_id" => opt_display(self.to_stop_id.as_ref()),
            "from_route_id" => opt_display(self.from_route_id.as_ref()),
            "to_route_id" => opt_display(self.to_route_id.as_ref()),
            "from_trip_id" => opt_display(self.from_trip_id.as_ref()),
            "to_trip_id" => opt_display(self.to_trip_id.as_ref()),
            "transfer_type" => Some((self.transfer_type as i32).to_string()),
            "min_transfer_time" => self.min_transfer_time.map(|v| v.to_string()),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &[
            "from_stop_id",
            "to_stop_id",
            "from_route_id",
            "to_route_id",
            "from_trip_id",
            "to_trip_id",
            "transfer_type",
            "min_transfer_time",
        ]
    }
}

// ===========================================================================
// Pathway
// ===========================================================================
impl Filterable for Pathway {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "pathway_id" => Some(self.pathway_id.to_string()),
            "from_stop_id" => Some(self.from_stop_id.to_string()),
            "to_stop_id" => Some(self.to_stop_id.to_string()),
            "pathway_mode" => Some((self.pathway_mode as i32).to_string()),
            "is_bidirectional" => Some((self.is_bidirectional as i32).to_string()),
            "length" => self.length.map(|v| v.to_string()),
            "traversal_time" => self.traversal_time.map(|v| v.to_string()),
            "stair_count" => self.stair_count.map(|v| v.to_string()),
            "max_slope" => self.max_slope.map(|v| v.to_string()),
            "min_width" => self.min_width.map(|v| v.to_string()),
            "signposted_as" => self.signposted_as.clone(),
            "reversed_signposted_as" => self.reversed_signposted_as.clone(),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &[
            "pathway_id",
            "from_stop_id",
            "to_stop_id",
            "pathway_mode",
            "is_bidirectional",
            "length",
            "traversal_time",
            "stair_count",
            "max_slope",
            "min_width",
            "signposted_as",
            "reversed_signposted_as",
        ]
    }
}

// ===========================================================================
// Level
// ===========================================================================
impl Filterable for Level {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "level_id" => Some(self.level_id.to_string()),
            "level_index" => Some(self.level_index.to_string()),
            "level_name" => self.level_name.clone(),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &["level_id", "level_index", "level_name"]
    }
}

// ===========================================================================
// FeedInfo
// ===========================================================================
impl Filterable for FeedInfo {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "feed_publisher_name" => Some(self.feed_publisher_name.clone()),
            "feed_publisher_url" => Some(self.feed_publisher_url.to_string()),
            "feed_lang" => Some(self.feed_lang.to_string()),
            "default_lang" => opt_display(self.default_lang.as_ref()),
            "feed_start_date" => opt_display(self.feed_start_date.as_ref()),
            "feed_end_date" => opt_display(self.feed_end_date.as_ref()),
            "feed_version" => self.feed_version.clone(),
            "feed_contact_email" => opt_display(self.feed_contact_email.as_ref()),
            "feed_contact_url" => opt_display(self.feed_contact_url.as_ref()),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &[
            "feed_publisher_name",
            "feed_publisher_url",
            "feed_lang",
            "default_lang",
            "feed_start_date",
            "feed_end_date",
            "feed_version",
            "feed_contact_email",
            "feed_contact_url",
        ]
    }
}

// ===========================================================================
// FareAttribute
// ===========================================================================
impl Filterable for FareAttribute {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "fare_id" => Some(self.fare_id.to_string()),
            "price" => Some(self.price.to_string()),
            "currency_type" => Some(self.currency_type.to_string()),
            "payment_method" => Some(self.payment_method.to_string()),
            "transfers" => self.transfers.map(|v| v.to_string()),
            "agency_id" => opt_display(self.agency_id.as_ref()),
            "transfer_duration" => self.transfer_duration.map(|v| v.to_string()),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &[
            "fare_id",
            "price",
            "currency_type",
            "payment_method",
            "transfers",
            "agency_id",
            "transfer_duration",
        ]
    }
}

// ===========================================================================
// FareRule
// ===========================================================================
impl Filterable for FareRule {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "fare_id" => Some(self.fare_id.to_string()),
            "route_id" => opt_display(self.route_id.as_ref()),
            "origin_id" => self.origin_id.clone(),
            "destination_id" => self.destination_id.clone(),
            "contains_id" => self.contains_id.clone(),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &[
            "fare_id",
            "route_id",
            "origin_id",
            "destination_id",
            "contains_id",
        ]
    }
}

// ===========================================================================
// Translation
// ===========================================================================
impl Filterable for Translation {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "table_name" => Some(self.table_name.clone()),
            "field_name" => Some(self.field_name.clone()),
            "language" => Some(self.language.to_string()),
            "translation" => Some(self.translation.clone()),
            "record_id" => self.record_id.clone(),
            "record_sub_id" => self.record_sub_id.clone(),
            "field_value" => self.field_value.clone(),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &[
            "table_name",
            "field_name",
            "language",
            "translation",
            "record_id",
            "record_sub_id",
            "field_value",
        ]
    }
}

// ===========================================================================
// Attribution
// ===========================================================================
impl Filterable for Attribution {
    fn field_value(&self, field: &str) -> Option<String> {
        match field {
            "attribution_id" => self.attribution_id.clone(),
            "agency_id" => opt_display(self.agency_id.as_ref()),
            "route_id" => opt_display(self.route_id.as_ref()),
            "trip_id" => opt_display(self.trip_id.as_ref()),
            "organization_name" => Some(self.organization_name.clone()),
            "is_producer" => self.is_producer.map(|v| v.to_string()),
            "is_operator" => self.is_operator.map(|v| v.to_string()),
            "is_authority" => self.is_authority.map(|v| v.to_string()),
            "attribution_url" => opt_display(self.attribution_url.as_ref()),
            "attribution_email" => opt_display(self.attribution_email.as_ref()),
            "attribution_phone" => opt_display(self.attribution_phone.as_ref()),
            _ => None,
        }
    }

    fn valid_fields() -> &'static [&'static str] {
        &[
            "attribution_id",
            "agency_id",
            "route_id",
            "trip_id",
            "organization_name",
            "is_producer",
            "is_operator",
            "is_authority",
            "attribution_url",
            "attribution_email",
            "attribution_phone",
        ]
    }
}
