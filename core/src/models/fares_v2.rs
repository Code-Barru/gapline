use serde::{Deserialize, Serialize};

use super::enums::{DurationLimitType, FareMediaType, FareTransferType};
use super::ids::{
    AreaId, FareMediaId, FareProductId, LegGroupId, NetworkId, RiderCategoryId, RouteId, ServiceId,
    StopId, TimeframeId,
};
use super::types::{CurrencyCode, GtfsTime, Url};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FareMedia {
    pub fare_media_id: FareMediaId,
    pub fare_media_name: Option<String>,
    pub fare_media_type: FareMediaType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FareProduct {
    pub fare_product_id: FareProductId,
    pub fare_product_name: Option<String>,
    pub fare_media_id: Option<FareMediaId>,
    pub amount: f64,
    pub currency: CurrencyCode,
    pub rider_category_id: Option<RiderCategoryId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FareLegRule {
    pub leg_group_id: Option<LegGroupId>,
    pub network_id: Option<NetworkId>,
    pub from_area_id: Option<AreaId>,
    pub to_area_id: Option<AreaId>,
    pub from_timeframe_group_id: Option<TimeframeId>,
    pub to_timeframe_group_id: Option<TimeframeId>,
    pub fare_product_id: FareProductId,
    pub rule_priority: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FareTransferRule {
    pub from_leg_group_id: Option<LegGroupId>,
    pub to_leg_group_id: Option<LegGroupId>,
    pub transfer_count: Option<i32>,
    pub duration_limit: Option<u32>,
    pub duration_limit_type: Option<DurationLimitType>,
    pub fare_transfer_type: FareTransferType,
    pub fare_product_id: Option<FareProductId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiderCategory {
    pub rider_category_id: RiderCategoryId,
    pub rider_category_name: String,
    pub min_age: Option<u32>,
    pub max_age: Option<u32>,
    pub eligibility_url: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeframe {
    pub timeframe_group_id: TimeframeId,
    pub start_time: GtfsTime,
    pub end_time: GtfsTime,
    pub service_id: ServiceId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area {
    pub area_id: AreaId,
    pub area_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopArea {
    pub area_id: AreaId,
    pub stop_id: StopId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub network_id: NetworkId,
    pub network_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteNetwork {
    pub network_id: NetworkId,
    pub route_id: RouteId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FareLegJoinRule {
    pub from_network_id: NetworkId,
    pub to_network_id: NetworkId,
    pub from_stop_id: Option<StopId>,
    pub to_stop_id: Option<StopId>,
}
