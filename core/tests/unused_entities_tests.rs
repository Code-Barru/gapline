//! Tests for unused entity detection rules.

use gapline_core::models::*;
use gapline_core::validation::ValidationRule;
use gapline_core::validation::schedule_time_validation::unused_entities::{
    UnusedAgencyRule, UnusedFareRule, UnusedRouteRule, UnusedServiceRule, UnusedShapeRule,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_route(route_id: &str) -> Route {
    Route {
        route_id: RouteId::from(route_id),
        agency_id: None,
        route_short_name: None,
        route_long_name: None,
        route_desc: None,
        route_type: RouteType::Bus,
        route_url: None,
        route_color: None,
        route_text_color: None,
        route_sort_order: None,
        continuous_pickup: None,
        continuous_drop_off: None,
        network_id: None,
    }
}

fn make_route_with_agency(route_id: &str, agency_id: &str) -> Route {
    Route {
        route_id: RouteId::from(route_id),
        agency_id: Some(AgencyId::from(agency_id)),
        route_short_name: None,
        route_long_name: None,
        route_desc: None,
        route_type: RouteType::Bus,
        route_url: None,
        route_color: None,
        route_text_color: None,
        route_sort_order: None,
        continuous_pickup: None,
        continuous_drop_off: None,
        network_id: None,
    }
}

fn make_trip(trip_id: &str, route_id: &str, service_id: &str) -> Trip {
    Trip {
        route_id: RouteId::from(route_id),
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

fn make_trip_with_shape(trip_id: &str, route_id: &str, service_id: &str, shape_id: &str) -> Trip {
    Trip {
        route_id: RouteId::from(route_id),
        service_id: ServiceId::from(service_id),
        trip_id: TripId::from(trip_id),
        trip_headsign: None,
        trip_short_name: None,
        direction_id: None,
        block_id: None,
        shape_id: Some(ShapeId::from(shape_id)),
        wheelchair_accessible: None,
        bikes_allowed: None,
    }
}

fn make_shape_point(shape_id: &str, seq: u32) -> Shape {
    Shape {
        shape_id: ShapeId::from(shape_id),
        shape_pt_lat: Latitude(45.5),
        shape_pt_lon: Longitude(-73.5),
        shape_pt_sequence: seq,
        shape_dist_traveled: None,
    }
}

fn make_calendar(service_id: &str) -> Calendar {
    use chrono::NaiveDate;
    Calendar {
        service_id: ServiceId::from(service_id),
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        start_date: GtfsDate(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        end_date: GtfsDate(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
    }
}

fn make_calendar_date(service_id: &str) -> CalendarDate {
    use chrono::NaiveDate;
    CalendarDate {
        service_id: ServiceId::from(service_id),
        date: GtfsDate(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        exception_type: ExceptionType::Added,
    }
}

fn make_agency(agency_id: &str) -> Agency {
    Agency {
        agency_id: Some(AgencyId::from(agency_id)),
        agency_name: format!("Agency {agency_id}"),
        agency_url: Url::from("https://example.com"),
        agency_timezone: Timezone::from("America/Montreal"),
        agency_lang: None,
        agency_phone: None,
        agency_fare_url: None,
        agency_email: None,
    }
}

fn make_fare_attribute(fare_id: &str) -> FareAttribute {
    FareAttribute {
        fare_id: FareId::from(fare_id),
        price: 3.50,
        currency_type: CurrencyCode::from("CAD"),
        payment_method: 0,
        transfers: None,
        agency_id: None,
        transfer_duration: None,
    }
}

fn make_fare_rule(fare_id: &str) -> FareRule {
    FareRule {
        fare_id: FareId::from(fare_id),
        route_id: None,
        origin_id: None,
        destination_id: None,
        contains_id: None,
    }
}

// ---------------------------------------------------------------------------
// Test 1: unused route
// ---------------------------------------------------------------------------

#[test]
fn unused_route() {
    let feed = GtfsFeed {
        routes: vec![make_route("R1")],
        trips: vec![], // no trips reference R1
        ..Default::default()
    };
    let errors = UnusedRouteRule.validate(&feed);
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.rule_id == "unused_route")
            .count(),
        1,
    );
}

// ---------------------------------------------------------------------------
// Test 2: used route — no warning
// ---------------------------------------------------------------------------

#[test]
fn used_route_no_warning() {
    let feed = GtfsFeed {
        routes: vec![make_route("R1")],
        trips: vec![make_trip("T1", "R1", "S1")],
        ..Default::default()
    };
    let errors = UnusedRouteRule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// Test 3: unused shape
// ---------------------------------------------------------------------------

#[test]
fn unused_shape() {
    let feed = GtfsFeed {
        shapes: vec![make_shape_point("SH1", 1), make_shape_point("SH1", 2)],
        trips: vec![], // no trips reference SH1
        ..Default::default()
    };
    let errors = UnusedShapeRule.validate(&feed);
    let unused: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "unused_shape")
        .collect();
    assert_eq!(unused.len(), 1); // only 1 warning even though 2 shape points
}

// ---------------------------------------------------------------------------
// Test 4: unused service from calendar
// ---------------------------------------------------------------------------

#[test]
fn unused_service_from_calendar() {
    let feed = GtfsFeed {
        calendars: vec![make_calendar("SVC1")],
        trips: vec![], // no trips reference SVC1
        ..Default::default()
    };
    let errors = UnusedServiceRule.validate(&feed);
    let unused: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "unused_service")
        .collect();
    assert_eq!(unused.len(), 1);
    assert_eq!(unused[0].file_name.as_deref(), Some("calendar.txt"));
}

// ---------------------------------------------------------------------------
// Test 5: unused service from calendar_dates only
// ---------------------------------------------------------------------------

#[test]
fn unused_service_from_calendar_dates() {
    let feed = GtfsFeed {
        calendar_dates: vec![make_calendar_date("SVC2")],
        trips: vec![], // no trips reference SVC2
        ..Default::default()
    };
    let errors = UnusedServiceRule.validate(&feed);
    let unused: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "unused_service")
        .collect();
    assert_eq!(unused.len(), 1);
    assert_eq!(unused[0].file_name.as_deref(), Some("calendar_dates.txt"));
}

// ---------------------------------------------------------------------------
// Test 6: unused agency with >1 agency
// ---------------------------------------------------------------------------

#[test]
fn unused_agency_multiple_agencies() {
    let feed = GtfsFeed {
        agencies: vec![make_agency("AG1"), make_agency("AG2")],
        routes: vec![make_route_with_agency("R1", "AG1")], // only AG1 used
        ..Default::default()
    };
    let errors = UnusedAgencyRule.validate(&feed);
    let unused: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "unused_agency")
        .collect();
    assert_eq!(unused.len(), 1);
    assert_eq!(unused[0].value.as_deref(), Some("AG2"));
}

// ---------------------------------------------------------------------------
// Test 7: single agency is never flagged
// ---------------------------------------------------------------------------

#[test]
fn single_agency_never_flagged() {
    let feed = GtfsFeed {
        agencies: vec![make_agency("AG1")],
        routes: vec![make_route("R1")], // no explicit agency_id on route
        ..Default::default()
    };
    let errors = UnusedAgencyRule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// Test 8: unused fare
// ---------------------------------------------------------------------------

#[test]
fn unused_fare() {
    let feed = GtfsFeed {
        fare_attributes: vec![make_fare_attribute("F1")],
        fare_rules: vec![], // no fare_rules reference F1
        ..Default::default()
    };
    let errors = UnusedFareRule.validate(&feed);
    assert_eq!(
        errors.iter().filter(|e| e.rule_id == "unused_fare").count(),
        1,
    );
}

// ---------------------------------------------------------------------------
// Test 9: empty feed — no errors
// ---------------------------------------------------------------------------

#[test]
fn empty_feed_no_errors() {
    let feed = GtfsFeed::default();
    assert!(UnusedRouteRule.validate(&feed).is_empty());
    assert!(UnusedShapeRule.validate(&feed).is_empty());
    assert!(UnusedServiceRule.validate(&feed).is_empty());
    assert!(UnusedAgencyRule.validate(&feed).is_empty());
    assert!(UnusedFareRule.validate(&feed).is_empty());
}

// ---------------------------------------------------------------------------
// Test 10: used shape — no warning
// ---------------------------------------------------------------------------

#[test]
fn used_shape_no_warning() {
    let feed = GtfsFeed {
        shapes: vec![make_shape_point("SH1", 1)],
        trips: vec![make_trip_with_shape("T1", "R1", "S1", "SH1")],
        ..Default::default()
    };
    let errors = UnusedShapeRule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// Test 11: used fare — no warning
// ---------------------------------------------------------------------------

#[test]
fn used_fare_no_warning() {
    let feed = GtfsFeed {
        fare_attributes: vec![make_fare_attribute("F1")],
        fare_rules: vec![make_fare_rule("F1")],
        ..Default::default()
    };
    let errors = UnusedFareRule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// Test 12: cumulative — multiple unused entities all reported
// ---------------------------------------------------------------------------

#[test]
fn cumulative_all_reported() {
    let feed = GtfsFeed {
        routes: vec![make_route("R_UNUSED1"), make_route("R_UNUSED2")],
        shapes: vec![make_shape_point("SH_UNUSED", 1)],
        calendars: vec![make_calendar("SVC_UNUSED")],
        fare_attributes: vec![make_fare_attribute("F_UNUSED")],
        trips: vec![], // nothing references any entity
        ..Default::default()
    };
    let mut all_errors = Vec::new();
    all_errors.extend(UnusedRouteRule.validate(&feed));
    all_errors.extend(UnusedShapeRule.validate(&feed));
    all_errors.extend(UnusedServiceRule.validate(&feed));
    all_errors.extend(UnusedFareRule.validate(&feed));

    assert_eq!(
        all_errors
            .iter()
            .filter(|e| e.rule_id == "unused_route")
            .count(),
        2
    );
    assert_eq!(
        all_errors
            .iter()
            .filter(|e| e.rule_id == "unused_shape")
            .count(),
        1
    );
    assert_eq!(
        all_errors
            .iter()
            .filter(|e| e.rule_id == "unused_service")
            .count(),
        1
    );
    assert_eq!(
        all_errors
            .iter()
            .filter(|e| e.rule_id == "unused_fare")
            .count(),
        1
    );
}
