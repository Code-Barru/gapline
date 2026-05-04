use serde::{Deserialize, Serialize};

use super::enums::BookingType;
use super::ids::{BookingRuleId, LocationGroupId, ServiceId, StopId};
use super::types::{GtfsTime, Phone, Url};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingRule {
    pub booking_rule_id: BookingRuleId,
    pub booking_type: BookingType,
    pub prior_notice_duration_min: Option<u32>,
    pub prior_notice_duration_max: Option<u32>,
    pub prior_notice_last_day: Option<u32>,
    pub prior_notice_last_time: Option<GtfsTime>,
    pub prior_notice_start_day: Option<u32>,
    pub prior_notice_start_time: Option<GtfsTime>,
    pub prior_notice_service_id: Option<ServiceId>,
    pub message: Option<String>,
    pub pickup_message: Option<String>,
    pub drop_off_message: Option<String>,
    pub phone_number: Option<Phone>,
    pub info_url: Option<Url>,
    pub booking_url: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationGroup {
    pub location_group_id: LocationGroupId,
    pub location_group_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationGroupStop {
    pub location_group_id: LocationGroupId,
    pub stop_id: StopId,
}
