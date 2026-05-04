//! Tests for section 13 — third-party validator compatibility rules.

use gapline_core::models::*;
use gapline_core::validation::third_party::conveyal::ConveyalTripWithoutShapeRule;
use gapline_core::validation::third_party::etalab::EtalabMissingContactRule;
use gapline_core::validation::third_party::google::{
    GoogleCoordinatesInStopNameRule, GoogleIdenticalRouteColorsRule,
};
use gapline_core::validation::third_party::otp::{
    OtpMissingFeedVersionRule, OtpTripTooFewStopsRule,
};
use gapline_core::validation::{Severity, ValidationRule};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_stop(id: &str) -> Stop {
    Stop {
        stop_id: StopId::from(id),
        stop_code: None,
        stop_name: Some("Gare Centrale".into()),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(48.8566)),
        stop_lon: Some(Longitude(2.3522)),
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

fn make_route(id: &str) -> Route {
    Route {
        route_id: RouteId::from(id),
        agency_id: None,
        route_short_name: Some("A1".into()),
        route_long_name: Some("Line A1".into()),
        route_desc: None,
        route_type: RouteType::Bus,
        route_url: None,
        route_color: Some(Color("FF0000".into())),
        route_text_color: Some(Color("FFFFFF".into())),
        route_sort_order: None,
        continuous_pickup: None,
        continuous_drop_off: None,
        network_id: None,
    }
}

fn make_trip(id: &str) -> Trip {
    Trip {
        route_id: RouteId::from("R1"),
        service_id: ServiceId::from("S1"),
        trip_id: TripId::from(id),
        trip_headsign: None,
        trip_short_name: None,
        direction_id: None,
        block_id: None,
        shape_id: Some(ShapeId::from("SH1")),
        wheelchair_accessible: None,
        bikes_allowed: None,
    }
}

fn make_stop_time(trip_id: &str, seq: u32) -> StopTime {
    StopTime {
        trip_id: TripId::from(trip_id),
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

fn make_shape(id: &str, seq: u32) -> Shape {
    Shape {
        shape_id: ShapeId::from(id),
        shape_pt_lat: Latitude(48.8566),
        shape_pt_lon: Longitude(2.3522),
        shape_pt_sequence: seq,
        shape_dist_traveled: None,
    }
}

fn make_feed_info() -> FeedInfo {
    FeedInfo {
        feed_publisher_name: "Test Publisher".into(),
        feed_publisher_url: Url("https://example.com".into()),
        feed_lang: LanguageCode("fr".into()),
        default_lang: None,
        feed_start_date: None,
        feed_end_date: None,
        feed_version: Some("1.0".into()),
        feed_contact_email: Some(Email("contact@example.com".into())),
        feed_contact_url: None,
    }
}

fn exemplary_feed() -> GtfsFeed {
    GtfsFeed {
        stops: vec![make_stop("S1"), make_stop("S2")],
        routes: vec![make_route("R1")],
        trips: vec![make_trip("T1")],
        stop_times: vec![make_stop_time("T1", 1), make_stop_time("T1", 2)],
        shapes: vec![make_shape("SH1", 1), make_shape("SH1", 2)],
        feed_info: Some(make_feed_info()),
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 1 — Exemplary feed (0 section-13 issues)
// ---------------------------------------------------------------------------

#[test]
fn test_feed_compatible_all_validators() {
    let feed = exemplary_feed();
    let rules: Vec<Box<dyn ValidationRule>> = vec![
        Box::new(ConveyalTripWithoutShapeRule),
        Box::new(EtalabMissingContactRule),
        Box::new(OtpTripTooFewStopsRule),
        Box::new(OtpMissingFeedVersionRule),
        Box::new(GoogleCoordinatesInStopNameRule),
        Box::new(GoogleIdenticalRouteColorsRule),
    ];

    let errors: Vec<_> = rules.iter().flat_map(|r| r.validate(&feed)).collect();
    assert!(errors.is_empty(), "Expected 0 issues, got: {errors:?}");
}

// ---------------------------------------------------------------------------
// 2 — Trip with 1 stop_time
// ---------------------------------------------------------------------------

#[test]
fn test_trip_one_stop_time() {
    let feed = GtfsFeed {
        trips: vec![make_trip("T1")],
        stop_times: vec![make_stop_time("T1", 1)],
        ..Default::default()
    };
    let errors = OtpTripTooFewStopsRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "otp_trip_too_few_stops");
    assert_eq!(errors[0].severity, Severity::Error);
    assert_eq!(errors[0].section, "13");
    assert_eq!(errors[0].file_name.as_deref(), Some("trips.txt"));
    assert_eq!(errors[0].line_number, Some(2));
    assert_eq!(errors[0].value.as_deref(), Some("T1"));
}

// ---------------------------------------------------------------------------
// 3 — Trip with 2 stop_times
// ---------------------------------------------------------------------------

#[test]
fn test_trip_two_stop_times() {
    let feed = GtfsFeed {
        trips: vec![make_trip("T1")],
        stop_times: vec![make_stop_time("T1", 1), make_stop_time("T1", 2)],
        ..Default::default()
    };
    let errors = OtpTripTooFewStopsRule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// 4 — Missing feed_version
// ---------------------------------------------------------------------------

#[test]
fn test_missing_feed_version() {
    let mut info = make_feed_info();
    info.feed_version = None;
    let feed = GtfsFeed {
        feed_info: Some(info),
        ..Default::default()
    };
    let errors = OtpMissingFeedVersionRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "otp_missing_feed_version");
    assert_eq!(errors[0].severity, Severity::Warning);
    assert_eq!(errors[0].field_name.as_deref(), Some("feed_version"));
}

// ---------------------------------------------------------------------------
// 5 — Missing feed_contact_email
// ---------------------------------------------------------------------------

#[test]
fn test_missing_contact_email() {
    let mut info = make_feed_info();
    info.feed_contact_email = None;
    let feed = GtfsFeed {
        feed_info: Some(info),
        ..Default::default()
    };
    let errors = EtalabMissingContactRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "etalab_missing_contact");
    assert_eq!(errors[0].severity, Severity::Warning);
    assert_eq!(errors[0].field_name.as_deref(), Some("feed_contact_email"));
}

// ---------------------------------------------------------------------------
// 6 — Coordinates in stop_name
// ---------------------------------------------------------------------------

#[test]
fn test_coordinates_in_stop_name() {
    let mut stop = make_stop("S1");
    stop.stop_name = Some("45.5017, -73.5673".into());
    let feed = GtfsFeed {
        stops: vec![stop],
        ..Default::default()
    };
    let errors = GoogleCoordinatesInStopNameRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "google_coordinates_in_stop_name");
    assert_eq!(errors[0].severity, Severity::Warning);
    assert_eq!(errors[0].value.as_deref(), Some("45.5017, -73.5673"));
}

// ---------------------------------------------------------------------------
// 7 — Normal stop_name
// ---------------------------------------------------------------------------

#[test]
fn test_normal_stop_name() {
    let feed = GtfsFeed {
        stops: vec![make_stop("S1")],
        ..Default::default()
    };
    let errors = GoogleCoordinatesInStopNameRule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// 8 — Identical route colors
// ---------------------------------------------------------------------------

#[test]
fn test_identical_route_colors() {
    let mut route = make_route("R1");
    route.route_color = Some(Color("FF0000".into()));
    route.route_text_color = Some(Color("FF0000".into()));
    let feed = GtfsFeed {
        routes: vec![route],
        ..Default::default()
    };
    let errors = GoogleIdenticalRouteColorsRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "google_identical_route_colors");
    assert_eq!(errors[0].severity, Severity::Warning);
    assert_eq!(errors[0].value.as_deref(), Some("FF0000"));
}

// ---------------------------------------------------------------------------
// 9 — Different route colors
// ---------------------------------------------------------------------------

#[test]
fn test_different_route_colors() {
    let feed = GtfsFeed {
        routes: vec![make_route("R1")],
        ..Default::default()
    };
    let errors = GoogleIdenticalRouteColorsRule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// 10 — Invisible route text (white on white)
// ---------------------------------------------------------------------------

#[test]
fn test_invisible_route_text() {
    let mut route = make_route("R1");
    route.route_color = Some(Color("FFFFFF".into()));
    route.route_text_color = Some(Color("FFFFFF".into()));
    let feed = GtfsFeed {
        routes: vec![route],
        ..Default::default()
    };
    let errors = GoogleIdenticalRouteColorsRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "google_identical_route_colors");
}

// ---------------------------------------------------------------------------
// 11 — Trip without shape when shapes exist
// ---------------------------------------------------------------------------

#[test]
fn test_trip_without_shape() {
    let mut trip = make_trip("T1");
    trip.shape_id = None;
    let feed = GtfsFeed {
        trips: vec![trip],
        shapes: vec![make_shape("SH1", 1)],
        ..Default::default()
    };
    let errors = ConveyalTripWithoutShapeRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "conveyal_trip_without_shape");
    assert_eq!(errors[0].severity, Severity::Warning);
    assert_eq!(errors[0].file_name.as_deref(), Some("trips.txt"));
    assert_eq!(errors[0].field_name.as_deref(), Some("shape_id"));
}

// ---------------------------------------------------------------------------
// 12 — No feed_info.txt (feed_info rules skipped)
// ---------------------------------------------------------------------------

#[test]
fn test_no_feed_info() {
    let feed = GtfsFeed {
        feed_info: None,
        ..Default::default()
    };
    let etalab_errors = EtalabMissingContactRule.validate(&feed);
    let otp_errors = OtpMissingFeedVersionRule.validate(&feed);
    assert!(etalab_errors.is_empty());
    assert!(otp_errors.is_empty());
}
