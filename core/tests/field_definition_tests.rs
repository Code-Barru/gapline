//! Tests for section 4 — Field Definition Validation.

use headway_core::models::*;
use headway_core::validation::field_definition::agency::AgencyFieldDefinitionRule;
use headway_core::validation::field_definition::routes::RoutesFieldDefinitionRule;
use headway_core::validation::field_definition::stop_times::StopTimesFieldDefinitionRule;
use headway_core::validation::field_definition::stops::StopsFieldDefinitionRule;
use headway_core::validation::field_definition::trips::TripsFieldDefinitionRule;
use headway_core::validation::{Severity, ValidationRule};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_agency(id: Option<&str>, name: &str, url: &str, tz: &str) -> Agency {
    Agency {
        agency_id: id.map(|s| AgencyId::from(s.to_string())),
        agency_name: name.to_string(),
        agency_url: Url::from(url.to_string()),
        agency_timezone: Timezone::from(tz.to_string()),
        agency_lang: None,
        agency_phone: None,
        agency_fare_url: None,
        agency_email: None,
    }
}

fn make_stop(
    id: &str,
    name: Option<&str>,
    lat: Option<f64>,
    lon: Option<f64>,
    loc_type: Option<LocationType>,
    parent: Option<&str>,
) -> Stop {
    Stop {
        stop_id: StopId::from(id.to_string()),
        stop_code: None,
        stop_name: name.map(std::string::ToString::to_string),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: lat.map(Latitude),
        stop_lon: lon.map(Longitude),
        zone_id: None,
        stop_url: None,
        location_type: loc_type,
        parent_station: parent.map(|s| StopId::from(s.to_string())),
        stop_timezone: None,
        wheelchair_boarding: None,
        level_id: None,
        platform_code: None,
    }
}

fn make_route(id: &str, agency_id: Option<&str>, short: Option<&str>, long: Option<&str>) -> Route {
    Route {
        route_id: RouteId::from(id.to_string()),
        agency_id: agency_id.map(|s| AgencyId::from(s.to_string())),
        route_short_name: short.map(std::string::ToString::to_string),
        route_long_name: long.map(std::string::ToString::to_string),
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

fn make_trip(route_id: &str, service_id: &str, trip_id: &str, shape_id: Option<&str>) -> Trip {
    Trip {
        route_id: RouteId::from(route_id.to_string()),
        service_id: ServiceId::from(service_id.to_string()),
        trip_id: TripId::from(trip_id.to_string()),
        trip_headsign: None,
        trip_short_name: None,
        direction_id: None,
        block_id: None,
        shape_id: shape_id.map(|s| ShapeId::from(s.to_string())),
        wheelchair_accessible: None,
        bikes_allowed: None,
    }
}

fn make_stop_time(
    trip_id: &str,
    seq: u32,
    arrival: Option<GtfsTime>,
    departure: Option<GtfsTime>,
    timepoint: Option<Timepoint>,
) -> StopTime {
    StopTime {
        trip_id: TripId::from(trip_id.to_string()),
        arrival_time: arrival,
        departure_time: departure,
        stop_id: StopId::from(format!("S{seq}")),
        stop_sequence: seq,
        stop_headsign: None,
        pickup_type: None,
        drop_off_type: None,
        continuous_pickup: None,
        continuous_drop_off: None,
        shape_dist_traveled: None,
        timepoint,
    }
}

fn time(h: u32, m: u32) -> GtfsTime {
    GtfsTime::from_hms(h, m, 0)
}

fn valid_feed() -> GtfsFeed {
    let mut feed = GtfsFeed::default();
    feed.agencies.push(make_agency(
        Some("A1"),
        "Agency",
        "https://example.com",
        "America/Montreal",
    ));
    feed.stops.push(make_stop(
        "S1",
        Some("Stop 1"),
        Some(45.0),
        Some(-73.0),
        None,
        None,
    ));
    feed.stops.push(make_stop(
        "S2",
        Some("Stop 2"),
        Some(45.1),
        Some(-73.1),
        None,
        None,
    ));
    feed.routes
        .push(make_route("R1", Some("A1"), Some("1"), None));
    feed.trips.push(make_trip("R1", "SVC1", "T1", None));
    feed.stop_times.push(make_stop_time(
        "T1",
        1,
        Some(time(8, 0)),
        Some(time(8, 0)),
        None,
    ));
    feed.stop_times.push(make_stop_time(
        "T1",
        2,
        Some(time(8, 10)),
        Some(time(8, 10)),
        None,
    ));
    feed
}

fn count_errors(errors: &[headway_core::validation::ValidationError], severity: Severity) -> usize {
    errors.iter().filter(|e| e.severity == severity).count()
}

fn errors_for_field<'a>(
    errors: &'a [headway_core::validation::ValidationError],
    field: &str,
) -> Vec<&'a headway_core::validation::ValidationError> {
    errors
        .iter()
        .filter(|e| e.field_name.as_deref() == Some(field))
        .collect()
}

// ---------------------------------------------------------------------------
// Test 1 — Valid feed produces 0 errors
// ---------------------------------------------------------------------------

#[test]
fn valid_feed_produces_no_errors() {
    let feed = valid_feed();
    let rules: Vec<Box<dyn ValidationRule>> = vec![
        Box::new(AgencyFieldDefinitionRule),
        Box::new(StopsFieldDefinitionRule),
        Box::new(RoutesFieldDefinitionRule),
        Box::new(TripsFieldDefinitionRule),
        Box::new(StopTimesFieldDefinitionRule),
    ];
    let all_errors: Vec<_> = rules.iter().flat_map(|r| r.validate(&feed)).collect();
    assert!(
        all_errors.is_empty(),
        "Expected 0 errors, got: {all_errors:?}"
    );
}

// ---------------------------------------------------------------------------
// Agency tests (CA1, CA2)
// ---------------------------------------------------------------------------

#[test]
fn agency_name_missing() {
    let mut feed = valid_feed();
    feed.agencies[0].agency_name = String::new();
    let errors = AgencyFieldDefinitionRule.validate(&feed);
    assert_eq!(errors_for_field(&errors, "agency_name").len(), 1);
}

#[test]
fn agency_url_missing() {
    let mut feed = valid_feed();
    feed.agencies[0].agency_url = Url::from(String::new());
    let errors = AgencyFieldDefinitionRule.validate(&feed);
    assert_eq!(errors_for_field(&errors, "agency_url").len(), 1);
}

#[test]
fn agency_timezone_missing() {
    let mut feed = valid_feed();
    feed.agencies[0].agency_timezone = Timezone::from(String::new());
    let errors = AgencyFieldDefinitionRule.validate(&feed);
    assert_eq!(errors_for_field(&errors, "agency_timezone").len(), 1);
}

#[test]
fn agency_id_missing_multiple_agencies() {
    let mut feed = valid_feed();
    feed.agencies.push(make_agency(
        None,
        "Agency 2",
        "https://example2.com",
        "America/Montreal",
    ));
    feed.agencies[0].agency_id = None;
    let errors = AgencyFieldDefinitionRule.validate(&feed);
    assert_eq!(errors_for_field(&errors, "agency_id").len(), 2);
}

#[test]
fn agency_id_optional_single_agency() {
    let mut feed = valid_feed();
    feed.agencies[0].agency_id = None;
    let errors = AgencyFieldDefinitionRule.validate(&feed);
    assert!(errors_for_field(&errors, "agency_id").is_empty());
}

// ---------------------------------------------------------------------------
// Stops tests (CA3-CA6)
// ---------------------------------------------------------------------------

#[test]
fn stop_name_missing_type_0() {
    let mut feed = valid_feed();
    feed.stops[0].stop_name = None;
    let errors = StopsFieldDefinitionRule.validate(&feed);
    assert_eq!(errors_for_field(&errors, "stop_name").len(), 1);
}

#[test]
fn stop_lat_missing_type_1() {
    let mut feed = valid_feed();
    feed.stops[0].location_type = Some(LocationType::Station);
    feed.stops[0].stop_lat = None;
    let errors = StopsFieldDefinitionRule.validate(&feed);
    assert!(!errors_for_field(&errors, "stop_lat").is_empty());
}

#[test]
fn stop_name_not_required_type_3() {
    let mut feed = valid_feed();
    feed.stops[0] = make_stop(
        "S1",
        None,
        None,
        None,
        Some(LocationType::GenericNode),
        Some("P1"),
    );
    let errors = StopsFieldDefinitionRule.validate(&feed);
    assert!(errors_for_field(&errors, "stop_name").is_empty());
}

#[test]
fn parent_station_required_type_2() {
    let mut feed = valid_feed();
    feed.stops[0] = make_stop(
        "S1",
        None,
        None,
        None,
        Some(LocationType::EntranceExit),
        None,
    );
    let errors = StopsFieldDefinitionRule.validate(&feed);
    assert_eq!(errors_for_field(&errors, "parent_station").len(), 1);
    assert_eq!(
        errors_for_field(&errors, "parent_station")[0].severity,
        Severity::Error
    );
}

#[test]
fn parent_station_forbidden_type_1() {
    let mut feed = valid_feed();
    feed.stops[0] = make_stop(
        "S1",
        Some("Station"),
        Some(45.0),
        Some(-73.0),
        Some(LocationType::Station),
        Some("P1"),
    );
    let errors = StopsFieldDefinitionRule.validate(&feed);
    let ps_errors = errors_for_field(&errors, "parent_station");
    assert_eq!(ps_errors.len(), 1);
    assert_eq!(ps_errors[0].severity, Severity::Warning);
}

// ---------------------------------------------------------------------------
// Routes tests (CA7-CA9)
// ---------------------------------------------------------------------------

#[test]
fn route_both_names_empty() {
    let mut feed = valid_feed();
    feed.routes[0].route_short_name = None;
    feed.routes[0].route_long_name = None;
    let errors = RoutesFieldDefinitionRule.validate(&feed);
    assert_eq!(errors_for_field(&errors, "route_short_name").len(), 1);
}

#[test]
fn route_short_name_only_is_valid() {
    let mut feed = valid_feed();
    feed.routes[0].route_short_name = Some("A".to_string());
    feed.routes[0].route_long_name = None;
    let errors = RoutesFieldDefinitionRule.validate(&feed);
    assert!(errors_for_field(&errors, "route_short_name").is_empty());
}

#[test]
fn route_agency_id_required_multiple_agencies() {
    let mut feed = valid_feed();
    feed.agencies.push(make_agency(
        Some("A2"),
        "Agency 2",
        "https://example2.com",
        "America/Montreal",
    ));
    feed.routes[0].agency_id = None;
    let errors = RoutesFieldDefinitionRule.validate(&feed);
    assert_eq!(errors_for_field(&errors, "agency_id").len(), 1);
}

// ---------------------------------------------------------------------------
// Trips tests (CA10-CA11)
// ---------------------------------------------------------------------------

#[test]
fn trip_shape_id_required_when_shapes_present() {
    let mut feed = valid_feed();
    feed.loaded_files.insert("shapes.txt".to_string());
    feed.trips[0].shape_id = None;
    let errors = TripsFieldDefinitionRule.validate(&feed);
    assert_eq!(errors_for_field(&errors, "shape_id").len(), 1);
}

#[test]
fn trip_shape_id_optional_when_shapes_absent() {
    let mut feed = valid_feed();
    // shapes.txt not in loaded_files
    feed.trips[0].shape_id = None;
    let errors = TripsFieldDefinitionRule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// StopTimes tests (CA12-CA14)
// ---------------------------------------------------------------------------

#[test]
fn stop_time_first_stop_missing_arrival() {
    let mut feed = valid_feed();
    feed.stop_times[0].arrival_time = None;
    let errors = StopTimesFieldDefinitionRule.validate(&feed);
    assert_eq!(errors_for_field(&errors, "arrival_time").len(), 1);
}

#[test]
fn stop_time_last_stop_missing_departure() {
    let mut feed = valid_feed();
    feed.stop_times[1].departure_time = None;
    let errors = StopTimesFieldDefinitionRule.validate(&feed);
    assert_eq!(errors_for_field(&errors, "departure_time").len(), 1);
}

#[test]
fn stop_time_intermediate_missing_arrival_ok() {
    let mut feed = valid_feed();
    // Add a third stop so index 1 becomes intermediate
    feed.stop_times.push(make_stop_time(
        "T1",
        3,
        Some(time(8, 20)),
        Some(time(8, 20)),
        None,
    ));
    feed.stop_times[1].arrival_time = None;
    feed.stop_times[1].departure_time = None;
    let errors = StopTimesFieldDefinitionRule.validate(&feed);
    assert!(
        errors.is_empty(),
        "Intermediate stops should not require times: {errors:?}"
    );
}

#[test]
fn stop_time_timepoint_exact_requires_times() {
    let mut feed = valid_feed();
    // Add third stop so index 1 is intermediate
    feed.stop_times.push(make_stop_time(
        "T1",
        3,
        Some(time(8, 20)),
        Some(time(8, 20)),
        None,
    ));
    feed.stop_times[1].timepoint = Some(Timepoint::Exact);
    feed.stop_times[1].arrival_time = None;
    feed.stop_times[1].departure_time = None;
    let errors = StopTimesFieldDefinitionRule.validate(&feed);
    assert_eq!(errors_for_field(&errors, "arrival_time").len(), 1);
    assert_eq!(errors_for_field(&errors, "departure_time").len(), 1);
}

// ---------------------------------------------------------------------------
// Multi-file cumulation (Test 20)
// ---------------------------------------------------------------------------

#[test]
fn errors_from_multiple_files_are_all_reported() {
    let mut feed = valid_feed();
    // agency error
    feed.agencies[0].agency_name = String::new();
    // stops error
    feed.stops[0].stop_name = None;
    // routes error
    feed.routes[0].route_short_name = None;
    feed.routes[0].route_long_name = None;

    let rules: Vec<Box<dyn ValidationRule>> = vec![
        Box::new(AgencyFieldDefinitionRule),
        Box::new(StopsFieldDefinitionRule),
        Box::new(RoutesFieldDefinitionRule),
    ];
    let all_errors: Vec<_> = rules.iter().flat_map(|r| r.validate(&feed)).collect();
    assert!(
        count_errors(&all_errors, Severity::Error) >= 3,
        "Expected at least 3 errors across files, got: {all_errors:?}"
    );
}
