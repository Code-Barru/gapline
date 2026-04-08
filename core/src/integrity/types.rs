use crate::crud::read::GtfsTarget;
use crate::models::{
    AgencyId, FareId, GtfsDate, LevelId, PathwayId, RouteId, ServiceId, ShapeId, StopId, TripId,
    ZoneId,
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
}

impl EntityRef {
    /// Returns the [`GtfsTarget`] this entity belongs to.
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
}

impl RelationType {
    /// Returns the FK field name in the dependent record for this relation.
    #[must_use]
    pub fn fk_field_name(self) -> &'static str {
        match self {
            Self::AgencyOfRoute | Self::AgencyOfFareAttribute | Self::AgencyOfAttribution => {
                "agency_id"
            }
            Self::RouteOfTrip | Self::RouteOfFareRule | Self::RouteOfAttribution => "route_id",
            Self::ServiceOfTrip | Self::ServiceOfCalendarDate => "service_id",
            Self::ShapeOfTrip => "shape_id",
            Self::TripOfStopTime | Self::TripOfFrequency | Self::TripOfAttribution => "trip_id",
            Self::StopOfStopTime => "stop_id",
            Self::ParentStation => "parent_station",
            Self::LevelOfStop => "level_id",
            Self::TransferFromStop | Self::PathwayFromStop => "from_stop_id",
            Self::TransferToStop | Self::PathwayToStop => "to_stop_id",
            Self::TransferFromRoute => "from_route_id",
            Self::TransferToRoute => "to_route_id",
            Self::TransferFromTrip => "from_trip_id",
            Self::TransferToTrip => "to_trip_id",
            Self::FareOfFareRule => "fare_id",
            Self::OriginZoneOfFareRule => "origin_id",
            Self::DestinationZoneOfFareRule => "destination_id",
            Self::ContainsZoneOfFareRule => "contains_id",
        }
    }
}
