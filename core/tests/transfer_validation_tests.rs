//! Tests for section 7.9 — transfer validation.

use headway_core::models::*;
use headway_core::validation::schedule_time_validation::TransferThresholds;
use headway_core::validation::schedule_time_validation::transfers::TransferValidationRule;
use headway_core::validation::{Severity, ValidationRule};

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

fn make_transfer(
    from: Option<&str>,
    to: Option<&str>,
    transfer_type: TransferType,
    min_time: Option<u32>,
) -> Transfer {
    Transfer {
        from_stop_id: from.map(StopId::from),
        to_stop_id: to.map(StopId::from),
        from_route_id: None,
        to_route_id: None,
        from_trip_id: None,
        to_trip_id: None,
        transfer_type,
        min_transfer_time: min_time,
    }
}

fn default_thresholds() -> TransferThresholds {
    TransferThresholds {
        max_transfer_distance_m: 10_000.0,
        transfer_distance_warning_m: 2_000.0,
    }
}

fn rule() -> TransferValidationRule {
    TransferValidationRule::new(default_thresholds())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn valid_transfer_no_errors() {
    let offset = lat_offset(500.0);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        transfers: vec![make_transfer(
            Some("A"),
            Some("B"),
            TransferType::Recommended,
            None,
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    assert!(errors.is_empty(), "expected 0 errors, got {errors:?}");
}

#[test]
fn transfer_distance_too_large() {
    let offset = lat_offset(15_000.0);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        transfers: vec![make_transfer(
            Some("A"),
            Some("B"),
            TransferType::Recommended,
            None,
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "transfer_distance_too_large")
        .collect();
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].severity, Severity::Error);
}

#[test]
fn transfer_distance_suspicious() {
    let offset = lat_offset(3_000.0);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        transfers: vec![make_transfer(
            Some("A"),
            Some("B"),
            TransferType::Recommended,
            None,
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "transfer_distance_suspicious")
        .collect();
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].severity, Severity::Warning);
}

#[test]
fn error_takes_precedence_over_warning() {
    let offset = lat_offset(15_000.0);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        transfers: vec![make_transfer(
            Some("A"),
            Some("B"),
            TransferType::Recommended,
            None,
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let warnings: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "transfer_distance_suspicious")
        .collect();
    assert!(
        warnings.is_empty(),
        "should not emit WARNING when ERROR is emitted"
    );
}

#[test]
fn self_transfer_warning() {
    let feed = GtfsFeed {
        stops: vec![make_stop("S01", BASE_LAT, BASE_LON)],
        transfers: vec![make_transfer(
            Some("S01"),
            Some("S01"),
            TransferType::Recommended,
            None,
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "self_transfer")
        .collect();
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].severity, Severity::Warning);
}

#[test]
fn zero_transfer_time_warning() {
    let offset = lat_offset(100.0);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        transfers: vec![make_transfer(
            Some("A"),
            Some("B"),
            TransferType::MinimumTime,
            Some(0),
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "zero_transfer_time")
        .collect();
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].severity, Severity::Warning);
}

#[test]
fn self_transfer_and_zero_time_both_emitted() {
    let feed = GtfsFeed {
        stops: vec![make_stop("S01", BASE_LAT, BASE_LON)],
        transfers: vec![make_transfer(
            Some("S01"),
            Some("S01"),
            TransferType::MinimumTime,
            Some(0),
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let self_t: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "self_transfer")
        .collect();
    let zero_t: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "zero_transfer_time")
        .collect();
    assert_eq!(self_t.len(), 1);
    assert_eq!(zero_t.len(), 1);
}

#[test]
fn type2_nonzero_time_ok() {
    let offset = lat_offset(100.0);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        transfers: vec![make_transfer(
            Some("A"),
            Some("B"),
            TransferType::MinimumTime,
            Some(120),
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "zero_transfer_time")
        .collect();
    assert!(matched.is_empty());
}

#[test]
fn missing_coords_skipped() {
    let feed = GtfsFeed {
        stops: vec![make_stop_no_coords("A"), make_stop_no_coords("B")],
        transfers: vec![make_transfer(
            Some("A"),
            Some("B"),
            TransferType::Recommended,
            None,
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let distance_errors: Vec<_> = errors
        .iter()
        .filter(|e| {
            e.rule_id == "transfer_distance_too_large"
                || e.rule_id == "transfer_distance_suspicious"
        })
        .collect();
    assert!(distance_errors.is_empty());
}

#[test]
fn missing_stop_ids_skipped() {
    let feed = GtfsFeed {
        stops: vec![make_stop("A", BASE_LAT, BASE_LON)],
        transfers: vec![make_transfer(
            None,
            Some("A"),
            TransferType::Recommended,
            None,
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let distance_errors: Vec<_> = errors
        .iter()
        .filter(|e| {
            e.rule_id == "transfer_distance_too_large"
                || e.rule_id == "transfer_distance_suspicious"
        })
        .collect();
    assert!(distance_errors.is_empty());
}

#[test]
fn error_context_complete() {
    let offset = lat_offset(15_000.0);
    let feed = GtfsFeed {
        stops: vec![
            make_stop("A", BASE_LAT, BASE_LON),
            make_stop("B", BASE_LAT + offset, BASE_LON),
        ],
        transfers: vec![make_transfer(
            Some("A"),
            Some("B"),
            TransferType::Recommended,
            None,
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let err = errors
        .iter()
        .find(|e| e.rule_id == "transfer_distance_too_large")
        .expect("expected error");
    assert_eq!(err.section, "7");
    assert_eq!(err.severity, Severity::Error);
    assert_eq!(err.file_name.as_deref(), Some("transfers.txt"));
    assert_eq!(err.line_number, Some(2));
    assert_eq!(err.field_name.as_deref(), Some("from_stop_id"));
    assert_eq!(err.value.as_deref(), Some("A"));
    assert!(!err.message.is_empty());
}
