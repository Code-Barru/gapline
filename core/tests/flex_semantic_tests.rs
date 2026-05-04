//! Tests for section 9 — Flex semantic validation rules.

use std::sync::Arc;

use chrono::NaiveDate;
use gapline_core::models::*;
use gapline_core::validation::ValidationRule;
use gapline_core::validation::flex_semantic::prior_days_coverage::PriorDaysServiceCoverageRule;
use gapline_core::validation::flex_semantic::rules::{
    EmptyLocationGroupRule, MeanDurationFactorPositiveRule, PriorNoticeLastDayTimeRule,
    PriorNoticeMinMaxRule, PriorNoticeMinPositiveRule, SafeDurationFactorRule,
    ScheduledWithBookingRuleRule, WindowOrderRule,
};
use gapline_core::validation::schedule_time_validation::service_dates::ServiceDateCache;

fn t(h: u32, m: u32, s: u32) -> GtfsTime {
    GtfsTime::from_hms(h, m, s)
}

fn date(y: i32, m: u32, d: u32) -> GtfsDate {
    GtfsDate(NaiveDate::from_ymd_opt(y, m, d).unwrap())
}

fn stop_time(trip: &str, seq: u32) -> StopTime {
    StopTime {
        trip_id: TripId::from(trip),
        arrival_time: None,
        departure_time: None,
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

fn booking_rule(id: &str, bt: Option<BookingType>) -> BookingRule {
    BookingRule {
        booking_rule_id: BookingRuleId::from(id),
        booking_type: bt,
        prior_notice_duration_min: None,
        prior_notice_duration_max: None,
        prior_notice_last_day: None,
        prior_notice_last_time: None,
        prior_notice_start_day: None,
        prior_notice_start_time: None,
        prior_notice_service_id: None,
        message: None,
        pickup_message: None,
        drop_off_message: None,
        phone_number: None,
        info_url: None,
        booking_url: None,
    }
}

fn trip(id: &str, service: &str) -> Trip {
    Trip {
        route_id: RouteId::from("R1"),
        service_id: ServiceId::from(service),
        trip_id: TripId::from(id),
        trip_headsign: None,
        trip_short_name: None,
        direction_id: None,
        block_id: None,
        shape_id: None,
        wheelchair_accessible: None,
        bikes_allowed: None,
    }
}

fn calendar(service: &str, start: GtfsDate, end: GtfsDate, every_day: bool) -> Calendar {
    Calendar {
        service_id: ServiceId::from(service),
        monday: every_day,
        tuesday: every_day,
        wednesday: every_day,
        thursday: every_day,
        friday: every_day,
        saturday: every_day,
        sunday: every_day,
        start_date: start,
        end_date: end,
    }
}

/// Returns a feed pre-marked as Flex so rule guards (`has_flex()`) don't
/// short-circuit. Tests then push only the records they care about.
fn flex_feed() -> GtfsFeed {
    let mut feed = GtfsFeed::default();
    feed.loaded_files.insert("booking_rules.txt".to_string());
    feed
}

#[test]
fn valid_flex_feed_no_errors() {
    let mut feed = flex_feed();
    feed.loaded_files.insert("location_groups.txt".to_string());
    feed.loaded_files
        .insert("location_group_stops.txt".to_string());

    let mut br = booking_rule("BR1", Some(BookingType::SameDay));
    br.prior_notice_duration_min = Some(30);
    br.prior_notice_duration_max = Some(120);
    feed.booking_rules.push(br);

    let mut st = stop_time("T1", 1);
    st.start_pickup_drop_off_window = Some(t(8, 0, 0));
    st.end_pickup_drop_off_window = Some(t(9, 0, 0));
    st.pickup_type = Some(PickupType::PhoneAgency);
    st.pickup_booking_rule_id = Some(BookingRuleId::from("BR1"));
    st.mean_duration_factor = Some(1.0);
    st.safe_duration_factor = Some(1.5);
    feed.stop_times.push(st);

    feed.location_groups.push(LocationGroup {
        location_group_id: LocationGroupId::from("LG1"),
        location_group_name: Some("Zone".into()),
    });
    feed.location_group_stops.push(LocationGroupStop {
        location_group_id: LocationGroupId::from("LG1"),
        stop_id: StopId::from("S1"),
    });

    assert!(WindowOrderRule.validate(&feed).is_empty());
    assert!(PriorNoticeMinPositiveRule.validate(&feed).is_empty());
    assert!(PriorNoticeMinMaxRule.validate(&feed).is_empty());
    assert!(PriorNoticeLastDayTimeRule.validate(&feed).is_empty());
    assert!(EmptyLocationGroupRule.validate(&feed).is_empty());
    assert!(MeanDurationFactorPositiveRule.validate(&feed).is_empty());
    assert!(SafeDurationFactorRule.validate(&feed).is_empty());
    assert!(ScheduledWithBookingRuleRule.validate(&feed).is_empty());
}

#[test]
fn inverted_window_emits_error() {
    let mut feed = flex_feed();
    let mut st = stop_time("T1", 1);
    st.start_pickup_drop_off_window = Some(t(10, 0, 0));
    st.end_pickup_drop_off_window = Some(t(9, 0, 0));
    feed.stop_times.push(st);

    let errors = WindowOrderRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "flex_invalid_window");
    assert_eq!(errors[0].section, "9");
}

#[test]
fn zero_length_window_emits_error() {
    let mut feed = flex_feed();
    let mut st = stop_time("T1", 1);
    st.start_pickup_drop_off_window = Some(t(10, 0, 0));
    st.end_pickup_drop_off_window = Some(t(10, 0, 0));
    feed.stop_times.push(st);

    let errors = WindowOrderRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

// `prior_notice_duration_min` is `Option<u32>` — negative values are rejected
// by the parser, so 0 is the only invalid value reachable at this layer.
#[test]
fn prior_notice_min_zero_emits_error() {
    let mut feed = flex_feed();
    let mut br = booking_rule("BR1", None);
    br.prior_notice_duration_min = Some(0);
    feed.booking_rules.push(br);

    let errors = PriorNoticeMinPositiveRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "flex_invalid_prior_notice");
}

#[test]
fn prior_notice_max_below_min_emits_error() {
    let mut feed = flex_feed();
    let mut br = booking_rule("BR1", None);
    br.prior_notice_duration_min = Some(60);
    br.prior_notice_duration_max = Some(30);
    feed.booking_rules.push(br);

    let errors = PriorNoticeMinMaxRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

#[test]
fn last_day_without_last_time_emits_warning() {
    let mut feed = flex_feed();
    let mut br = booking_rule("BR1", None);
    br.prior_notice_last_day = Some(1);
    feed.booking_rules.push(br);

    let errors = PriorNoticeLastDayTimeRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

#[test]
fn empty_location_group_emits_warning() {
    let mut feed = flex_feed();
    feed.loaded_files.insert("location_groups.txt".to_string());
    feed.location_groups.push(LocationGroup {
        location_group_id: LocationGroupId::from("LG1"),
        location_group_name: None,
    });

    let errors = EmptyLocationGroupRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

#[test]
fn negative_mean_duration_factor_emits_error() {
    let mut feed = flex_feed();
    let mut st = stop_time("T1", 1);
    st.mean_duration_factor = Some(-0.5);
    feed.stop_times.push(st);

    let errors = MeanDurationFactorPositiveRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

#[test]
fn safe_below_mean_emits_warning() {
    let mut feed = flex_feed();
    let mut st = stop_time("T1", 1);
    st.mean_duration_factor = Some(1.5);
    st.safe_duration_factor = Some(1.0);
    feed.stop_times.push(st);

    let errors = SafeDurationFactorRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

#[test]
fn scheduled_pickup_with_booking_rule_emits_warning() {
    let mut feed = flex_feed();
    let mut st = stop_time("T1", 1);
    st.pickup_type = Some(PickupType::Regular);
    st.pickup_booking_rule_id = Some(BookingRuleId::from("BR1"));
    feed.stop_times.push(st);

    let errors = ScheduledWithBookingRuleRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].field_name.as_deref(),
        Some("pickup_booking_rule_id")
    );
}

#[test]
fn scheduled_drop_off_with_booking_rule_emits_warning() {
    let mut feed = flex_feed();
    let mut st = stop_time("T1", 1);
    st.drop_off_type = Some(DropOffType::Regular);
    st.drop_off_booking_rule_id = Some(BookingRuleId::from("BR1"));
    feed.stop_times.push(st);

    let errors = ScheduledWithBookingRuleRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].field_name.as_deref(),
        Some("drop_off_booking_rule_id")
    );
}

#[test]
fn feed_without_flex_no_errors() {
    let feed = GtfsFeed::default();
    assert!(!feed.has_flex());
    assert!(WindowOrderRule.validate(&feed).is_empty());
    assert!(PriorNoticeMinPositiveRule.validate(&feed).is_empty());
    assert!(PriorNoticeMinMaxRule.validate(&feed).is_empty());
    assert!(PriorNoticeLastDayTimeRule.validate(&feed).is_empty());
    assert!(EmptyLocationGroupRule.validate(&feed).is_empty());
    assert!(MeanDurationFactorPositiveRule.validate(&feed).is_empty());
    assert!(SafeDurationFactorRule.validate(&feed).is_empty());
    assert!(ScheduledWithBookingRuleRule.validate(&feed).is_empty());
    let cache = Arc::new(ServiceDateCache::new());
    assert!(
        PriorDaysServiceCoverageRule::new(cache)
            .validate(&feed)
            .is_empty()
    );
}

#[test]
fn prior_days_with_single_day_service_emits_warning() {
    let mut feed = flex_feed();
    feed.booking_rules
        .push(booking_rule("BR1", Some(BookingType::PriorDays)));
    feed.trips.push(trip("T1", "SVC1"));
    feed.calendars
        .push(calendar("SVC1", date(2024, 1, 1), date(2024, 1, 1), true));
    let mut st = stop_time("T1", 1);
    st.pickup_booking_rule_id = Some(BookingRuleId::from("BR1"));
    feed.stop_times.push(st);

    let errors =
        PriorDaysServiceCoverageRule::new(Arc::new(ServiceDateCache::new())).validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "flex_insufficient_service_coverage");
}

#[test]
fn prior_days_with_multi_day_service_no_warning() {
    let mut feed = flex_feed();
    feed.booking_rules
        .push(booking_rule("BR1", Some(BookingType::PriorDays)));
    feed.trips.push(trip("T1", "SVC1"));
    feed.calendars
        .push(calendar("SVC1", date(2024, 1, 1), date(2024, 1, 31), true));
    let mut st = stop_time("T1", 1);
    st.pickup_booking_rule_id = Some(BookingRuleId::from("BR1"));
    feed.stop_times.push(st);

    let errors =
        PriorDaysServiceCoverageRule::new(Arc::new(ServiceDateCache::new())).validate(&feed);
    assert!(errors.is_empty());
}

#[test]
fn prior_days_dedups_per_trip_booking_rule_pair() {
    let mut feed = flex_feed();
    feed.booking_rules
        .push(booking_rule("BR1", Some(BookingType::PriorDays)));
    feed.trips.push(trip("T1", "SVC1"));
    feed.calendars
        .push(calendar("SVC1", date(2024, 1, 1), date(2024, 1, 1), true));
    for seq in 1..=3 {
        let mut st = stop_time("T1", seq);
        st.pickup_booking_rule_id = Some(BookingRuleId::from("BR1"));
        feed.stop_times.push(st);
    }

    let errors =
        PriorDaysServiceCoverageRule::new(Arc::new(ServiceDateCache::new())).validate(&feed);
    assert_eq!(errors.len(), 1);
}
