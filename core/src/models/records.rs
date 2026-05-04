use serde::{Deserialize, Serialize};

use super::enums::{
    BikesAllowed, ContinuousDropOff, ContinuousPickup, DirectionId, DropOffType, ExactTimes,
    ExceptionType, IsBidirectional, LocationType, PathwayMode, PickupType, RouteType, Timepoint,
    TransferType, WheelchairAccessible,
};
use super::ids::{
    AgencyId, BookingRuleId, FareId, LevelId, PathwayId, RouteId, ServiceId, ShapeId, StopId,
    TripId,
};
use super::types::{
    Color, CurrencyCode, Email, GtfsDate, GtfsTime, LanguageCode, Latitude, Longitude, Phone,
    Timezone, Url,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agency {
    pub agency_id: Option<AgencyId>,
    pub agency_name: String,
    pub agency_url: Url,
    pub agency_timezone: Timezone,
    pub agency_lang: Option<LanguageCode>,
    pub agency_phone: Option<Phone>,
    pub agency_fare_url: Option<Url>,
    pub agency_email: Option<Email>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stop {
    pub stop_id: StopId,
    pub stop_code: Option<String>,
    pub stop_name: Option<String>,
    pub tts_stop_name: Option<String>,
    pub stop_desc: Option<String>,
    pub stop_lat: Option<Latitude>,
    pub stop_lon: Option<Longitude>,
    pub zone_id: Option<String>,
    pub stop_url: Option<Url>,
    pub location_type: Option<LocationType>,
    pub parent_station: Option<StopId>,
    pub stop_timezone: Option<Timezone>,
    pub wheelchair_boarding: Option<WheelchairAccessible>,
    pub level_id: Option<LevelId>,
    pub platform_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub route_id: RouteId,
    pub agency_id: Option<AgencyId>,
    pub route_short_name: Option<String>,
    pub route_long_name: Option<String>,
    pub route_desc: Option<String>,
    pub route_type: RouteType,
    pub route_url: Option<Url>,
    pub route_color: Option<Color>,
    pub route_text_color: Option<Color>,
    pub route_sort_order: Option<u32>,
    pub continuous_pickup: Option<ContinuousPickup>,
    pub continuous_drop_off: Option<ContinuousDropOff>,
    pub network_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trip {
    pub route_id: RouteId,
    pub service_id: ServiceId,
    pub trip_id: TripId,
    pub trip_headsign: Option<String>,
    pub trip_short_name: Option<String>,
    pub direction_id: Option<DirectionId>,
    pub block_id: Option<String>,
    pub shape_id: Option<ShapeId>,
    pub wheelchair_accessible: Option<WheelchairAccessible>,
    pub bikes_allowed: Option<BikesAllowed>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopTime {
    pub trip_id: TripId,
    pub arrival_time: Option<GtfsTime>,
    pub departure_time: Option<GtfsTime>,
    pub stop_id: StopId,
    pub stop_sequence: u32,
    pub stop_headsign: Option<String>,
    pub pickup_type: Option<PickupType>,
    pub drop_off_type: Option<DropOffType>,
    pub continuous_pickup: Option<ContinuousPickup>,
    pub continuous_drop_off: Option<ContinuousDropOff>,
    pub shape_dist_traveled: Option<f64>,
    pub timepoint: Option<Timepoint>,
    pub start_pickup_drop_off_window: Option<GtfsTime>,
    pub end_pickup_drop_off_window: Option<GtfsTime>,
    pub pickup_booking_rule_id: Option<BookingRuleId>,
    pub drop_off_booking_rule_id: Option<BookingRuleId>,
    pub mean_duration_factor: Option<f64>,
    pub mean_duration_offset: Option<f64>,
    pub safe_duration_factor: Option<f64>,
    pub safe_duration_offset: Option<f64>,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    pub service_id: ServiceId,
    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,
    pub start_date: GtfsDate,
    pub end_date: GtfsDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarDate {
    pub service_id: ServiceId,
    pub date: GtfsDate,
    pub exception_type: ExceptionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shape {
    pub shape_id: ShapeId,
    pub shape_pt_lat: Latitude,
    pub shape_pt_lon: Longitude,
    pub shape_pt_sequence: u32,
    pub shape_dist_traveled: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frequency {
    pub trip_id: TripId,
    pub start_time: GtfsTime,
    pub end_time: GtfsTime,
    pub headway_secs: u32,
    pub exact_times: Option<ExactTimes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub from_stop_id: Option<StopId>,
    pub to_stop_id: Option<StopId>,
    pub from_route_id: Option<RouteId>,
    pub to_route_id: Option<RouteId>,
    pub from_trip_id: Option<TripId>,
    pub to_trip_id: Option<TripId>,
    pub transfer_type: TransferType,
    pub min_transfer_time: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pathway {
    pub pathway_id: PathwayId,
    pub from_stop_id: StopId,
    pub to_stop_id: StopId,
    pub pathway_mode: PathwayMode,
    pub is_bidirectional: IsBidirectional,
    pub length: Option<f64>,
    pub traversal_time: Option<u32>,
    pub stair_count: Option<i32>,
    pub max_slope: Option<f64>,
    pub min_width: Option<f64>,
    pub signposted_as: Option<String>,
    pub reversed_signposted_as: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub level_id: LevelId,
    pub level_index: f64,
    pub level_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedInfo {
    pub feed_publisher_name: String,
    pub feed_publisher_url: Url,
    pub feed_lang: LanguageCode,
    pub default_lang: Option<LanguageCode>,
    pub feed_start_date: Option<GtfsDate>,
    pub feed_end_date: Option<GtfsDate>,
    pub feed_version: Option<String>,
    pub feed_contact_email: Option<Email>,
    pub feed_contact_url: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FareAttribute {
    pub fare_id: FareId,
    pub price: f64,
    pub currency_type: CurrencyCode,
    pub payment_method: u8,
    pub transfers: Option<u8>,
    pub agency_id: Option<AgencyId>,
    pub transfer_duration: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FareRule {
    pub fare_id: FareId,
    pub route_id: Option<RouteId>,
    pub origin_id: Option<String>,
    pub destination_id: Option<String>,
    pub contains_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Translation {
    pub table_name: String,
    pub field_name: String,
    pub language: LanguageCode,
    pub translation: String,
    pub record_id: Option<String>,
    pub record_sub_id: Option<String>,
    pub field_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribution {
    pub attribution_id: Option<String>,
    pub agency_id: Option<AgencyId>,
    pub route_id: Option<RouteId>,
    pub trip_id: Option<TripId>,
    pub organization_name: String,
    pub is_producer: Option<u8>,
    pub is_operator: Option<u8>,
    pub is_authority: Option<u8>,
    pub attribution_url: Option<Url>,
    pub attribution_email: Option<Email>,
    pub attribution_phone: Option<Phone>,
}
