use crate::crud::read::GtfsTarget;
use crate::models::{
    AgencyId, AreaId, FareId, FareMediaId, FareProductId, GtfsDate, LegGroupId, LevelId, NetworkId,
    PathwayId, RiderCategoryId, RouteId, ServiceId, ShapeId, StopId, TimeframeId, TripId, ZoneId,
};

/// Identifies a GTFS entity. `Transfer`/`FareRule`/`Attribution` use their
/// positional index in the feed (no natural PK).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityRef {
    Agency(AgencyId),
    Stop(StopId),
    Zone(ZoneId),
    Route(RouteId),
    Trip(TripId),
    Service(ServiceId),
    Shape(ShapeId),
    Fare(FareId),
    Pathway(PathwayId),
    Level(LevelId),
    /// (`trip_id`, `stop_sequence`)
    StopTime(TripId, u32),
    /// (`shape_id`, `shape_pt_sequence`)
    ShapePoint(ShapeId, u32),
    /// (`trip_id`, `start_time` as `total_seconds`)
    Frequency(TripId, u32),
    /// (`service_id`, date)
    CalendarDate(ServiceId, GtfsDate),
    /// Index in `GtfsFeed::transfers`
    Transfer(usize),
    /// Index in `GtfsFeed::fare_rules`
    FareRule(usize),
    /// Index in `GtfsFeed::attributions`
    Attribution(usize),

    // Fares v2
    FareMedia(FareMediaId),
    FareProduct(FareProductId),
    RiderCategory(RiderCategoryId),
    Timeframe(TimeframeId),
    Area(AreaId),
    Network(NetworkId),
    LegGroup(LegGroupId),
    /// Index in `GtfsFeed::fare_leg_rules`
    FareLegRule(usize),
    /// Index in `GtfsFeed::fare_transfer_rules`
    FareTransferRule(usize),
    /// Index in `GtfsFeed::stop_areas`
    StopArea(usize),
    /// Index in `GtfsFeed::route_networks`
    RouteNetwork(usize),
    /// Index in `GtfsFeed::fare_leg_join_rules`
    FareLegJoinRule(usize),
}

impl EntityRef {
    /// Returns the [`GtfsTarget`] this entity belongs to.
    ///
    /// `LegGroup` has no dedicated file (IDs are declared inline in
    /// `fare_leg_rules.txt`); it is bucketed under `FareLegRules`.
    #[must_use]
    pub fn target(&self) -> GtfsTarget {
        match self {
            Self::Agency(_) => GtfsTarget::Agency,
            Self::Stop(_) | Self::Zone(_) => GtfsTarget::Stops,
            Self::Route(_) => GtfsTarget::Routes,
            Self::Trip(_) => GtfsTarget::Trips,
            Self::Service(_) => GtfsTarget::Calendar,
            Self::Shape(_) | Self::ShapePoint(_, _) => GtfsTarget::Shapes,
            Self::Fare(_) => GtfsTarget::FareAttributes,
            Self::Pathway(_) => GtfsTarget::Pathways,
            Self::Level(_) => GtfsTarget::Levels,
            Self::StopTime(_, _) => GtfsTarget::StopTimes,
            Self::Frequency(_, _) => GtfsTarget::Frequencies,
            Self::CalendarDate(_, _) => GtfsTarget::CalendarDates,
            Self::Transfer(_) => GtfsTarget::Transfers,
            Self::FareRule(_) => GtfsTarget::FareRules,
            Self::Attribution(_) => GtfsTarget::Attributions,
            Self::FareMedia(_) => GtfsTarget::FareMedia,
            Self::FareProduct(_) => GtfsTarget::FareProducts,
            Self::RiderCategory(_) => GtfsTarget::RiderCategories,
            Self::Timeframe(_) => GtfsTarget::Timeframes,
            Self::Area(_) => GtfsTarget::Areas,
            Self::Network(_) => GtfsTarget::Networks,
            Self::LegGroup(_) | Self::FareLegRule(_) => GtfsTarget::FareLegRules,
            Self::FareTransferRule(_) => GtfsTarget::FareTransferRules,
            Self::StopArea(_) => GtfsTarget::StopAreas,
            Self::RouteNetwork(_) => GtfsTarget::RouteNetworks,
            Self::FareLegJoinRule(_) => GtfsTarget::FareLegJoinRules,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationType {
    // Core
    AgencyOfRoute,
    RouteOfTrip,
    ServiceOfTrip,
    ShapeOfTrip,
    TripOfStopTime,
    StopOfStopTime,
    ServiceOfCalendarDate,
    TripOfFrequency,

    // Stops self-references
    ParentStation,
    LevelOfStop,

    // Transfers
    TransferFromStop,
    TransferToStop,
    TransferFromRoute,
    TransferToRoute,
    TransferFromTrip,
    TransferToTrip,

    // Pathways
    PathwayFromStop,
    PathwayToStop,

    // Fares v1
    FareOfFareRule,
    RouteOfFareRule,
    OriginZoneOfFareRule,
    DestinationZoneOfFareRule,
    ContainsZoneOfFareRule,
    AgencyOfFareAttribute,

    // Attributions
    AgencyOfAttribution,
    RouteOfAttribution,
    TripOfAttribution,

    // Fares v2
    MediaOfFareProduct,
    RiderCategoryOfFareProduct,
    LegGroupOfFareLegRule,
    NetworkOfFareLegRule,
    FromAreaOfFareLegRule,
    ToAreaOfFareLegRule,
    FromTimeframeOfFareLegRule,
    ToTimeframeOfFareLegRule,
    ProductOfFareLegRule,
    FromLegGroupOfFareTransferRule,
    ToLegGroupOfFareTransferRule,
    ProductOfFareTransferRule,
    AreaOfStopArea,
    StopOfStopArea,
    NetworkOfRouteNetwork,
    RouteOfRouteNetwork,
    FromNetworkOfFareLegJoinRule,
    ToNetworkOfFareLegJoinRule,
    FromStopOfFareLegJoinRule,
    ToStopOfFareLegJoinRule,
}

impl RelationType {
    /// Returns the FK field name in the dependent record for this relation.
    #[must_use]
    pub fn fk_field_name(self) -> &'static str {
        match self {
            Self::AgencyOfRoute | Self::AgencyOfFareAttribute | Self::AgencyOfAttribution => {
                "agency_id"
            }
            Self::RouteOfTrip
            | Self::RouteOfFareRule
            | Self::RouteOfAttribution
            | Self::RouteOfRouteNetwork => "route_id",
            Self::ServiceOfTrip | Self::ServiceOfCalendarDate => "service_id",
            Self::ShapeOfTrip => "shape_id",
            Self::TripOfStopTime | Self::TripOfFrequency | Self::TripOfAttribution => "trip_id",
            Self::StopOfStopTime | Self::StopOfStopArea => "stop_id",
            Self::ParentStation => "parent_station",
            Self::LevelOfStop => "level_id",
            Self::TransferFromStop | Self::PathwayFromStop | Self::FromStopOfFareLegJoinRule => {
                "from_stop_id"
            }
            Self::TransferToStop | Self::PathwayToStop | Self::ToStopOfFareLegJoinRule => {
                "to_stop_id"
            }
            Self::TransferFromRoute => "from_route_id",
            Self::TransferToRoute => "to_route_id",
            Self::TransferFromTrip => "from_trip_id",
            Self::TransferToTrip => "to_trip_id",
            Self::FareOfFareRule => "fare_id",
            Self::OriginZoneOfFareRule => "origin_id",
            Self::DestinationZoneOfFareRule => "destination_id",
            Self::ContainsZoneOfFareRule => "contains_id",
            Self::MediaOfFareProduct => "fare_media_id",
            Self::RiderCategoryOfFareProduct => "rider_category_id",
            Self::LegGroupOfFareLegRule => "leg_group_id",
            Self::NetworkOfFareLegRule | Self::NetworkOfRouteNetwork => "network_id",
            Self::FromAreaOfFareLegRule => "from_area_id",
            Self::ToAreaOfFareLegRule => "to_area_id",
            Self::FromTimeframeOfFareLegRule => "from_timeframe_group_id",
            Self::ToTimeframeOfFareLegRule => "to_timeframe_group_id",
            Self::ProductOfFareLegRule | Self::ProductOfFareTransferRule => "fare_product_id",
            Self::FromLegGroupOfFareTransferRule => "from_leg_group_id",
            Self::ToLegGroupOfFareTransferRule => "to_leg_group_id",
            Self::AreaOfStopArea => "area_id",
            Self::FromNetworkOfFareLegJoinRule => "from_network_id",
            Self::ToNetworkOfFareLegJoinRule => "to_network_id",
        }
    }
}
