//! Tests for section 7.3 & 7.4 - shape geometry & stop-to-shape distances.

use std::time::Instant;

use gapline_core::geo::haversine_meters;
use gapline_core::models::*;
use gapline_core::validation::ValidationRule;
use gapline_core::validation::schedule_time_validation::distances::StopToShapeDistanceRule;
use gapline_core::validation::schedule_time_validation::shapes::ShapesGeometryRule;

// ---------------------------------------------------------------------------
// Constants & conversions
// ---------------------------------------------------------------------------

/// Meters per degree of latitude (constant with the spherical Earth model we
/// use - `EARTH_RADIUS_M` = 6 371 000 m ⟹ π·R/180 ≈ 111 194.93 m/°).
const M_PER_DEG_LAT: f64 = 111_194.93;

/// Offset `meters` along the latitude axis (no longitude drift).
fn lat_offset(meters: f64) -> f64 {
    meters / M_PER_DEG_LAT
}

// Real-world reference coordinates.
const MONTREAL_LAT: f64 = 45.5017;
const MONTREAL_LON: f64 = -73.5673;
const PARIS_LAT: f64 = 48.8566;
const PARIS_LON: f64 = 2.3522;

// ---------------------------------------------------------------------------
// Builders
// ---------------------------------------------------------------------------

fn shape_point(shape_id: &str, seq: u32, lat: f64, lon: f64) -> Shape {
    Shape {
        shape_id: ShapeId::from(shape_id),
        shape_pt_lat: Latitude(lat),
        shape_pt_lon: Longitude(lon),
        shape_pt_sequence: seq,
        shape_dist_traveled: None,
    }
}

fn shape_point_dist(shape_id: &str, seq: u32, lat: f64, lon: f64, dist: f64) -> Shape {
    Shape {
        shape_dist_traveled: Some(dist),
        ..shape_point(shape_id, seq, lat, lon)
    }
}

fn stop(id: &str, lat: f64, lon: f64) -> Stop {
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

fn trip(trip_id: &str, shape_id: Option<&str>) -> Trip {
    Trip {
        route_id: RouteId::from("R1"),
        service_id: ServiceId::from("SVC1"),
        trip_id: TripId::from(trip_id),
        trip_headsign: None,
        trip_short_name: None,
        direction_id: None,
        block_id: None,
        shape_id: shape_id.map(ShapeId::from),
        wheelchair_accessible: None,
        bikes_allowed: None,
    }
}

fn stop_time(trip_id: &str, seq: u32, stop_id: &str) -> StopTime {
    StopTime {
        trip_id: TripId::from(trip_id),
        arrival_time: None,
        departure_time: None,
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

// ---------------------------------------------------------------------------
// Haversine helper tests
// ---------------------------------------------------------------------------

#[test]
fn haversine_identical_points_is_zero() {
    let d = haversine_meters(MONTREAL_LAT, MONTREAL_LON, MONTREAL_LAT, MONTREAL_LON);
    assert!(d.abs() < 1e-9, "expected 0, got {d}");
}

#[test]
fn haversine_paris_montreal_about_5500km() {
    let d = haversine_meters(PARIS_LAT, PARIS_LON, MONTREAL_LAT, MONTREAL_LON);
    assert!(
        (5_480_000.0..=5_530_000.0).contains(&d),
        "expected ~5505 km, got {d}"
    );
}

#[test]
fn haversine_one_degree_latitude_is_111km() {
    let d = haversine_meters(45.0, 0.0, 46.0, 0.0);
    assert!((d - M_PER_DEG_LAT).abs() < 100.0, "got {d}");
}

#[test]
fn haversine_is_symmetric() {
    let a = haversine_meters(PARIS_LAT, PARIS_LON, MONTREAL_LAT, MONTREAL_LON);
    let b = haversine_meters(MONTREAL_LAT, MONTREAL_LON, PARIS_LAT, PARIS_LON);
    assert!((a - b).abs() < 1e-6);
}

#[test]
fn haversine_tiny_offset_matches_expected() {
    // 5m offset along latitude.
    let d = haversine_meters(45.0, 0.0, 45.0 + lat_offset(5.0), 0.0);
    assert!((d - 5.0).abs() < 0.01, "got {d}");
}

// ---------------------------------------------------------------------------
// ShapesGeometryRule
// ---------------------------------------------------------------------------

fn rule_shapes() -> ShapesGeometryRule {
    ShapesGeometryRule::new(1.11, 0.5)
}

/// Case #1: 10 well-spaced shape points form a coherent trace → no errors.
#[test]
fn valid_shape_produces_no_errors() {
    // 10 points, each 50m apart along latitude, starting at Montréal.
    let points: Vec<Shape> = (0..10)
        .map(|i| {
            shape_point(
                "SH1",
                i + 1,
                MONTREAL_LAT + lat_offset(50.0 * f64::from(i)),
                MONTREAL_LON,
            )
        })
        .collect();
    let feed = GtfsFeed {
        shapes: points,
        ..Default::default()
    };
    assert!(rule_shapes().validate(&feed).is_empty());
}

/// Case #2: two consecutive points 0.5m apart → 1 WARNING `shape_points_too_close`.
#[test]
fn consecutive_points_too_close_are_flagged() {
    let feed = GtfsFeed {
        shapes: vec![
            shape_point("SH1", 1, MONTREAL_LAT, MONTREAL_LON),
            shape_point("SH1", 2, MONTREAL_LAT + lat_offset(0.5), MONTREAL_LON),
        ],
        ..Default::default()
    };
    let errors = rule_shapes().validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "shape_points_too_close")
            .count(),
        1
    );
}

/// Case #3: points exactly at the threshold → 0 warnings (comparison is strictly `<`).
///
/// Two points are placed with a latitude offset of 1.11m. The rule's minimum
/// distance threshold is set to the Haversine distance those points actually
/// resolve to, so the boundary comparison is exercised directly without
/// floating-point roundtrip surprises.
#[test]
fn points_at_exact_threshold_pass() {
    let lat_a = MONTREAL_LAT;
    let lat_b = MONTREAL_LAT + lat_offset(1.11);
    let exact = haversine_meters(lat_a, MONTREAL_LON, lat_b, MONTREAL_LON);
    let feed = GtfsFeed {
        shapes: vec![
            shape_point("SH1", 1, lat_a, MONTREAL_LON),
            shape_point("SH1", 2, lat_b, MONTREAL_LON),
        ],
        ..Default::default()
    };
    let rule = ShapesGeometryRule::new(exact, 0.5);
    let errors = rule.validate(&feed);
    assert!(
        errors.iter().all(|e| e.rule_id != "shape_points_too_close"),
        "distance at exact threshold ({exact}) must not trigger warning"
    );
}

/// Case #4: shape with a single point → 1 WARNING `degenerate_shape`.
#[test]
fn single_point_shape_is_degenerate() {
    let feed = GtfsFeed {
        shapes: vec![shape_point("SH1", 1, MONTREAL_LAT, MONTREAL_LON)],
        ..Default::default()
    };
    let errors = rule_shapes().validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "degenerate_shape")
            .count(),
        1
    );
    let err = errors
        .iter()
        .find(|e| e.rule_id == "degenerate_shape")
        .unwrap();
    assert_eq!(err.section, "7");
    assert_eq!(err.file_name.as_deref(), Some("shapes.txt"));
    assert_eq!(err.line_number, Some(2));
}

/// Case #9: one segment's declared `shape_dist_traveled` diverges from the
/// shape's median ratio → 1 aggregated WARNING for the shape.
#[test]
fn incoherent_shape_dist_traveled_is_flagged() {
    // 4 coherent segments in meters (ratio=1), then one wildly off segment.
    // Median ratio across the 4 segments is 1.0. The bad segment's ratio is
    // 50/100 * big = out of tolerance.
    let feed = GtfsFeed {
        shapes: vec![
            shape_point_dist("SH1", 1, MONTREAL_LAT, MONTREAL_LON, 0.0),
            shape_point_dist(
                "SH1",
                2,
                MONTREAL_LAT + lat_offset(100.0),
                MONTREAL_LON,
                100.0,
            ),
            shape_point_dist(
                "SH1",
                3,
                MONTREAL_LAT + lat_offset(200.0),
                MONTREAL_LON,
                200.0,
            ),
            shape_point_dist(
                "SH1",
                4,
                MONTREAL_LAT + lat_offset(300.0),
                MONTREAL_LON,
                300.0,
            ),
            // Last segment: 100m haversine but declared jumps by 5000 → ratio=50.
            shape_point_dist(
                "SH1",
                5,
                MONTREAL_LAT + lat_offset(400.0),
                MONTREAL_LON,
                5300.0,
            ),
        ],
        ..Default::default()
    };
    let errors = rule_shapes().validate(&feed);
    let hits: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "shape_dist_traveled_incoherent")
        .collect();
    assert_eq!(hits.len(), 1, "expected 1 aggregated warning per shape");
    // Message should reference the 4-segment median (meters) and mention the count.
    assert!(hits[0].message.contains("1/4"), "got: {}", hits[0].message);
    assert!(
        hits[0].message.contains("meters"),
        "unit detection failed: {}",
        hits[0].message
    );
}

/// Coherent `shape_dist_traveled` increments should not be flagged.
#[test]
fn coherent_shape_dist_traveled_passes() {
    let feed = GtfsFeed {
        shapes: vec![
            shape_point_dist("SH1", 1, MONTREAL_LAT, MONTREAL_LON, 0.0),
            shape_point_dist(
                "SH1",
                2,
                MONTREAL_LAT + lat_offset(100.0),
                MONTREAL_LON,
                100.0,
            ),
            shape_point_dist(
                "SH1",
                3,
                MONTREAL_LAT + lat_offset(200.0),
                MONTREAL_LON,
                200.0,
            ),
        ],
        ..Default::default()
    };
    assert!(
        rule_shapes()
            .validate(&feed)
            .iter()
            .all(|e| e.rule_id != "shape_dist_traveled_incoherent")
    );
}

/// Two exactly-coincident consecutive points → 1 `duplicate_shape_point`,
/// 0 `shape_points_too_close` (duplicates are a separate finding).
#[test]
fn duplicate_shape_points_are_flagged_as_duplicate() {
    let feed = GtfsFeed {
        shapes: vec![
            shape_point("SH1", 1, MONTREAL_LAT, MONTREAL_LON),
            shape_point("SH1", 2, MONTREAL_LAT, MONTREAL_LON),
        ],
        ..Default::default()
    };
    let errors = rule_shapes().validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "duplicate_shape_point")
            .count(),
        1
    );
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "shape_points_too_close")
            .count(),
        0
    );
}

/// Five consecutive close pairs in one shape → exactly 1 aggregated warning
/// (count=5), not 5 warnings.
#[test]
fn multiple_close_pairs_in_one_shape_produce_single_aggregated_warning() {
    // 6 points, each 0.5m apart → 5 pairs all under the 1.11m threshold.
    let points: Vec<Shape> = (0..6)
        .map(|i| {
            shape_point(
                "SH1",
                i + 1,
                MONTREAL_LAT + lat_offset(0.5 * f64::from(i)),
                MONTREAL_LON,
            )
        })
        .collect();
    let feed = GtfsFeed {
        shapes: points,
        ..Default::default()
    };
    let errors = rule_shapes().validate(&feed);
    let hits: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "shape_points_too_close")
        .collect();
    assert_eq!(hits.len(), 1, "expected 1 aggregated warning");
    assert!(
        hits[0].message.contains("5 consecutive"),
        "count missing in message: {}",
        hits[0].message
    );
}

/// Feed in kilometers (like tadao): 3 points at 100m apart with
/// `shape_dist_traveled` expressed in km → no incoherence warning.
#[test]
fn kilometer_unit_is_detected_as_coherent() {
    let feed = GtfsFeed {
        shapes: vec![
            shape_point_dist("SH1", 1, MONTREAL_LAT, MONTREAL_LON, 0.0),
            shape_point_dist(
                "SH1",
                2,
                MONTREAL_LAT + lat_offset(100.0),
                MONTREAL_LON,
                0.1,
            ),
            shape_point_dist(
                "SH1",
                3,
                MONTREAL_LAT + lat_offset(200.0),
                MONTREAL_LON,
                0.2,
            ),
        ],
        ..Default::default()
    };
    assert!(
        rule_shapes()
            .validate(&feed)
            .iter()
            .all(|e| e.rule_id != "shape_dist_traveled_incoherent")
    );
}

/// Two separate shapes, one degenerate, one valid → only the bad one is flagged.
#[test]
fn degenerate_shape_does_not_affect_siblings() {
    let feed = GtfsFeed {
        shapes: vec![
            shape_point("SH_OK", 1, PARIS_LAT, PARIS_LON),
            shape_point("SH_OK", 2, PARIS_LAT + lat_offset(100.0), PARIS_LON),
            shape_point("SH_BAD", 1, MONTREAL_LAT, MONTREAL_LON),
        ],
        ..Default::default()
    };
    let errors = rule_shapes().validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "degenerate_shape");
    assert_eq!(errors[0].value.as_deref(), Some("SH_BAD"));
}

// ---------------------------------------------------------------------------
// StopToShapeDistanceRule
// ---------------------------------------------------------------------------

fn rule_distances(max_m: f64) -> StopToShapeDistanceRule {
    StopToShapeDistanceRule::new(max_m)
}

/// Case #7: stop 250m from nearest shape point, threshold 100m → 1 WARNING.
#[test]
fn stop_too_far_from_shape_is_flagged() {
    let shape_lat = MONTREAL_LAT;
    let shape_lon = MONTREAL_LON;
    let far_stop_lat = shape_lat + lat_offset(250.0);
    let feed = GtfsFeed {
        shapes: vec![
            shape_point("SH1", 1, shape_lat, shape_lon),
            shape_point("SH1", 2, shape_lat + lat_offset(10.0), shape_lon),
        ],
        stops: vec![stop("S1", far_stop_lat, shape_lon)],
        trips: vec![trip("T1", Some("SH1"))],
        stop_times: vec![stop_time("T1", 1, "S1")],
        ..Default::default()
    };
    let errors = rule_distances(100.0).validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "stop_too_far_from_shape");
    assert_eq!(errors[0].section, "7");
    assert_eq!(errors[0].file_name.as_deref(), Some("stop_times.txt"));
    // Message should carry the distance.
    assert!(errors[0].message.contains("240") || errors[0].message.contains("250"));
}

/// Case #8: stop 80m from shape, threshold 100m → no warning.
#[test]
fn stop_within_threshold_passes() {
    let feed = GtfsFeed {
        shapes: vec![
            shape_point("SH1", 1, MONTREAL_LAT, MONTREAL_LON),
            shape_point("SH1", 2, MONTREAL_LAT + lat_offset(10.0), MONTREAL_LON),
        ],
        stops: vec![stop("S1", MONTREAL_LAT + lat_offset(80.0), MONTREAL_LON)],
        trips: vec![trip("T1", Some("SH1"))],
        stop_times: vec![stop_time("T1", 1, "S1")],
        ..Default::default()
    };
    assert!(rule_distances(100.0).validate(&feed).is_empty());
}

/// Case #10: trip without a `shape_id` → no stop-to-shape checks.
#[test]
fn trip_without_shape_is_skipped() {
    let feed = GtfsFeed {
        shapes: vec![],
        stops: vec![stop("S1", MONTREAL_LAT + lat_offset(9_999.0), MONTREAL_LON)],
        trips: vec![trip("T1", None)],
        stop_times: vec![stop_time("T1", 1, "S1")],
        ..Default::default()
    };
    assert!(rule_distances(100.0).validate(&feed).is_empty());
}

/// Trip whose `shape_id` is `Some("")` (empty) → treated as no shape.
#[test]
fn trip_with_empty_shape_id_is_skipped() {
    let feed = GtfsFeed {
        shapes: vec![],
        stops: vec![stop("S1", MONTREAL_LAT + lat_offset(9_999.0), MONTREAL_LON)],
        trips: vec![trip("T1", Some(""))],
        stop_times: vec![stop_time("T1", 1, "S1")],
        ..Default::default()
    };
    assert!(rule_distances(100.0).validate(&feed).is_empty());
}

/// Case #11: custom threshold of 500m, stop at 250m → no warning.
#[test]
fn custom_threshold_suppresses_warning() {
    let feed = GtfsFeed {
        shapes: vec![
            shape_point("SH1", 1, MONTREAL_LAT, MONTREAL_LON),
            shape_point("SH1", 2, MONTREAL_LAT + lat_offset(10.0), MONTREAL_LON),
        ],
        stops: vec![stop("S1", MONTREAL_LAT + lat_offset(250.0), MONTREAL_LON)],
        trips: vec![trip("T1", Some("SH1"))],
        stop_times: vec![stop_time("T1", 1, "S1")],
        ..Default::default()
    };
    assert!(rule_distances(500.0).validate(&feed).is_empty());
}

/// Stop without coordinates is skipped (no false positive).
#[test]
fn stop_without_coordinates_is_skipped() {
    let mut s = stop("S1", 0.0, 0.0);
    s.stop_lat = None;
    s.stop_lon = None;
    let feed = GtfsFeed {
        shapes: vec![
            shape_point("SH1", 1, MONTREAL_LAT, MONTREAL_LON),
            shape_point("SH1", 2, MONTREAL_LAT + lat_offset(10.0), MONTREAL_LON),
        ],
        stops: vec![s],
        trips: vec![trip("T1", Some("SH1"))],
        stop_times: vec![stop_time("T1", 1, "S1")],
        ..Default::default()
    };
    assert!(rule_distances(100.0).validate(&feed).is_empty());
}

/// Case #12: 10 000 shape points × 100 stops completes in under 3s.
#[test]
fn performance_10k_points_100_stops() {
    let shapes: Vec<Shape> = (0..10_000)
        .map(|i| {
            shape_point(
                "SH1",
                i + 1,
                MONTREAL_LAT + lat_offset(5.0 * f64::from(i)),
                MONTREAL_LON,
            )
        })
        .collect();
    let stops: Vec<Stop> = (0..100)
        .map(|i| {
            stop(
                &format!("S{i}"),
                MONTREAL_LAT + lat_offset(500.0 * f64::from(i) + 50.0),
                MONTREAL_LON,
            )
        })
        .collect();
    let stop_times: Vec<StopTime> = (0..100)
        .map(|i| stop_time("T1", u32::try_from(i).unwrap() + 1, &format!("S{i}")))
        .collect();
    let feed = GtfsFeed {
        shapes,
        stops,
        trips: vec![trip("T1", Some("SH1"))],
        stop_times,
        ..Default::default()
    };

    let start = Instant::now();
    let _errors = rule_distances(100.0).validate(&feed);
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs_f64() < 3.0,
        "validation took {:.2}s",
        elapsed.as_secs_f64()
    );
}
