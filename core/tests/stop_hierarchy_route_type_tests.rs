use gapline_core::models::{
    GtfsFeed, LocationType, Route, RouteId, RouteType, Stop, StopId, StopTime, TripId,
};
use gapline_core::validation::schedule_time_validation::route_type_semantics::RouteTypeSemanticsRule;
use gapline_core::validation::schedule_time_validation::stop_hierarchy::{
    InvalidParentTypeRule, UnusedStationRule, UnusedStopRule,
};
use gapline_core::validation::{Severity, ValidationRule};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_stop(id: &str, loc_type: Option<LocationType>, parent: Option<&str>) -> Stop {
    Stop {
        stop_id: StopId::from(id),
        stop_code: None,
        stop_name: Some("Stop".to_string()),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: None,
        stop_lon: None,
        zone_id: None,
        stop_url: None,
        location_type: loc_type,
        parent_station: parent.map(|p| StopId::from(p.to_string())),
        stop_timezone: None,
        wheelchair_boarding: None,
        level_id: None,
        platform_code: None,
    }
}

fn make_route(id: &str, route_type: RouteType) -> Route {
    Route {
        route_id: RouteId::from(id.to_string()),
        agency_id: None,
        route_short_name: Some("R".to_string()),
        route_long_name: None,
        route_desc: None,
        route_type,
        route_url: None,
        route_color: None,
        route_text_color: None,
        route_sort_order: None,
        continuous_pickup: None,
        continuous_drop_off: None,
        network_id: None,
    }
}

fn make_stop_time(trip_id: &str, stop_id: &str, seq: u32) -> StopTime {
    StopTime {
        trip_id: TripId::from(trip_id.to_string()),
        arrival_time: None,
        departure_time: None,
        stop_id: StopId::from(stop_id.to_string()),
        stop_sequence: seq,
        stop_headsign: None,
        pickup_type: None,
        drop_off_type: None,
        continuous_pickup: None,
        continuous_drop_off: None,
        shape_dist_traveled: None,
        timepoint: None,
        start_pickup_drop_off_window: None,
        end_pickup_drop_off_window: None,
        pickup_booking_rule_id: None,
        drop_off_booking_rule_id: None,
        mean_duration_factor: None,
        mean_duration_offset: None,
        safe_duration_factor: None,
        safe_duration_offset: None,
    }
}

fn count(errors: &[gapline_core::validation::ValidationError], severity: Severity) -> usize {
    errors.iter().filter(|e| e.severity == severity).count()
}

// ---------------------------------------------------------------------------
// Boarding Area (type 4) parent must be Stop/Platform (type 0)
// ---------------------------------------------------------------------------

#[test]
fn boarding_area_under_station_error() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop("STATION1", Some(LocationType::Station), None),
            make_stop("BA1", Some(LocationType::BoardingArea), Some("STATION1")),
        ],
        ..Default::default()
    };
    let errors = InvalidParentTypeRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "invalid_parent_type");
    assert_eq!(errors[0].severity, Severity::Error);
}

#[test]
fn boarding_area_under_stop_valid() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop("STOP1", Some(LocationType::StopOrPlatform), None),
            make_stop("BA1", Some(LocationType::BoardingArea), Some("STOP1")),
        ],
        ..Default::default()
    };
    let errors = InvalidParentTypeRule.validate(&feed);
    assert_eq!(errors.len(), 0);
}

#[test]
fn boarding_area_under_entrance_error() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop("ENT1", Some(LocationType::EntranceExit), None),
            make_stop("BA1", Some(LocationType::BoardingArea), Some("ENT1")),
        ],
        ..Default::default()
    };
    let errors = InvalidParentTypeRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

#[test]
fn boarding_area_under_node_error() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop("NODE1", Some(LocationType::GenericNode), None),
            make_stop("BA1", Some(LocationType::BoardingArea), Some("NODE1")),
        ],
        ..Default::default()
    };
    let errors = InvalidParentTypeRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

// ---------------------------------------------------------------------------
// Entrance/Exit (type 2) parent must be Station (type 1)
// ---------------------------------------------------------------------------

#[test]
fn entrance_under_stop_error() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop("STOP1", Some(LocationType::StopOrPlatform), None),
            make_stop("ENT1", Some(LocationType::EntranceExit), Some("STOP1")),
        ],
        ..Default::default()
    };
    let errors = InvalidParentTypeRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "invalid_parent_type");
}

#[test]
fn entrance_under_station_valid() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop("STATION1", Some(LocationType::Station), None),
            make_stop("ENT1", Some(LocationType::EntranceExit), Some("STATION1")),
        ],
        ..Default::default()
    };
    let errors = InvalidParentTypeRule.validate(&feed);
    assert_eq!(errors.len(), 0);
}

#[test]
fn entrance_under_boarding_area_error() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop("BA1", Some(LocationType::BoardingArea), None),
            make_stop("ENT1", Some(LocationType::EntranceExit), Some("BA1")),
        ],
        ..Default::default()
    };
    let errors = InvalidParentTypeRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

// ---------------------------------------------------------------------------
// Generic Node (type 3) parent must be Station (type 1)
// ---------------------------------------------------------------------------

#[test]
fn generic_node_under_stop_error() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop("STOP1", Some(LocationType::StopOrPlatform), None),
            make_stop("NODE1", Some(LocationType::GenericNode), Some("STOP1")),
        ],
        ..Default::default()
    };
    let errors = InvalidParentTypeRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

#[test]
fn generic_node_under_station_valid() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop("STATION1", Some(LocationType::Station), None),
            make_stop("NODE1", Some(LocationType::GenericNode), Some("STATION1")),
        ],
        ..Default::default()
    };
    let errors = InvalidParentTypeRule.validate(&feed);
    assert_eq!(errors.len(), 0);
}

// ---------------------------------------------------------------------------
// Unused station
// ---------------------------------------------------------------------------

#[test]
fn station_with_children_no_warning() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop("STATION1", Some(LocationType::Station), None),
            make_stop(
                "STOP1",
                Some(LocationType::StopOrPlatform),
                Some("STATION1"),
            ),
        ],
        ..Default::default()
    };
    let errors = UnusedStationRule.validate(&feed);
    assert_eq!(errors.len(), 0);
}

#[test]
fn station_without_children_warning() {
    let feed = GtfsFeed {
        stops: vec![make_stop("STATION1", Some(LocationType::Station), None)],
        ..Default::default()
    };
    let errors = UnusedStationRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "unused_station");
    assert_eq!(errors[0].severity, Severity::Warning);
}

#[test]
fn multiple_stations_mixed() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop("STATION1", Some(LocationType::Station), None),
            make_stop("STATION2", Some(LocationType::Station), None),
            make_stop(
                "STOP1",
                Some(LocationType::StopOrPlatform),
                Some("STATION1"),
            ),
        ],
        ..Default::default()
    };
    let errors = UnusedStationRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].value.as_deref(), Some("STATION2"));
}

// ---------------------------------------------------------------------------
// Unused stop
// ---------------------------------------------------------------------------

#[test]
fn stop_referenced_by_stop_time_no_warning() {
    let feed = GtfsFeed {
        stops: vec![make_stop("STOP1", Some(LocationType::StopOrPlatform), None)],
        stop_times: vec![make_stop_time("T1", "STOP1", 1)],
        ..Default::default()
    };
    let errors = UnusedStopRule.validate(&feed);
    assert_eq!(errors.len(), 0);
}

#[test]
fn stop_not_referenced_warning() {
    let feed = GtfsFeed {
        stops: vec![make_stop("STOP1", Some(LocationType::StopOrPlatform), None)],
        stop_times: vec![],
        ..Default::default()
    };
    let errors = UnusedStopRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "unused_stop");
    assert_eq!(errors[0].severity, Severity::Warning);
}

#[test]
fn station_not_in_stop_times_no_warning() {
    let feed = GtfsFeed {
        stops: vec![make_stop("STATION1", Some(LocationType::Station), None)],
        stop_times: vec![],
        ..Default::default()
    };
    let errors = UnusedStopRule.validate(&feed);
    assert_eq!(errors.len(), 0);
}

// ---------------------------------------------------------------------------
// Unknown route_type → WARNING
// ---------------------------------------------------------------------------

#[test]
fn route_type_standard_no_finding() {
    let feed = GtfsFeed {
        routes: vec![make_route("R1", RouteType::Bus)],
        ..Default::default()
    };
    let errors = RouteTypeSemanticsRule.validate(&feed);
    assert_eq!(errors.len(), 0);
}

#[test]
fn route_type_unknown_99_warning() {
    let feed = GtfsFeed {
        routes: vec![make_route("R1", RouteType::Unknown(99))],
        ..Default::default()
    };
    let errors = RouteTypeSemanticsRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "unknown_route_type");
    assert_eq!(errors[0].severity, Severity::Warning);
    assert_eq!(errors[0].value.as_deref(), Some("99"));
}

#[test]
fn route_type_unknown_9999_warning() {
    let feed = GtfsFeed {
        routes: vec![make_route("R1", RouteType::Unknown(9999))],
        ..Default::default()
    };
    let errors = RouteTypeSemanticsRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "unknown_route_type");
    assert_eq!(errors[0].severity, Severity::Warning);
}

// ---------------------------------------------------------------------------
// Extended route_type → INFO
// ---------------------------------------------------------------------------

#[test]
fn route_type_extended_200_info() {
    let feed = GtfsFeed {
        routes: vec![make_route("R1", RouteType::Hvt(200))],
        ..Default::default()
    };
    let errors = RouteTypeSemanticsRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "extended_route_type");
    assert_eq!(errors[0].severity, Severity::Info);
}

#[test]
fn route_type_extended_1702_info() {
    let feed = GtfsFeed {
        routes: vec![make_route("R1", RouteType::Hvt(1702))],
        ..Default::default()
    };
    let errors = RouteTypeSemanticsRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "extended_route_type");
    assert_eq!(errors[0].severity, Severity::Info);
}

#[test]
fn route_type_hvt_not_official_warning() {
    // 199 is in the Hvt range (100-1799) but not an official Extended Route Type.
    let feed = GtfsFeed {
        routes: vec![make_route("R1", RouteType::Hvt(199))],
        ..Default::default()
    };
    let errors = RouteTypeSemanticsRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "unknown_route_type");
    assert_eq!(errors[0].severity, Severity::Warning);
}

// ---------------------------------------------------------------------------
// Error context completeness
// ---------------------------------------------------------------------------

#[test]
fn error_includes_full_context() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop("STATION1", Some(LocationType::Station), None),
            make_stop("BA1", Some(LocationType::BoardingArea), Some("STATION1")),
        ],
        ..Default::default()
    };
    let errors = InvalidParentTypeRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    let e = &errors[0];
    assert_eq!(e.section, "7");
    assert_eq!(e.rule_id, "invalid_parent_type");
    assert_eq!(e.file_name.as_deref(), Some("stops.txt"));
    assert_eq!(e.line_number, Some(3)); // line 3 = index 1 + 2
    assert_eq!(e.field_name.as_deref(), Some("parent_station"));
    assert_eq!(e.value.as_deref(), Some("STATION1"));
}

// ---------------------------------------------------------------------------
// Integration - Valid full hierarchy produces no errors
// ---------------------------------------------------------------------------

#[test]
fn valid_full_hierarchy_no_errors() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop("STATION1", Some(LocationType::Station), None),
            make_stop(
                "STOP1",
                Some(LocationType::StopOrPlatform),
                Some("STATION1"),
            ),
            make_stop("BA1", Some(LocationType::BoardingArea), Some("STOP1")),
            make_stop("ENT1", Some(LocationType::EntranceExit), Some("STATION1")),
            make_stop("NODE1", Some(LocationType::GenericNode), Some("STATION1")),
        ],
        stop_times: vec![make_stop_time("T1", "STOP1", 1)],
        routes: vec![make_route("R1", RouteType::Bus)],
        ..Default::default()
    };

    let hierarchy_errors = InvalidParentTypeRule.validate(&feed);
    let station_errors = UnusedStationRule.validate(&feed);
    let stop_errors = UnusedStopRule.validate(&feed);
    let route_errors = RouteTypeSemanticsRule.validate(&feed);

    assert_eq!(count(&hierarchy_errors, Severity::Error), 0);
    assert_eq!(count(&station_errors, Severity::Warning), 0);
    assert_eq!(count(&stop_errors, Severity::Warning), 0);
    assert_eq!(route_errors.len(), 0);
}
