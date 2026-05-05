//! Tests for section 7 - Schedule Time Validation.

use gapline_core::models::*;
use gapline_core::validation::ValidationRule;
use gapline_core::validation::schedule_time_validation::frequencies::FrequenciesCoherenceRule;
use gapline_core::validation::schedule_time_validation::stop_times::StopTimesTimeSequenceRule;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_stop_time(
    trip_id: &str,
    seq: u32,
    arr: Option<GtfsTime>,
    dep: Option<GtfsTime>,
) -> StopTime {
    StopTime {
        trip_id: TripId::from(trip_id),
        arrival_time: arr,
        departure_time: dep,
        stop_id: StopId::from("S1"),
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

fn make_stop_time_with_dist(
    trip_id: &str,
    seq: u32,
    arr: Option<GtfsTime>,
    dep: Option<GtfsTime>,
    dist: Option<f64>,
) -> StopTime {
    StopTime {
        shape_dist_traveled: dist,
        ..make_stop_time(trip_id, seq, arr, dep)
    }
}

fn make_frequency(trip_id: &str, start: GtfsTime, end: GtfsTime, headway: u32) -> Frequency {
    Frequency {
        trip_id: TripId::from(trip_id),
        start_time: start,
        end_time: end,
        headway_secs: headway,
        exact_times: None,
    }
}

fn feed_with_stop_times(stop_times: Vec<StopTime>) -> GtfsFeed {
    GtfsFeed {
        stop_times,
        ..Default::default()
    }
}

fn feed_with_frequencies(frequencies: Vec<Frequency>) -> GtfsFeed {
    GtfsFeed {
        frequencies,
        ..Default::default()
    }
}

fn t(h: u32, m: u32, s: u32) -> GtfsTime {
    GtfsTime::from_hms(h, m, s)
}

// ---------------------------------------------------------------------------
// stop_times rules
// ---------------------------------------------------------------------------

#[test]
fn trip_with_increasing_times() {
    let feed = feed_with_stop_times(vec![
        make_stop_time("T1", 1, Some(t(8, 0, 0)), Some(t(8, 0, 0))),
        make_stop_time("T1", 2, Some(t(8, 5, 0)), Some(t(8, 5, 0))),
        make_stop_time("T1", 3, Some(t(8, 10, 0)), Some(t(8, 10, 0))),
    ]);
    let rule = StopTimesTimeSequenceRule::new(Some(24));
    assert!(rule.validate(&feed).is_empty());
}

#[test]
fn decreasing_arrival_time() {
    let feed = feed_with_stop_times(vec![
        make_stop_time("T1", 1, Some(t(8, 10, 0)), Some(t(8, 10, 0))),
        make_stop_time("T1", 2, Some(t(8, 5, 0)), Some(t(8, 5, 0))),
    ]);
    let errors = StopTimesTimeSequenceRule::new(Some(24)).validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "decreasing_time")
            .count(),
        1
    );
}

#[test]
fn departure_before_arrival() {
    let feed = feed_with_stop_times(vec![
        make_stop_time("T1", 1, Some(t(8, 0, 0)), Some(t(8, 0, 0))),
        make_stop_time("T1", 2, Some(t(8, 10, 0)), Some(t(8, 5, 0))),
    ]);
    let errors = StopTimesTimeSequenceRule::new(Some(24)).validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "departure_before_arrival")
            .count(),
        1
    );
}

#[test]
fn non_increasing_stop_sequence() {
    // Duplicate stop_sequence values within the same trip.
    let feed = feed_with_stop_times(vec![
        make_stop_time("T1", 1, Some(t(8, 0, 0)), Some(t(8, 0, 0))),
        make_stop_time("T1", 3, Some(t(8, 5, 0)), Some(t(8, 5, 0))),
        make_stop_time("T1", 3, Some(t(8, 10, 0)), Some(t(8, 10, 0))),
    ]);
    let errors = StopTimesTimeSequenceRule::new(Some(24)).validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "non_increasing_stop_sequence")
            .count(),
        1
    );
}

#[test]
fn decreasing_shape_dist() {
    let feed = feed_with_stop_times(vec![
        make_stop_time_with_dist("T1", 1, Some(t(8, 0, 0)), Some(t(8, 0, 0)), Some(100.0)),
        make_stop_time_with_dist("T1", 2, Some(t(8, 5, 0)), Some(t(8, 5, 0)), Some(50.0)),
    ]);
    let errors = StopTimesTimeSequenceRule::new(Some(24)).validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "decreasing_shape_dist")
            .count(),
        1
    );
}

#[test]
fn trip_too_long() {
    let feed = feed_with_stop_times(vec![
        make_stop_time("T1", 1, Some(t(6, 0, 0)), Some(t(6, 0, 0))),
        make_stop_time("T1", 2, Some(t(31, 0, 0)), Some(t(31, 0, 0))),
    ]);
    let errors = StopTimesTimeSequenceRule::new(Some(24)).validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "trip_too_long")
            .count(),
        1
    );
}

#[test]
fn trip_exactly_24h() {
    let feed = feed_with_stop_times(vec![
        make_stop_time("T1", 1, Some(t(6, 0, 0)), Some(t(6, 0, 0))),
        make_stop_time("T1", 2, Some(t(30, 0, 0)), Some(t(30, 0, 0))),
    ]);
    let errors = StopTimesTimeSequenceRule::new(Some(24)).validate(&feed);
    assert!(
        errors
            .iter()
            .filter(|e| e.rule_id == "trip_too_long")
            .count()
            == 0
    );
}

#[test]
fn first_stop_times_differ() {
    let feed = feed_with_stop_times(vec![
        make_stop_time("T1", 1, Some(t(8, 0, 0)), Some(t(8, 2, 0))),
        make_stop_time("T1", 2, Some(t(8, 10, 0)), Some(t(8, 10, 0))),
    ]);
    let errors = StopTimesTimeSequenceRule::new(Some(24)).validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "first_stop_times_differ")
            .count(),
        1
    );
}

// ---------------------------------------------------------------------------
// frequencies rules
// ---------------------------------------------------------------------------

#[test]
fn frequency_start_ge_end() {
    let feed = feed_with_frequencies(vec![make_frequency(
        "T1",
        GtfsTime::from_hms(10, 0, 0),
        GtfsTime::from_hms(9, 0, 0),
        600,
    )]);
    let errors = FrequenciesCoherenceRule.validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "invalid_time_range")
            .count(),
        1
    );
}

#[test]
fn overlapping_frequencies() {
    let feed = feed_with_frequencies(vec![
        make_frequency(
            "T1",
            GtfsTime::from_hms(6, 0, 0),
            GtfsTime::from_hms(9, 0, 0),
            600,
        ),
        make_frequency(
            "T1",
            GtfsTime::from_hms(8, 0, 0),
            GtfsTime::from_hms(12, 0, 0),
            600,
        ),
    ]);
    let errors = FrequenciesCoherenceRule.validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "overlapping_frequencies")
            .count(),
        1
    );
}

#[test]
fn non_overlapping_frequencies() {
    let feed = feed_with_frequencies(vec![
        make_frequency(
            "T1",
            GtfsTime::from_hms(6, 0, 0),
            GtfsTime::from_hms(9, 0, 0),
            600,
        ),
        make_frequency(
            "T1",
            GtfsTime::from_hms(9, 0, 0),
            GtfsTime::from_hms(12, 0, 0),
            600,
        ),
    ]);
    let errors = FrequenciesCoherenceRule.validate(&feed);
    assert!(
        errors
            .iter()
            .filter(|e| e.rule_id == "overlapping_frequencies")
            .count()
            == 0
    );
}

#[test]
fn headway_secs_zero() {
    let feed = feed_with_frequencies(vec![make_frequency(
        "T1",
        GtfsTime::from_hms(6, 0, 0),
        GtfsTime::from_hms(9, 0, 0),
        0,
    )]);
    let errors = FrequenciesCoherenceRule.validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "invalid_headway")
            .count(),
        1
    );
}

#[test]
fn complex_trip_multiple_errors() {
    let mut stop_times: Vec<StopTime> = (1..=50)
        .map(|seq| {
            let minutes = seq * 2;
            make_stop_time("T1", seq, Some(t(8, minutes, 0)), Some(t(8, minutes, 0)))
        })
        .collect();

    stop_times[19] = make_stop_time("T1", 20, Some(t(8, 0, 0)), Some(t(8, 0, 0)));
    stop_times[39] = make_stop_time("T1", 40, Some(t(8, 0, 0)), Some(t(8, 0, 0)));

    let errors = StopTimesTimeSequenceRule::new(None).validate(&feed_with_stop_times(stop_times));
    let matching: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "decreasing_time")
        .collect();
    assert_eq!(matching.len(), 2);
    for err in &matching {
        assert!(err.line_number.is_some());
    }
}

#[test]
fn valid_times_over_24h() {
    let feed = feed_with_stop_times(vec![
        make_stop_time("T1", 1, Some(t(25, 0, 0)), Some(t(25, 0, 0))),
        make_stop_time("T1", 2, Some(t(25, 30, 0)), Some(t(25, 30, 0))),
        make_stop_time("T1", 3, Some(t(26, 0, 0)), Some(t(26, 0, 0))),
    ]);
    assert!(
        StopTimesTimeSequenceRule::new(Some(24))
            .validate(&feed)
            .is_empty()
    );
}
