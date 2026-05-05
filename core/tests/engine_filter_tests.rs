//! Tests that [`ValidationEngine::new`] honors `disabled_rules` /
//! `enabled_rules` from the config.
//!
//! These are the only "engine-level" filtering tests; per-rule behaviour
//! lives in the rule-specific test files (`speed_validation_tests.rs`,
//! etc.). The strategy here is behavioural: build a feed that *would*
//! trigger `speed_validation`, run the engine twice (once with the rule
//! enabled, once with it disabled), and assert the rule's findings appear
//! in one report and not the other.

use std::sync::Arc;

use gapline_core::config::Config;
use gapline_core::models::*;
use gapline_core::validation::engine::ValidationEngine;

const M_PER_DEG_LAT: f64 = 111_194.93;
const BASE_LAT: f64 = 45.5017;
const BASE_LON: f64 = -73.5673;

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

fn make_stop_time(seq: u32, stop_id: &str, h: u32, m: u32) -> StopTime {
    StopTime {
        trip_id: TripId::from("T1"),
        arrival_time: Some(GtfsTime::from_hms(h, m, 0)),
        departure_time: Some(GtfsTime::from_hms(h, m, 0)),
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

/// Builds a feed with a single bus trip clearly above the default 150 km/h
/// limit (100 km in 30 min = 200 km/h).
fn fast_bus_feed() -> GtfsFeed {
    let dist_m = 100_000.0;
    let offset = dist_m / M_PER_DEG_LAT;
    GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        routes: vec![make_route("R1", RouteType::Bus)],
        trips: vec![make_trip("T1", "R1")],
        stop_times: vec![make_stop_time(1, "A", 8, 0), make_stop_time(2, "B", 8, 30)],
        ..Default::default()
    }
}

/// Ticket scenario 7: a rule listed in `validation.disabled_rules` does
/// not run, even when its conditions are clearly met.
#[test]
fn disabled_rules_skip_speed_validation() {
    let feed = fast_bus_feed();

    // Baseline: speed_validation enabled → at least one `unrealistic_speed`
    // finding is produced.
    let baseline = ValidationEngine::new(Arc::new(Config::default())).validate_feed(&feed, &[]);
    let baseline_speed_errors: Vec<_> = baseline
        .errors()
        .iter()
        .filter(|e| e.rule_id == "unrealistic_speed")
        .collect();
    assert!(
        !baseline_speed_errors.is_empty(),
        "baseline must produce speed errors so the disable test is meaningful"
    );

    // With `speed_validation` disabled, the same feed must produce zero
    // `unrealistic_speed` findings.
    let mut config = Config::default();
    config
        .validation
        .disabled_rules
        .push("speed_validation".into());
    let filtered = ValidationEngine::new(Arc::new(config)).validate_feed(&feed, &[]);
    let filtered_speed_errors: Vec<_> = filtered
        .errors()
        .iter()
        .filter(|e| e.rule_id == "unrealistic_speed")
        .collect();
    assert!(
        filtered_speed_errors.is_empty(),
        "disabled rule must not contribute findings, got: {filtered_speed_errors:?}"
    );
}

/// `pre_rules` and `post_rules` getters expose every registered rule
/// for introspection (used by `gapline rules list`). Both lists must be
/// non-empty when constructed from the default config.
#[test]
fn engine_exposes_pre_and_post_rules() {
    let engine = ValidationEngine::new(Arc::new(Config::default()));
    assert!(!engine.pre_rules().is_empty(), "no pre rules registered");
    assert!(!engine.post_rules().is_empty(), "no post rules registered");
    // The listing command relies on rule_id being callable through the
    // returned trait objects.
    let pre_id = engine.pre_rules()[0].rule_id();
    assert!(!pre_id.is_empty());
    let post_id = engine.post_rules()[0].rule_id();
    assert!(!post_id.is_empty());
}

/// `enabled_rules`, when set, restricts execution to the listed rule IDs.
/// All other rules are skipped.
#[test]
fn enabled_rules_whitelist_only_listed() {
    let feed = fast_bus_feed();

    let mut config = Config::default();
    config
        .validation
        .enabled_rules
        .push("speed_validation".into());
    let report = ValidationEngine::new(Arc::new(config)).validate_feed(&feed, &[]);

    // The whitelist contains exactly one rule. Every finding in the report
    // must come from `unrealistic_speed` (the error code emitted by
    // `speed_validation`). If the whitelist were ignored, dozens of
    // structural / referential rules would also fire on this minimal feed.
    for err in report.errors() {
        assert_eq!(
            err.rule_id, "unrealistic_speed",
            "unexpected rule survived the whitelist: {}",
            err.rule_id
        );
    }
}
