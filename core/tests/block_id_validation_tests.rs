//! Tests for section 7.12 — `block_id` overlap validation.

use std::sync::Arc;

use chrono::NaiveDate;

use headway_core::models::*;
use headway_core::validation::ValidationRule;
use headway_core::validation::schedule_time_validation::block_id::BlockIdTripOverlapRule;
use headway_core::validation::schedule_time_validation::service_dates::ServiceDateCache;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn d(y: i32, m: u32, day: u32) -> GtfsDate {
    GtfsDate(NaiveDate::from_ymd_opt(y, m, day).expect("valid date"))
}

fn time(h: u32, m: u32) -> GtfsTime {
    GtfsTime::from_hms(h, m, 0)
}

fn make_trip_with_block(trip_id: &str, service_id: &str, block_id: &str) -> Trip {
    Trip {
        route_id: RouteId::from("R1"),
        service_id: ServiceId::from(service_id),
        trip_id: TripId::from(trip_id),
        trip_headsign: None,
        trip_short_name: None,
        direction_id: None,
        block_id: Some(block_id.to_string()),
        shape_id: None,
        wheelchair_accessible: None,
        bikes_allowed: None,
    }
}

fn make_stop_time(trip_id: &str, seq: u32, arr: GtfsTime, dep: GtfsTime) -> StopTime {
    StopTime {
        trip_id: TripId::from(trip_id),
        arrival_time: Some(arr),
        departure_time: Some(dep),
        stop_id: StopId::from("S1"),
        stop_sequence: seq,
        stop_headsign: None,
        pickup_type: None,
        drop_off_type: None,
        continuous_pickup: None,
        continuous_drop_off: None,
        shape_dist_traveled: None,
        timepoint: None,
    }
}

fn make_calendar(service_id: &str, start: GtfsDate, end: GtfsDate, days: [bool; 7]) -> Calendar {
    Calendar {
        service_id: ServiceId::from(service_id),
        monday: days[0],
        tuesday: days[1],
        wednesday: days[2],
        thursday: days[3],
        friday: days[4],
        saturday: days[5],
        sunday: days[6],
        start_date: start,
        end_date: end,
    }
}

fn all_days() -> [bool; 7] {
    [true; 7]
}

fn rule() -> BlockIdTripOverlapRule {
    BlockIdTripOverlapRule::new(Arc::new(ServiceDateCache::new()))
}

// ---------------------------------------------------------------------------
// Test 1: no overlap, consecutive trips
// ---------------------------------------------------------------------------

#[test]
fn no_overlap_consecutive_trips() {
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 1),
            d(2026, 1, 31),
            all_days(),
        )],
        trips: vec![
            make_trip_with_block("T1", "S1", "B1"),
            make_trip_with_block("T2", "S1", "B1"),
        ],
        stop_times: vec![
            make_stop_time("T1", 1, time(6, 0), time(6, 0)),
            make_stop_time("T1", 2, time(7, 0), time(7, 0)),
            make_stop_time("T2", 1, time(7, 30), time(7, 30)),
            make_stop_time("T2", 2, time(8, 30), time(8, 30)),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    assert!(
        errors.iter().all(|e| e.rule_id != "block_id_trip_overlap"),
        "consecutive trips should not overlap"
    );
}

// ---------------------------------------------------------------------------
// Test 2: overlap same block, same service
// ---------------------------------------------------------------------------

#[test]
fn overlap_same_block_same_service() {
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 1),
            d(2026, 1, 31),
            all_days(),
        )],
        trips: vec![
            make_trip_with_block("T1", "S1", "B1"),
            make_trip_with_block("T2", "S1", "B1"),
        ],
        stop_times: vec![
            make_stop_time("T1", 1, time(6, 0), time(6, 0)),
            make_stop_time("T1", 2, time(7, 30), time(7, 30)),
            make_stop_time("T2", 1, time(7, 0), time(7, 0)),
            make_stop_time("T2", 2, time(8, 30), time(8, 30)),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let overlaps: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "block_id_trip_overlap")
        .collect();
    assert_eq!(overlaps.len(), 1);
}

// ---------------------------------------------------------------------------
// Test 3: same block, different services with no common days
// ---------------------------------------------------------------------------

#[test]
fn same_block_different_services_no_common_days() {
    let monday_only = [true, false, false, false, false, false, false];
    let tuesday_only = [false, true, false, false, false, false, false];
    let feed = GtfsFeed {
        calendars: vec![
            make_calendar("S_MON", d(2026, 1, 1), d(2026, 1, 31), monday_only),
            make_calendar("S_TUE", d(2026, 1, 1), d(2026, 1, 31), tuesday_only),
        ],
        trips: vec![
            make_trip_with_block("T1", "S_MON", "B1"),
            make_trip_with_block("T2", "S_TUE", "B1"),
        ],
        stop_times: vec![
            make_stop_time("T1", 1, time(6, 0), time(6, 0)),
            make_stop_time("T1", 2, time(7, 30), time(7, 30)),
            make_stop_time("T2", 1, time(6, 0), time(6, 0)),
            make_stop_time("T2", 2, time(7, 30), time(7, 30)),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    assert!(
        errors.iter().all(|e| e.rule_id != "block_id_trip_overlap"),
        "trips on disjoint service days should not conflict"
    );
}

// ---------------------------------------------------------------------------
// Test 4: adjacent boundaries are NOT overlap
// ---------------------------------------------------------------------------

#[test]
fn adjacent_boundaries_no_overlap() {
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 1),
            d(2026, 1, 31),
            all_days(),
        )],
        trips: vec![
            make_trip_with_block("T1", "S1", "B1"),
            make_trip_with_block("T2", "S1", "B1"),
        ],
        stop_times: vec![
            make_stop_time("T1", 1, time(6, 0), time(6, 0)),
            make_stop_time("T1", 2, time(7, 0), time(7, 0)),
            make_stop_time("T2", 1, time(7, 0), time(7, 0)),
            make_stop_time("T2", 2, time(8, 0), time(8, 0)),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    assert!(
        errors.iter().all(|e| e.rule_id != "block_id_trip_overlap"),
        "adjacent [06:00-07:00] + [07:00-08:00] should not overlap"
    );
}

// ---------------------------------------------------------------------------
// Test 5: trips without block_id are skipped
// ---------------------------------------------------------------------------

#[test]
fn trips_without_block_id_skipped() {
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 1),
            d(2026, 1, 31),
            all_days(),
        )],
        trips: vec![Trip {
            route_id: RouteId::from("R1"),
            service_id: ServiceId::from("S1"),
            trip_id: TripId::from("T1"),
            trip_headsign: None,
            trip_short_name: None,
            direction_id: None,
            block_id: None,
            shape_id: None,
            wheelchair_accessible: None,
            bikes_allowed: None,
        }],
        stop_times: vec![
            make_stop_time("T1", 1, time(6, 0), time(6, 0)),
            make_stop_time("T1", 2, time(7, 0), time(7, 0)),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// Test 6: trip with no stop_times — no crash
// ---------------------------------------------------------------------------

#[test]
fn trip_without_stop_times_no_crash() {
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 1),
            d(2026, 1, 31),
            all_days(),
        )],
        trips: vec![make_trip_with_block("T1", "S1", "B1")],
        stop_times: vec![],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// Test 7: error context completeness
// ---------------------------------------------------------------------------

#[test]
fn error_context_completeness() {
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 1),
            d(2026, 1, 31),
            all_days(),
        )],
        trips: vec![
            make_trip_with_block("T1", "S1", "B1"),
            make_trip_with_block("T2", "S1", "B1"),
        ],
        stop_times: vec![
            make_stop_time("T1", 1, time(6, 0), time(6, 0)),
            make_stop_time("T1", 2, time(7, 30), time(7, 30)),
            make_stop_time("T2", 1, time(7, 0), time(7, 0)),
            make_stop_time("T2", 2, time(8, 30), time(8, 30)),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let e = errors
        .iter()
        .find(|e| e.rule_id == "block_id_trip_overlap")
        .expect("should find overlap error");
    assert_eq!(e.section, "7");
    assert_eq!(e.file_name.as_deref(), Some("trips.txt"));
    assert!(e.line_number.is_some());
    assert_eq!(e.field_name.as_deref(), Some("block_id"));
    assert_eq!(e.value.as_deref(), Some("B1"));
}
