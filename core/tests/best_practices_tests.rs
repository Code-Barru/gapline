//! Tests for section 8 — best-practice validation rules.

use gapline_core::models::*;
use gapline_core::validation::best_practices::NamingThresholds;
use gapline_core::validation::best_practices::missing_agency_email::MissingAgencyEmailRule;
use gapline_core::validation::best_practices::missing_bikes_info::MissingBikesInfoRule;
use gapline_core::validation::best_practices::missing_direction_id::MissingDirectionIdRule;
use gapline_core::validation::best_practices::missing_route_colors::MissingRouteColorsRule;
use gapline_core::validation::best_practices::missing_wheelchair_info::{
    MissingWheelchairStopsRule, MissingWheelchairTripsRule,
};
use gapline_core::validation::best_practices::redundant_route_name::RedundantRouteNameRule;
use gapline_core::validation::best_practices::route_short_name_too_long::RouteShortNameTooLongRule;
use gapline_core::validation::best_practices::stop_name_all_caps::StopNameAllCapsRule;
use gapline_core::validation::{Severity, ValidationRule};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_agency() -> Agency {
    Agency {
        agency_id: Some(AgencyId::from("A1")),
        agency_name: "Test Agency".into(),
        agency_url: Url("https://example.com".into()),
        agency_timezone: Timezone("Europe/Paris".into()),
        agency_lang: None,
        agency_phone: None,
        agency_fare_url: None,
        agency_email: Some(Email("contact@example.com".into())),
    }
}

fn make_route(id: &str) -> Route {
    Route {
        route_id: RouteId::from(id),
        agency_id: None,
        route_short_name: Some("A1".into()),
        route_long_name: Some("Line A1 — Main".into()),
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
        wheelchair_boarding: Some(WheelchairAccessible::Some),
        level_id: None,
        platform_code: None,
    }
}

fn make_trip(id: &str) -> Trip {
    Trip {
        route_id: RouteId::from("R1"),
        service_id: ServiceId::from("S1"),
        trip_id: TripId::from(id),
        trip_headsign: None,
        trip_short_name: None,
        direction_id: Some(DirectionId::Outbound),
        block_id: None,
        shape_id: None,
        wheelchair_accessible: Some(WheelchairAccessible::Some),
        bikes_allowed: Some(BikesAllowed::Allowed),
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
        feed_version: None,
        feed_contact_email: None,
        feed_contact_url: None,
    }
}

fn exemplary_feed() -> GtfsFeed {
    GtfsFeed {
        agencies: vec![make_agency()],
        routes: vec![make_route("R1")],
        stops: vec![make_stop("S1")],
        trips: vec![make_trip("T1")],
        feed_info: Some(make_feed_info()),
        ..Default::default()
    }
}

fn default_thresholds() -> NamingThresholds {
    NamingThresholds {
        max_route_short_name_length: 12,
    }
}

// ---------------------------------------------------------------------------
//1 — Exemplary feed (0 issues)
// ---------------------------------------------------------------------------

#[test]
fn test_exemplary_feed_no_warnings() {
    let feed = exemplary_feed();
    let rules: Vec<Box<dyn ValidationRule>> = vec![
        Box::new(MissingAgencyEmailRule),
        Box::new(MissingRouteColorsRule),
        Box::new(MissingDirectionIdRule),
        Box::new(RouteShortNameTooLongRule::new(default_thresholds())),
        Box::new(StopNameAllCapsRule),
        Box::new(RedundantRouteNameRule),
        Box::new(MissingWheelchairStopsRule),
        Box::new(MissingWheelchairTripsRule),
        Box::new(MissingBikesInfoRule),
    ];

    let errors: Vec<_> = rules.iter().flat_map(|r| r.validate(&feed)).collect();
    assert!(errors.is_empty(), "Expected 0 issues, got: {errors:?}");
}

// ---------------------------------------------------------------------------
//3 — missing agency_email
// ---------------------------------------------------------------------------

#[test]
fn test_missing_agency_email() {
    let mut agency = make_agency();
    agency.agency_email = None;
    let feed = GtfsFeed {
        agencies: vec![agency],
        ..Default::default()
    };
    let errors = MissingAgencyEmailRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "missing_agency_email");
    assert_eq!(errors[0].severity, Severity::Info);
    assert_eq!(errors[0].file_name.as_deref(), Some("agency.txt"));
    assert_eq!(errors[0].line_number, Some(2));
    assert_eq!(errors[0].field_name.as_deref(), Some("agency_email"));
}

// ---------------------------------------------------------------------------
//4 — missing route_color
// ---------------------------------------------------------------------------

#[test]
fn test_missing_route_color() {
    let mut route = make_route("R1");
    route.route_color = None;
    route.route_text_color = None;
    let feed = GtfsFeed {
        routes: vec![route],
        ..Default::default()
    };
    let errors = MissingRouteColorsRule.validate(&feed);
    assert_eq!(errors.len(), 2);
    assert!(errors.iter().all(|e| e.rule_id == "missing_route_colors"));
    assert!(errors.iter().all(|e| e.severity == Severity::Info));
    assert!(errors.iter().all(|e| e.section == "8"));
    let fields: Vec<_> = errors
        .iter()
        .filter_map(|e| e.field_name.as_deref())
        .collect();
    assert!(fields.contains(&"route_color"));
    assert!(fields.contains(&"route_text_color"));
}

// ---------------------------------------------------------------------------
//5 — missing direction_id (global)
// ---------------------------------------------------------------------------

#[test]
fn test_missing_direction_id() {
    let mut trip = make_trip("T1");
    trip.direction_id = None;
    let feed = GtfsFeed {
        trips: vec![trip],
        ..Default::default()
    };
    let errors = MissingDirectionIdRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "missing_direction_id");
    assert_eq!(errors[0].severity, Severity::Warning);
    assert_eq!(errors[0].file_name.as_deref(), Some("trips.txt"));
    assert_eq!(errors[0].field_name.as_deref(), Some("direction_id"));
    assert!(errors[0].line_number.is_none());
}

// ---------------------------------------------------------------------------
//6 — route_short_name too long
// ---------------------------------------------------------------------------

#[test]
fn test_route_short_name_too_long() {
    let mut route = make_route("R1");
    route.route_short_name = Some("ExtraLongBusRoute".into());
    let feed = GtfsFeed {
        routes: vec![route],
        ..Default::default()
    };
    let rule = RouteShortNameTooLongRule::new(default_thresholds());
    let errors = rule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "route_short_name_too_long");
    assert_eq!(errors[0].severity, Severity::Warning);
    assert_eq!(errors[0].value.as_deref(), Some("ExtraLongBusRoute"));
    assert_eq!(errors[0].line_number, Some(2));
}

// ---------------------------------------------------------------------------
//7 — route_short_name OK
// ---------------------------------------------------------------------------

#[test]
fn test_route_short_name_ok() {
    let feed = exemplary_feed();
    let rule = RouteShortNameTooLongRule::new(default_thresholds());
    let errors = rule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
//8 — stop_name all caps
// ---------------------------------------------------------------------------

#[test]
fn test_stop_name_all_caps() {
    let mut stop = make_stop("S1");
    stop.stop_name = Some("GARE CENTRALE".into());
    let feed = GtfsFeed {
        stops: vec![stop],
        ..Default::default()
    };
    let errors = StopNameAllCapsRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "stop_name_all_caps");
    assert_eq!(errors[0].severity, Severity::Warning);
    assert_eq!(errors[0].value.as_deref(), Some("GARE CENTRALE"));
    assert_eq!(errors[0].line_number, Some(2));
}

// ---------------------------------------------------------------------------
//9 — stop_name mixed case
// ---------------------------------------------------------------------------

#[test]
fn test_stop_name_mixed_case() {
    let feed = exemplary_feed();
    let errors = StopNameAllCapsRule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
//10 — redundant route name
// ---------------------------------------------------------------------------

#[test]
fn test_redundant_route_name() {
    let mut route = make_route("R1");
    route.route_short_name = Some("A1".into());
    route.route_long_name = Some("A1".into());
    let feed = GtfsFeed {
        routes: vec![route],
        ..Default::default()
    };
    let errors = RedundantRouteNameRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "redundant_route_name");
    assert_eq!(errors[0].severity, Severity::Warning);
    assert_eq!(errors[0].value.as_deref(), Some("A1"));
    assert_eq!(errors[0].line_number, Some(2));
}

// ---------------------------------------------------------------------------
//11 — missing wheelchair_boarding in stops
// ---------------------------------------------------------------------------

#[test]
fn test_missing_wheelchair_stops() {
    let mut stop = make_stop("S1");
    stop.wheelchair_boarding = None;
    let feed = GtfsFeed {
        stops: vec![stop],
        ..Default::default()
    };
    let errors = MissingWheelchairStopsRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "missing_wheelchair_info");
    assert_eq!(errors[0].severity, Severity::Warning);
    assert_eq!(errors[0].file_name.as_deref(), Some("stops.txt"));
    assert_eq!(errors[0].field_name.as_deref(), Some("wheelchair_boarding"));
    assert_eq!(errors[0].line_number, Some(2));
}

// ---------------------------------------------------------------------------
//12 — missing wheelchair_accessible in trips
// ---------------------------------------------------------------------------

#[test]
fn test_missing_wheelchair_trips() {
    let mut trip = make_trip("T1");
    trip.wheelchair_accessible = None;
    let feed = GtfsFeed {
        trips: vec![trip],
        ..Default::default()
    };
    let errors = MissingWheelchairTripsRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "missing_wheelchair_info");
    assert_eq!(errors[0].severity, Severity::Warning);
    assert_eq!(errors[0].file_name.as_deref(), Some("trips.txt"));
    assert_eq!(
        errors[0].field_name.as_deref(),
        Some("wheelchair_accessible")
    );
    assert_eq!(errors[0].line_number, Some(2));
}

// ---------------------------------------------------------------------------
//13 — missing bikes_allowed
// ---------------------------------------------------------------------------

#[test]
fn test_missing_bikes_info() {
    let mut trip = make_trip("T1");
    trip.bikes_allowed = None;
    let feed = GtfsFeed {
        trips: vec![trip],
        ..Default::default()
    };
    let errors = MissingBikesInfoRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "missing_bikes_info");
    assert_eq!(errors[0].severity, Severity::Info);
    assert_eq!(errors[0].file_name.as_deref(), Some("trips.txt"));
    assert_eq!(errors[0].field_name.as_deref(), Some("bikes_allowed"));
}

// ---------------------------------------------------------------------------
//14 — cumulative issues
// ---------------------------------------------------------------------------

#[test]
fn test_cumulative_issues() {
    let mut stop = make_stop("S1");
    stop.stop_name = Some("GARE CENTRALE".into());
    stop.wheelchair_boarding = None;

    let feed = GtfsFeed {
        feed_info: None,
        stops: vec![stop],
        ..Default::default()
    };

    let rules: Vec<Box<dyn ValidationRule>> = vec![
        Box::new(StopNameAllCapsRule),
        Box::new(MissingWheelchairStopsRule),
    ];
    let errors: Vec<_> = rules.iter().flat_map(|r| r.validate(&feed)).collect();
    assert_eq!(errors.len(), 2);

    let rule_ids: Vec<&str> = errors.iter().map(|e| e.rule_id.as_str()).collect();
    assert!(rule_ids.contains(&"stop_name_all_caps"));
    assert!(rule_ids.contains(&"missing_wheelchair_info"));

    assert!(errors.iter().all(|e| e.severity != Severity::Error));
}

// ---------------------------------------------------------------------------
//15 — custom short name threshold
// ---------------------------------------------------------------------------

#[test]
fn test_custom_short_name_threshold() {
    let mut route = make_route("R1");
    route.route_short_name = Some("Metro1".into());

    let feed = GtfsFeed {
        routes: vec![route],
        ..Default::default()
    };

    let rule = RouteShortNameTooLongRule::new(NamingThresholds {
        max_route_short_name_length: 6,
    });
    let errors = rule.validate(&feed);
    assert!(errors.is_empty()); // 6 == 6, not exceeded

    // lower the threshold so "Metro1" (6 chars) exceeds it
    let rule = RouteShortNameTooLongRule::new(NamingThresholds {
        max_route_short_name_length: 5,
    });
    let errors = rule.validate(&feed);
    assert_eq!(errors.len(), 1);
}
