//! Tests for section 7.11 — speed validation.

use gapline_core::models::*;
use gapline_core::validation::schedule_time_validation::SpeedThresholds;
use gapline_core::validation::schedule_time_validation::speed::SpeedValidationRule;
use gapline_core::validation::{Severity, ValidationRule};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Meters per degree of latitude (spherical Earth).
const M_PER_DEG_LAT: f64 = 111_194.93;

fn lat_offset(meters: f64) -> f64 {
    meters / M_PER_DEG_LAT
}

const BASE_LAT: f64 = 45.5017;
const BASE_LON: f64 = -73.5673;

// ---------------------------------------------------------------------------
// Builders
// ---------------------------------------------------------------------------

fn make_stop(id: &str, lat: f64, lon: f64) -> Stop {
    Stop {
        stop_id: StopId::from(id),
        stop_code: None,
        stop_name: None,
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(lat)),
        stop_lon: Some(Longitude(lon)),
        zone_id: None,
        stop_url: None,
        location_type: None,
        parent_station: None,
        stop_timezone: None,
        wheelchair_boarding: None,
        level_id: None,
        platform_code: None,
    }
}

fn make_stop_no_coords(id: &str) -> Stop {
    Stop {
        stop_lat: None,
        stop_lon: None,
        ..make_stop(id, 0.0, 0.0)
    }
}

fn make_route(id: &str, route_type: RouteType) -> Route {
    Route {
        route_id: RouteId::from(id),
        agency_id: None,
        route_short_name: None,
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

fn make_trip(trip_id: &str, route_id: &str) -> Trip {
    Trip {
        route_id: RouteId::from(route_id),
        service_id: ServiceId::from("SVC1"),
        trip_id: TripId::from(trip_id),
        trip_headsign: None,
        trip_short_name: None,
        direction_id: None,
        block_id: None,
        shape_id: None,
        wheelchair_accessible: None,
        bikes_allowed: None,
    }
}

fn make_stop_time(
    trip_id: &str,
    seq: u32,
    stop_id: &str,
    arr: Option<GtfsTime>,
    dep: Option<GtfsTime>,
) -> StopTime {
    StopTime {
        trip_id: TripId::from(trip_id),
        arrival_time: arr,
        departure_time: dep,
        stop_id: StopId::from(stop_id),
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

fn t(h: u32, m: u32, s: u32) -> GtfsTime {
    GtfsTime::from_hms(h, m, s)
}

fn default_thresholds() -> SpeedThresholds {
    SpeedThresholds {
        tram_kmh: 150.0,
        subway_kmh: 150.0,
        rail_kmh: 500.0,
        bus_kmh: 150.0,
        ferry_kmh: 150.0,
        cable_tram_kmh: 30.0,
        aerial_lift_kmh: 50.0,
        funicular_kmh: 50.0,
        trolleybus_kmh: 150.0,
        monorail_kmh: 150.0,
        default_kmh: 150.0,
    }
}

fn rule() -> SpeedValidationRule {
    SpeedValidationRule::new(default_thresholds())
}

// ---------------------------------------------------------------------------
// CA8: unrealistic_speed
// ---------------------------------------------------------------------------

#[test]
fn bus_above_limit() {
    // Bus: 100 km in 30 min = 200 km/h (limit 150)
    let dist_m = 100_000.0;
    let offset = lat_offset(dist_m);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        routes: vec![make_route("R1", RouteType::Bus)],
        trips: vec![make_trip("T1", "R1")],
        stop_times: vec![
            make_stop_time("T1", 1, "A", Some(t(8, 0, 0)), Some(t(8, 0, 0))),
            make_stop_time("T1", 2, "B", Some(t(8, 30, 0)), Some(t(8, 30, 0))),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "unrealistic_speed")
        .collect();
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].severity, Severity::Warning);
}

#[test]
fn train_within_limit() {
    // Train: 200 km in 1h = 200 km/h (limit 500)
    let dist_m = 200_000.0;
    let offset = lat_offset(dist_m);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        routes: vec![make_route("R1", RouteType::Rail)],
        trips: vec![make_trip("T1", "R1")],
        stop_times: vec![
            make_stop_time("T1", 1, "A", Some(t(8, 0, 0)), Some(t(8, 0, 0))),
            make_stop_time("T1", 2, "B", Some(t(9, 0, 0)), Some(t(9, 0, 0))),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "unrealistic_speed")
        .collect();
    assert!(matched.is_empty());
}

#[test]
fn subway_above_limit() {
    // Subway: 100 km in 30 min = 200 km/h (limit 150)
    let dist_m = 100_000.0;
    let offset = lat_offset(dist_m);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        routes: vec![make_route("R1", RouteType::Subway)],
        trips: vec![make_trip("T1", "R1")],
        stop_times: vec![
            make_stop_time("T1", 1, "A", Some(t(8, 0, 0)), Some(t(8, 0, 0))),
            make_stop_time("T1", 2, "B", Some(t(8, 30, 0)), Some(t(8, 30, 0))),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "unrealistic_speed")
        .collect();
    assert_eq!(matched.len(), 1);
}

#[test]
fn ferry_within_limit() {
    // Ferry: 50 km in 1h = 50 km/h (limit 150)
    let dist_m = 50_000.0;
    let offset = lat_offset(dist_m);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        routes: vec![make_route("R1", RouteType::Ferry)],
        trips: vec![make_trip("T1", "R1")],
        stop_times: vec![
            make_stop_time("T1", 1, "A", Some(t(8, 0, 0)), Some(t(8, 0, 0))),
            make_stop_time("T1", 2, "B", Some(t(9, 0, 0)), Some(t(9, 0, 0))),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "unrealistic_speed")
        .collect();
    assert!(matched.is_empty());
}

// ---------------------------------------------------------------------------
// CA9: zero_speed
// ---------------------------------------------------------------------------

#[test]
fn zero_speed_different_coords() {
    // 5 km apart, same arrival time
    let dist_m = 5_000.0;
    let offset = lat_offset(dist_m);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        routes: vec![make_route("R1", RouteType::Bus)],
        trips: vec![make_trip("T1", "R1")],
        stop_times: vec![
            make_stop_time("T1", 1, "A", Some(t(8, 0, 0)), Some(t(8, 0, 0))),
            make_stop_time("T1", 2, "B", Some(t(8, 0, 0)), Some(t(8, 0, 0))),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "zero_speed")
        .collect();
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].severity, Severity::Warning);
}

#[test]
fn zero_speed_same_coords_ok() {
    // Same coordinates, same time — not flagged (distance < 1m)
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT, BASE_LON),
        ],
        routes: vec![make_route("R1", RouteType::Bus)],
        trips: vec![make_trip("T1", "R1")],
        stop_times: vec![
            make_stop_time("T1", 1, "A", Some(t(8, 0, 0)), Some(t(8, 0, 0))),
            make_stop_time("T1", 2, "B", Some(t(8, 0, 0)), Some(t(8, 0, 0))),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "zero_speed")
        .collect();
    assert!(matched.is_empty());
}

// ---------------------------------------------------------------------------
// CA10: skip conditions
// ---------------------------------------------------------------------------

#[test]
fn missing_arrival_time_skipped() {
    let dist_m = 100_000.0;
    let offset = lat_offset(dist_m);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        routes: vec![make_route("R1", RouteType::Bus)],
        trips: vec![make_trip("T1", "R1")],
        stop_times: vec![
            make_stop_time("T1", 1, "A", Some(t(8, 0, 0)), Some(t(8, 0, 0))),
            make_stop_time("T1", 2, "B", None, None),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    assert!(errors.is_empty());
}

#[test]
fn missing_departure_time_skipped() {
    let dist_m = 100_000.0;
    let offset = lat_offset(dist_m);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        routes: vec![make_route("R1", RouteType::Bus)],
        trips: vec![make_trip("T1", "R1")],
        stop_times: vec![
            make_stop_time("T1", 1, "A", Some(t(8, 0, 0)), None),
            make_stop_time("T1", 2, "B", Some(t(8, 30, 0)), Some(t(8, 30, 0))),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    assert!(errors.is_empty());
}

#[test]
fn missing_coords_skipped() {
    let feed = GtfsFeed {
        stops: vec![make_stop_no_coords("A"), make_stop_no_coords("B")],
        routes: vec![make_route("R1", RouteType::Bus)],
        trips: vec![make_trip("T1", "R1")],
        stop_times: vec![
            make_stop_time("T1", 1, "A", Some(t(8, 0, 0)), Some(t(8, 0, 0))),
            make_stop_time("T1", 2, "B", Some(t(8, 30, 0)), Some(t(8, 30, 0))),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// CA11: custom speed limits
// ---------------------------------------------------------------------------

#[test]
fn custom_speed_limit() {
    // Bus limit raised to 200 km/h, bus at 180 km/h → no warning
    let dist_m = 90_000.0; // 90 km in 30 min = 180 km/h
    let offset = lat_offset(dist_m);
    let thresholds = SpeedThresholds {
        bus_kmh: 200.0,
        ..default_thresholds()
    };
    let custom_rule = SpeedValidationRule::new(thresholds);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        routes: vec![make_route("R1", RouteType::Bus)],
        trips: vec![make_trip("T1", "R1")],
        stop_times: vec![
            make_stop_time("T1", 1, "A", Some(t(8, 0, 0)), Some(t(8, 0, 0))),
            make_stop_time("T1", 2, "B", Some(t(8, 30, 0)), Some(t(8, 30, 0))),
        ],
        ..Default::default()
    };
    let errors = custom_rule.validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "unrealistic_speed")
        .collect();
    assert!(matched.is_empty());
}

// ---------------------------------------------------------------------------
// Multiple trips
// ---------------------------------------------------------------------------

#[test]
fn multiple_trips_independent() {
    let fast_dist = 100_000.0; // 100 km in 30 min = 200 km/h (speeding)
    let slow_dist = 10_000.0; // 10 km in 30 min = 20 km/h (ok)
    let fast_offset = lat_offset(fast_dist);
    let slow_offset = lat_offset(slow_dist);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + fast_offset, BASE_LON),
            make_stop("C", BASE_LAT, BASE_LON),
            make_stop("D", BASE_LAT + slow_offset, BASE_LON),
        ],
        routes: vec![make_route("R1", RouteType::Bus)],
        trips: vec![make_trip("T1", "R1"), make_trip("T2", "R1")],
        stop_times: vec![
            make_stop_time("T1", 1, "A", Some(t(8, 0, 0)), Some(t(8, 0, 0))),
            make_stop_time("T1", 2, "B", Some(t(8, 30, 0)), Some(t(8, 30, 0))),
            make_stop_time("T2", 1, "C", Some(t(8, 0, 0)), Some(t(8, 0, 0))),
            make_stop_time("T2", 2, "D", Some(t(8, 30, 0)), Some(t(8, 30, 0))),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "unrealistic_speed")
        .collect();
    assert_eq!(matched.len(), 1, "only 1 trip should be speeding");
}

// ---------------------------------------------------------------------------
// CA13: error context completeness
// ---------------------------------------------------------------------------

#[test]
fn error_context_complete() {
    let dist_m = 100_000.0;
    let offset = lat_offset(dist_m);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        routes: vec![make_route("R1", RouteType::Bus)],
        trips: vec![make_trip("T1", "R1")],
        stop_times: vec![
            make_stop_time("T1", 1, "A", Some(t(8, 0, 0)), Some(t(8, 0, 0))),
            make_stop_time("T1", 2, "B", Some(t(8, 30, 0)), Some(t(8, 30, 0))),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let err = errors
        .iter()
        .find(|e| e.rule_id == "unrealistic_speed")
        .expect("expected error");
    assert_eq!(err.section, "7");
    assert_eq!(err.severity, Severity::Warning);
    assert_eq!(err.file_name.as_deref(), Some("stop_times.txt"));
    assert!(err.line_number.is_some());
    assert_eq!(err.field_name.as_deref(), Some("arrival_time"));
    assert!(err.value.is_some());
    assert!(err.message.contains("km/h"));
}
