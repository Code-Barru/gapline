use crate::models::{
    Agency, Area, Attribution, BookingRule, Calendar, CalendarDate, FareAttribute, FareLegJoinRule,
    FareLegRule, FareMedia, FareProduct, FareRule, FareTransferRule, FeedInfo, Frequency, Level,
    LocationGroup, LocationGroupStop, Network, Pathway, RiderCategory, Route, RouteNetwork, Shape,
    Stop, StopArea, StopTime, Timeframe, Transfer, Translation, Trip,
};

/// Trait for GTFS records that can be filtered by the query engine.
///
/// Each implementor maps GTFS field names (as they appear in the CSV spec)
/// to their string representation.
pub trait Filterable {
    /// Returns the string value of `field`, or `None` if the field is unset (`Option::None`).
    ///
    /// Unknown field names also return `None` - use [`valid_fields`](Self::valid_fields)
    /// to check if a field name is recognized.
    fn field_value(&self, field: &str) -> Option<String>;

    /// Returns the list of recognized field names for this record type.
    fn valid_fields() -> &'static [&'static str];
}

fn bool_str(val: bool) -> String {
    if val { "1".into() } else { "0".into() }
}

macro_rules! filterable_value {
    ($self:ident, $f:ident, req) => {
        Some($self.$f.to_string())
    };
    ($self:ident, $f:ident, opt) => {
        $self.$f.as_ref().map(ToString::to_string)
    };
    ($self:ident, $f:ident, req_coord) => {
        Some($self.$f.0.to_string())
    };
    ($self:ident, $f:ident, opt_coord) => {
        $self.$f.as_ref().map(|v| v.0.to_string())
    };
    ($self:ident, $f:ident, req_enum) => {
        Some(($self.$f as i32).to_string())
    };
    ($self:ident, $f:ident, opt_enum) => {
        $self.$f.as_ref().map(|e| (*e as i32).to_string())
    };
    ($self:ident, $f:ident, req_bool) => {
        Some(bool_str($self.$f))
    };
    ($self:ident, $f:ident, route_type) => {
        Some($self.$f.to_i32().to_string())
    };
}

macro_rules! impl_filterable {
    ($ty:ty { $( $field:ident : $kind:tt ),* $(,)? }) => {
        impl Filterable for $ty {
            fn field_value(&self, field: &str) -> Option<String> {
                match field {
                    $( stringify!($field) => filterable_value!(self, $field, $kind), )*
                    _ => None,
                }
            }

            fn valid_fields() -> &'static [&'static str] {
                &[ $( stringify!($field) ),* ]
            }
        }
    };
}

impl_filterable!(Agency {
    agency_id: opt,
    agency_name: req,
    agency_url: req,
    agency_timezone: req,
    agency_lang: opt,
    agency_phone: opt,
    agency_fare_url: opt,
    agency_email: opt,
});

impl_filterable!(Stop {
    stop_id: req,
    stop_code: opt,
    stop_name: opt,
    tts_stop_name: opt,
    stop_desc: opt,
    stop_lat: opt_coord,
    stop_lon: opt_coord,
    zone_id: opt,
    stop_url: opt,
    location_type: opt_enum,
    parent_station: opt,
    stop_timezone: opt,
    wheelchair_boarding: opt_enum,
    level_id: opt,
    platform_code: opt,
});

impl_filterable!(Route {
    route_id: req,
    agency_id: opt,
    route_short_name: opt,
    route_long_name: opt,
    route_desc: opt,
    route_type: route_type,
    route_url: opt,
    route_color: opt,
    route_text_color: opt,
    route_sort_order: opt,
    continuous_pickup: opt_enum,
    continuous_drop_off: opt_enum,
    network_id: opt,
});

impl_filterable!(Trip {
    route_id: req,
    service_id: req,
    trip_id: req,
    trip_headsign: opt,
    trip_short_name: opt,
    direction_id: opt_enum,
    block_id: opt,
    shape_id: opt,
    wheelchair_accessible: opt_enum,
    bikes_allowed: opt_enum,
});

impl_filterable!(StopTime {
    trip_id: req,
    arrival_time: opt,
    departure_time: opt,
    stop_id: req,
    stop_sequence: req,
    stop_headsign: opt,
    pickup_type: opt_enum,
    drop_off_type: opt_enum,
    continuous_pickup: opt_enum,
    continuous_drop_off: opt_enum,
    shape_dist_traveled: opt,
    timepoint: opt_enum,
    start_pickup_drop_off_window: opt,
    end_pickup_drop_off_window: opt,
    pickup_booking_rule_id: opt,
    drop_off_booking_rule_id: opt,
    mean_duration_factor: opt,
    mean_duration_offset: opt,
    safe_duration_factor: opt,
    safe_duration_offset: opt,
});

impl_filterable!(Calendar {
    service_id: req,
    monday: req_bool,
    tuesday: req_bool,
    wednesday: req_bool,
    thursday: req_bool,
    friday: req_bool,
    saturday: req_bool,
    sunday: req_bool,
    start_date: req,
    end_date: req,
});

impl_filterable!(CalendarDate {
    service_id: req,
    date: req,
    exception_type: req_enum,
});

impl_filterable!(Shape {
    shape_id: req,
    shape_pt_lat: req_coord,
    shape_pt_lon: req_coord,
    shape_pt_sequence: req,
    shape_dist_traveled: opt,
});

impl_filterable!(Frequency {
    trip_id: req,
    start_time: req,
    end_time: req,
    headway_secs: req,
    exact_times: opt_enum,
});

impl_filterable!(Transfer {
    from_stop_id: opt,
    to_stop_id: opt,
    from_route_id: opt,
    to_route_id: opt,
    from_trip_id: opt,
    to_trip_id: opt,
    transfer_type: req_enum,
    min_transfer_time: opt,
});

impl_filterable!(Pathway {
    pathway_id: req,
    from_stop_id: req,
    to_stop_id: req,
    pathway_mode: req_enum,
    is_bidirectional: req_enum,
    length: opt,
    traversal_time: opt,
    stair_count: opt,
    max_slope: opt,
    min_width: opt,
    signposted_as: opt,
    reversed_signposted_as: opt,
});

impl_filterable!(Level {
    level_id: req,
    level_index: req,
    level_name: opt,
});

impl_filterable!(FeedInfo {
    feed_publisher_name: req,
    feed_publisher_url: req,
    feed_lang: req,
    default_lang: opt,
    feed_start_date: opt,
    feed_end_date: opt,
    feed_version: opt,
    feed_contact_email: opt,
    feed_contact_url: opt,
});

impl_filterable!(FareAttribute {
    fare_id: req,
    price: req,
    currency_type: req,
    payment_method: req,
    transfers: opt,
    agency_id: opt,
    transfer_duration: opt,
});

impl_filterable!(FareRule {
    fare_id: req,
    route_id: opt,
    origin_id: opt,
    destination_id: opt,
    contains_id: opt,
});

impl_filterable!(Translation {
    table_name: req,
    field_name: req,
    language: req,
    translation: req,
    record_id: opt,
    record_sub_id: opt,
    field_value: opt,
});

impl_filterable!(Attribution {
    attribution_id: opt,
    agency_id: opt,
    route_id: opt,
    trip_id: opt,
    organization_name: req,
    is_producer: opt,
    is_operator: opt,
    is_authority: opt,
    attribution_url: opt,
    attribution_email: opt,
    attribution_phone: opt,
});

impl_filterable!(BookingRule {
    booking_rule_id: req,
    booking_type: opt_enum,
    prior_notice_duration_min: opt,
    prior_notice_duration_max: opt,
    prior_notice_last_day: opt,
    prior_notice_last_time: opt,
    prior_notice_start_day: opt,
    prior_notice_start_time: opt,
    prior_notice_service_id: opt,
    message: opt,
    pickup_message: opt,
    drop_off_message: opt,
    phone_number: opt,
    info_url: opt,
    booking_url: opt,
});

impl_filterable!(LocationGroup {
    location_group_id: req,
    location_group_name: opt,
});

impl_filterable!(LocationGroupStop {
    location_group_id: req,
    stop_id: req,
});

impl_filterable!(FareMedia {
    fare_media_id: req,
    fare_media_name: opt,
    fare_media_type: req_enum,
});

impl_filterable!(FareProduct {
    fare_product_id: req,
    fare_product_name: opt,
    fare_media_id: opt,
    amount: req,
    currency: req,
    rider_category_id: opt,
});

impl_filterable!(FareLegRule {
    leg_group_id: opt,
    network_id: opt,
    from_area_id: opt,
    to_area_id: opt,
    from_timeframe_group_id: opt,
    to_timeframe_group_id: opt,
    fare_product_id: req,
    rule_priority: opt,
});

impl_filterable!(FareTransferRule {
    from_leg_group_id: opt,
    to_leg_group_id: opt,
    transfer_count: opt,
    duration_limit: opt,
    duration_limit_type: opt_enum,
    fare_transfer_type: req_enum,
    fare_product_id: opt,
});

impl_filterable!(RiderCategory {
    rider_category_id: req,
    rider_category_name: req,
    min_age: opt,
    max_age: opt,
    eligibility_url: opt,
});

impl_filterable!(Timeframe {
    timeframe_group_id: req,
    start_time: req,
    end_time: req,
    service_id: req,
});

impl_filterable!(Area {
    area_id: req,
    area_name: opt,
});

impl_filterable!(StopArea {
    area_id: req,
    stop_id: req,
});

impl_filterable!(Network {
    network_id: req,
    network_name: opt,
});

impl_filterable!(RouteNetwork {
    network_id: req,
    route_id: req,
});

impl_filterable!(FareLegJoinRule {
    from_network_id: req,
    to_network_id: req,
    from_stop_id: opt,
    to_stop_id: opt,
});
