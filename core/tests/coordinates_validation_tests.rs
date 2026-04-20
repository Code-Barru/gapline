//! Tests for section 7.13 — coordinate sanity validation.

use gapline_core::models::*;
use gapline_core::validation::ValidationRule;
use gapline_core::validation::schedule_time_validation::coordinates::{
    CoordinatesNearOriginRule, CoordinatesNearPoleRule, DuplicateCoordinatesRule,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_stop(stop_id: &str, lat: f64, lon: f64) -> Stop {
    Stop {
        stop_id: StopId::from(stop_id),
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

// ---------------------------------------------------------------------------
// Test 1: stop near origin
// ---------------------------------------------------------------------------

#[test]
fn stop_near_origin() {
    let feed = GtfsFeed {
        stops: vec![make_stop("S1", 0.005, 0.003)],
        ..Default::default()
    };
    let rule = CoordinatesNearOriginRule::new(1000.0);
    let errors = rule.validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "coordinates_near_origin")
            .count(),
        1,
    );
}

// ---------------------------------------------------------------------------
// Test 2: stop far from origin (Montreal)
// ---------------------------------------------------------------------------

#[test]
fn stop_far_from_origin() {
    let feed = GtfsFeed {
        stops: vec![make_stop("S1", 45.5017, -73.5673)],
        ..Default::default()
    };
    let rule = CoordinatesNearOriginRule::new(1000.0);
    let errors = rule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// Test 3: stop near north pole
// ---------------------------------------------------------------------------

#[test]
fn stop_near_north_pole() {
    let feed = GtfsFeed {
        stops: vec![make_stop("S1", 89.999, 10.0)],
        ..Default::default()
    };
    let rule = CoordinatesNearPoleRule::new(1000.0);
    let errors = rule.validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "coordinates_near_pole")
            .count(),
        1,
    );
}

// ---------------------------------------------------------------------------
// Test 4: stop near south pole
// ---------------------------------------------------------------------------

#[test]
fn stop_near_south_pole() {
    let feed = GtfsFeed {
        stops: vec![make_stop("S1", -89.999, 10.0)],
        ..Default::default()
    };
    let rule = CoordinatesNearPoleRule::new(1000.0);
    let errors = rule.validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "coordinates_near_pole")
            .count(),
        1,
    );
}

// ---------------------------------------------------------------------------
// Test 5: duplicate coordinates, 2 stops
// ---------------------------------------------------------------------------

#[test]
fn duplicate_coordinates_two_stops() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop("S1", 45.5017, -73.5673),
            make_stop("S2", 45.5017, -73.5673),
        ],
        ..Default::default()
    };
    let errors = DuplicateCoordinatesRule.validate(&feed);
    let dupes: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "duplicate_coordinates")
        .collect();
    assert_eq!(dupes.len(), 1);
    // The message should mention both stops.
    assert!(dupes[0].message.contains("S1"));
    assert!(dupes[0].message.contains("S2"));
}

// ---------------------------------------------------------------------------
// Test 6: unique coordinates — no warning
// ---------------------------------------------------------------------------

#[test]
fn unique_coordinates_no_warning() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop("S1", 45.5017, -73.5673),
            make_stop("S2", 48.8566, 2.3522),
        ],
        ..Default::default()
    };
    let errors = DuplicateCoordinatesRule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// Test 7: stop without lat/lon is skipped
// ---------------------------------------------------------------------------

#[test]
fn stop_without_coords_skipped() {
    let feed = GtfsFeed {
        stops: vec![Stop {
            stop_id: StopId::from("S1"),
            stop_code: None,
            stop_name: None,
            tts_stop_name: None,
            stop_desc: None,
            stop_lat: None,
            stop_lon: None,
            zone_id: None,
            stop_url: None,
            location_type: None,
            parent_station: None,
            stop_timezone: None,
            wheelchair_boarding: None,
            level_id: None,
            platform_code: None,
        }],
        ..Default::default()
    };
    let origin_errors = CoordinatesNearOriginRule::new(1000.0).validate(&feed);
    let pole_errors = CoordinatesNearPoleRule::new(1000.0).validate(&feed);
    let dupe_errors = DuplicateCoordinatesRule.validate(&feed);
    assert!(origin_errors.is_empty());
    assert!(pole_errors.is_empty());
    assert!(dupe_errors.is_empty());
}

// ---------------------------------------------------------------------------
// Test 8: error context completeness
// ---------------------------------------------------------------------------

#[test]
fn error_context_completeness() {
    let feed = GtfsFeed {
        stops: vec![make_stop("S1", 0.005, 0.003)],
        ..Default::default()
    };
    let errors = CoordinatesNearOriginRule::new(1000.0).validate(&feed);
    let e = errors
        .iter()
        .find(|e| e.rule_id == "coordinates_near_origin")
        .expect("should find origin warning");
    assert_eq!(e.section, "7");
    assert_eq!(e.file_name.as_deref(), Some("stops.txt"));
    assert!(e.line_number.is_some());
    assert_eq!(e.field_name.as_deref(), Some("stop_lat"));
    assert!(e.value.is_some());
}
