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
