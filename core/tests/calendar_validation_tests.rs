//! Tests for section 7 calendar validation rules (7.5 and 7.6).

use std::sync::Arc;

use chrono::NaiveDate;

use gapline_core::models::*;
use gapline_core::validation::ValidationRule;
use gapline_core::validation::schedule_time_validation::calendar_dates_coherence::CalendarDatesCoherenceRule;
use gapline_core::validation::schedule_time_validation::calendar_ranges::CalendarRangesRule;
use gapline_core::validation::schedule_time_validation::feed_coverage::FeedCoverageRule;
use gapline_core::validation::schedule_time_validation::service_dates::ServiceDateCache;
use gapline_core::validation::schedule_time_validation::trip_activity::TripActivityRule;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn d(y: i32, m: u32, day: u32) -> GtfsDate {
    GtfsDate(NaiveDate::from_ymd_opt(y, m, day).expect("valid date"))
}

fn make_calendar(
    service_id: &str,
    start: GtfsDate,
    end: GtfsDate,
    days: [bool; 7], // mon..sun
) -> Calendar {
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

fn make_calendar_date(service_id: &str, date: GtfsDate, ex: ExceptionType) -> CalendarDate {
    CalendarDate {
        service_id: ServiceId::from(service_id),
        date,
        exception_type: ex,
    }
}

fn make_trip(trip_id: &str, service_id: &str) -> Trip {
    Trip {
        route_id: RouteId::from("R1"),
        service_id: ServiceId::from(service_id),
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

fn make_feed_info(end_date: Option<GtfsDate>) -> FeedInfo {
    FeedInfo {
        feed_publisher_name: "Test".to_string(),
        feed_publisher_url: Url::from("https://example.com"),
        feed_lang: LanguageCode::from("en"),
        default_lang: None,
        feed_start_date: None,
        feed_end_date: end_date,
        feed_version: None,
        feed_contact_email: None,
        feed_contact_url: None,
    }
}

fn all_days(on: bool) -> [bool; 7] {
    [on; 7]
}

fn monday_only() -> [bool; 7] {
    [true, false, false, false, false, false, false]
}

// ---------------------------------------------------------------------------
// Test 1: valid calendar — no errors
// ---------------------------------------------------------------------------

#[test]
fn test_1_valid_calendar() {
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 1),
            d(2026, 6, 30),
            monday_only(),
        )],
        ..Default::default()
    };
    assert!(CalendarRangesRule.validate(&feed).is_empty());
}

// ---------------------------------------------------------------------------
// Test 2: inverted date range — CA1
// ---------------------------------------------------------------------------

#[test]
fn test_2_inverted_date_range() {
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 6, 30),
            d(2026, 1, 1),
            monday_only(),
        )],
        ..Default::default()
    };
    let errors = CalendarRangesRule.validate(&feed);
    let inverted: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "inverted_date_range")
        .collect();
    assert_eq!(inverted.len(), 1);
    assert_eq!(inverted[0].section, "7");
    assert_eq!(inverted[0].file_name.as_deref(), Some("calendar.txt"));
}

// ---------------------------------------------------------------------------
// Test 3: start_date == end_date is valid (one-day service) — CA1 edge
// ---------------------------------------------------------------------------

#[test]
fn test_3_single_day_service() {
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 3, 1),
            d(2026, 3, 1),
            monday_only(),
        )],
        ..Default::default()
    };
    let errors = CalendarRangesRule.validate(&feed);
    assert!(
        errors.iter().all(|e| e.rule_id != "inverted_date_range"),
        "single-day service should not trigger inverted_date_range"
    );
}

// ---------------------------------------------------------------------------
// Test 4: inactive service (all days 0, no exceptions) — CA2
// ---------------------------------------------------------------------------

#[test]
fn test_4_inactive_service() {
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 1),
            d(2026, 6, 30),
            all_days(false),
        )],
        ..Default::default()
    };
    let errors = CalendarRangesRule.validate(&feed);
    let inactive: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "inactive_service")
        .collect();
    assert_eq!(inactive.len(), 1);
}

// ---------------------------------------------------------------------------
// Test 5: all days 0 but exceptions add active days — CA2 (no warning)
// ---------------------------------------------------------------------------

#[test]
fn test_5_inactive_with_additions() {
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 1),
            d(2026, 6, 30),
            all_days(false),
        )],
        calendar_dates: vec![make_calendar_date(
            "S1",
            d(2026, 3, 15),
            ExceptionType::Added,
        )],
        ..Default::default()
    };
    let errors = CalendarRangesRule.validate(&feed);
    assert!(errors.iter().all(|e| e.rule_id != "inactive_service"));
}

// ---------------------------------------------------------------------------
// Test 6: expired feed — CA4
// ---------------------------------------------------------------------------

#[test]
fn test_6_expired_feed() {
    let feed = GtfsFeed {
        feed_info: Some(make_feed_info(Some(d(2025, 1, 1)))),
        ..Default::default()
    };
    let rule = FeedCoverageRule::new(30, 7, Some(d(2026, 4, 5)));
    let errors = rule.validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "expired_feed")
            .count(),
        1
    );
}

// ---------------------------------------------------------------------------
// Test 7: feed expires in 3 days (< 7 default) — CA5
// ---------------------------------------------------------------------------

#[test]
fn test_7_feed_expiring_soon() {
    let today = d(2026, 4, 5);
    let feed = GtfsFeed {
        feed_info: Some(make_feed_info(Some(d(2026, 4, 8)))),
        ..Default::default()
    };
    let rule = FeedCoverageRule::new(30, 7, Some(today));
    let errors = rule.validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "feed_expiring_soon")
            .count(),
        1
    );
    assert!(errors.iter().all(|e| e.rule_id != "expired_feed"));
}

// ---------------------------------------------------------------------------
// Test 8: feed expires in 30 days (> 7 default) — no warning
// ---------------------------------------------------------------------------

#[test]
fn test_8_feed_far_expiration() {
    let today = d(2026, 4, 5);
    let feed = GtfsFeed {
        feed_info: Some(make_feed_info(Some(d(2026, 5, 5)))),
        ..Default::default()
    };
    let rule = FeedCoverageRule::new(30, 7, Some(today));
    let errors = rule.validate(&feed);
    assert!(
        errors
            .iter()
            .all(|e| e.rule_id != "feed_expiring_soon" && e.rule_id != "expired_feed")
    );
}

// ---------------------------------------------------------------------------
// Test 9: no feed_info — no expiration checks
// ---------------------------------------------------------------------------

#[test]
fn test_9_no_feed_info() {
    let feed = GtfsFeed::default();
    let rule = FeedCoverageRule::new(30, 7, Some(d(2026, 4, 5)));
    let errors = rule.validate(&feed);
    assert!(
        errors
            .iter()
            .all(|e| e.rule_id != "expired_feed" && e.rule_id != "feed_expiring_soon")
    );
}

// ---------------------------------------------------------------------------
// Test 10: short feed coverage — CA3
// ---------------------------------------------------------------------------

#[test]
fn test_10_short_feed_coverage() {
    // 15-day range → < 30-day threshold → WARNING.
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 1),
            d(2026, 1, 15),
            monday_only(),
        )],
        ..Default::default()
    };
    let rule = FeedCoverageRule::new(30, 7, Some(d(2026, 4, 5)));
    let errors = rule.validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "short_feed_coverage")
            .count(),
        1
    );
}

// ---------------------------------------------------------------------------
// Test 11: service only in calendar_dates with only exception_type=2 — CA6
// ---------------------------------------------------------------------------

#[test]
fn test_11_service_never_active() {
    let feed = GtfsFeed {
        calendar_dates: vec![
            make_calendar_date("SVC1", d(2026, 3, 10), ExceptionType::Removed),
            make_calendar_date("SVC1", d(2026, 3, 11), ExceptionType::Removed),
        ],
        ..Default::default()
    };
    let errors = CalendarDatesCoherenceRule.validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "service_never_active")
            .count(),
        1
    );
}

// ---------------------------------------------------------------------------
// Test 12: exception date out of range — CA8
// ---------------------------------------------------------------------------

#[test]
fn test_12_exception_date_out_of_range() {
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 1),
            d(2026, 6, 30),
            monday_only(),
        )],
        calendar_dates: vec![make_calendar_date(
            "S1",
            d(2026, 12, 1),
            ExceptionType::Added,
        )],
        ..Default::default()
    };
    let errors = CalendarDatesCoherenceRule.validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "exception_date_out_of_range")
            .count(),
        1
    );
}

// ---------------------------------------------------------------------------
// Test 13: exception date inside calendar range — CA8 no warning
// ---------------------------------------------------------------------------

#[test]
fn test_13_exception_date_in_range() {
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 1),
            d(2026, 6, 30),
            monday_only(),
        )],
        calendar_dates: vec![make_calendar_date(
            "S1",
            d(2026, 3, 15),
            ExceptionType::Added,
        )],
        ..Default::default()
    };
    let errors = CalendarDatesCoherenceRule.validate(&feed);
    assert!(
        errors
            .iter()
            .all(|e| e.rule_id != "exception_date_out_of_range")
    );
}

// ---------------------------------------------------------------------------
// Test 14: trip with low activity (only 3 days) — CA7
// ---------------------------------------------------------------------------

#[test]
fn test_14_low_trip_activity() {
    // Service is Monday-only over a 3-week window → 3 active Mondays.
    // 2026-01-05 is a Monday.
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 5),
            d(2026, 1, 19),
            monday_only(),
        )],
        trips: vec![make_trip("T1", "S1")],
        ..Default::default()
    };
    let rule = TripActivityRule::new(7, Arc::new(ServiceDateCache::new()));
    let errors = rule.validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "low_trip_activity")
            .count(),
        1
    );
}

// ---------------------------------------------------------------------------
// Edge cases beyond the 14 scenarios in the ticket
// ---------------------------------------------------------------------------

#[test]
fn coverage_exactly_at_threshold_passes() {
    // 30-day range: 2026-01-01 → 2026-01-30 inclusive = 30 days.
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 1),
            d(2026, 1, 30),
            monday_only(),
        )],
        ..Default::default()
    };
    let rule = FeedCoverageRule::new(30, 7, Some(d(2026, 4, 5)));
    let errors = rule.validate(&feed);
    assert!(errors.iter().all(|e| e.rule_id != "short_feed_coverage"));
}

#[test]
fn feed_end_date_equals_today_is_not_expired() {
    let today = d(2026, 4, 5);
    let feed = GtfsFeed {
        feed_info: Some(make_feed_info(Some(today))),
        ..Default::default()
    };
    let rule = FeedCoverageRule::new(30, 7, Some(today));
    let errors = rule.validate(&feed);
    assert!(errors.iter().all(|e| e.rule_id != "expired_feed"));
    // 0 days until expiry < 7 → should trigger feed_expiring_soon.
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "feed_expiring_soon")
            .count(),
        1
    );
}

#[test]
fn exception_date_equals_start_or_end_is_in_range() {
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 1),
            d(2026, 6, 30),
            monday_only(),
        )],
        calendar_dates: vec![
            make_calendar_date("S1", d(2026, 1, 1), ExceptionType::Added),
            make_calendar_date("S1", d(2026, 6, 30), ExceptionType::Removed),
        ],
        ..Default::default()
    };
    let errors = CalendarDatesCoherenceRule.validate(&feed);
    assert!(
        errors
            .iter()
            .all(|e| e.rule_id != "exception_date_out_of_range")
    );
}

#[test]
fn trip_activity_includes_exception_additions() {
    // Service with no weekdays active, but 10 explicit additions → 10 days.
    let additions: Vec<CalendarDate> = (1..=10)
        .map(|day| make_calendar_date("S1", d(2026, 1, day), ExceptionType::Added))
        .collect();
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 1),
            d(2026, 6, 30),
            all_days(false),
        )],
        calendar_dates: additions,
        trips: vec![make_trip("T1", "S1")],
        ..Default::default()
    };
    let rule = TripActivityRule::new(7, Arc::new(ServiceDateCache::new()));
    let errors = rule.validate(&feed);
    assert!(errors.iter().all(|e| e.rule_id != "low_trip_activity"));
}

#[test]
fn trip_activity_respects_exception_removals() {
    // Monday-only service over 3 Mondays, but 1 Monday removed → 2 days < 7.
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 5),
            d(2026, 1, 19),
            monday_only(),
        )],
        calendar_dates: vec![make_calendar_date(
            "S1",
            d(2026, 1, 12),
            ExceptionType::Removed,
        )],
        trips: vec![make_trip("T1", "S1")],
        ..Default::default()
    };
    let rule = TripActivityRule::new(7, Arc::new(ServiceDateCache::new()));
    let errors = rule.validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "low_trip_activity")
            .count(),
        1
    );
}

#[test]
fn feed_coverage_union_includes_calendar_dates() {
    // calendar says Jan 1-10 (10 days) but calendar_dates extends to Feb 28.
    // Union = Jan 1 → Feb 28 = 59 days → above 30-day threshold.
    let feed = GtfsFeed {
        calendars: vec![make_calendar(
            "S1",
            d(2026, 1, 1),
            d(2026, 1, 10),
            monday_only(),
        )],
        calendar_dates: vec![make_calendar_date(
            "S1",
            d(2026, 2, 28),
            ExceptionType::Added,
        )],
        ..Default::default()
    };
    let rule = FeedCoverageRule::new(30, 7, Some(d(2026, 4, 5)));
    let errors = rule.validate(&feed);
    assert!(errors.iter().all(|e| e.rule_id != "short_feed_coverage"));
}
