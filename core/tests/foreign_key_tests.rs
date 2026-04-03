//! Tests for section 5 — Foreign Key Validation.

use chrono::NaiveDate;
use headway_core::models::*;
use headway_core::validation::foreign_key::calendar_dates_service::CalendarDatesServiceFkRule;
use headway_core::validation::foreign_key::frequencies_trip::FrequenciesTripFkRule;
use headway_core::validation::foreign_key::routes_agency::RoutesAgencyFkRule;
use headway_core::validation::foreign_key::stop_times_stop::StopTimesStopFkRule;
use headway_core::validation::foreign_key::stop_times_trip::StopTimesTripFkRule;
use headway_core::validation::foreign_key::stops_level::StopsLevelFkRule;
use headway_core::validation::foreign_key::stops_parent_station::StopsParentStationFkRule;
use headway_core::validation::foreign_key::trips_route::TripsRouteFkRule;
use headway_core::validation::foreign_key::trips_service::TripsServiceFkRule;
use headway_core::validation::foreign_key::trips_shape::TripsShapeFkRule;
use headway_core::validation::{Severity, ValidationError, ValidationRule};

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

fn make_stop_with_level(id: &str, level_id: Option<&str>) -> Stop {
    let mut stop = make_stop(id, Some("Stop"), Some(45.0), Some(-73.0), None, None);
    stop.level_id = level_id.map(|s| LevelId::from(s.to_string()));
    stop
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

fn make_stop_time(trip_id: &str, seq: u32, stop_id: &str) -> StopTime {
    StopTime {
        trip_id: TripId::from(trip_id.to_string()),
        arrival_time: Some(GtfsTime::from_hms(8, 0, 0)),
        departure_time: Some(GtfsTime::from_hms(8, 0, 0)),
        stop_id: StopId::from(stop_id.to_string()),
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

fn make_calendar(service_id: &str) -> Calendar {
    Calendar {
        service_id: ServiceId::from(service_id.to_string()),
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        start_date: GtfsDate(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
        end_date: GtfsDate(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
    }
}

fn make_calendar_date(service_id: &str) -> CalendarDate {
    CalendarDate {
        service_id: ServiceId::from(service_id.to_string()),
        date: GtfsDate(NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()),
        exception_type: ExceptionType::Added,
    }
}

fn make_shape(shape_id: &str) -> Shape {
    Shape {
        shape_id: ShapeId::from(shape_id.to_string()),
        shape_pt_lat: Latitude(45.0),
        shape_pt_lon: Longitude(-73.0),
        shape_pt_sequence: 1,
        shape_dist_traveled: None,
    }
}

fn make_frequency(trip_id: &str) -> Frequency {
    Frequency {
        trip_id: TripId::from(trip_id.to_string()),
        start_time: GtfsTime::from_hms(6, 0, 0),
        end_time: GtfsTime::from_hms(22, 0, 0),
        headway_secs: 600,
        exact_times: None,
    }
}

fn make_level(level_id: &str) -> Level {
    Level {
        level_id: LevelId::from(level_id.to_string()),
        level_index: 0.0,
        level_name: None,
    }
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
    feed.calendars.push(make_calendar("SVC1"));
    feed.trips.push(make_trip("R1", "SVC1", "T1", None));
    feed.stop_times.push(make_stop_time("T1", 1, "S1"));
    feed.stop_times.push(make_stop_time("T1", 2, "S2"));
    feed
}

fn count_errors(errors: &[ValidationError], severity: Severity) -> usize {
    errors.iter().filter(|e| e.severity == severity).count()
}

// ---------------------------------------------------------------------------
// All rules — valid feed
// ---------------------------------------------------------------------------

#[test]
fn valid_feed_no_fk_errors() {
    let feed = valid_feed();
    let rules: Vec<Box<dyn ValidationRule>> = vec![
        Box::new(RoutesAgencyFkRule),
        Box::new(TripsRouteFkRule),
        Box::new(TripsServiceFkRule),
        Box::new(TripsShapeFkRule),
        Box::new(StopTimesTripFkRule),
        Box::new(StopTimesStopFkRule),
        Box::new(CalendarDatesServiceFkRule),
        Box::new(FrequenciesTripFkRule),
        Box::new(StopsParentStationFkRule),
        Box::new(StopsLevelFkRule),
    ];
    let all_errors: Vec<_> = rules.iter().flat_map(|r| r.validate(&feed)).collect();
    assert!(
        all_errors.is_empty(),
        "Expected 0 errors, got: {all_errors:?}"
    );
}

// ---------------------------------------------------------------------------
// routes.agency_id → agency (CA1, CA2)
// ---------------------------------------------------------------------------

#[test]
fn route_agency_orphan() {
    let mut feed = valid_feed();
    feed.routes
        .push(make_route("R2", Some("AG99"), Some("2"), None));

    let errors = RoutesAgencyFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].file_name.as_deref(), Some("routes.txt"));
    assert_eq!(errors[0].field_name.as_deref(), Some("agency_id"));
    assert_eq!(errors[0].value.as_deref(), Some("AG99"));
    assert_eq!(errors[0].rule_id, "foreign_key_violation");
    assert_eq!(errors[0].section, "5");
}

#[test]
fn route_agency_implicit() {
    let mut feed = valid_feed();
    // Single agency, route with no agency_id
    feed.routes.push(make_route("R2", None, Some("2"), None));

    let errors = RoutesAgencyFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 0);
}

// ---------------------------------------------------------------------------
// trips.route_id → routes (CA3)
// ---------------------------------------------------------------------------

#[test]
fn trip_route_orphan() {
    let mut feed = valid_feed();
    feed.trips.push(make_trip("R99", "SVC1", "T2", None));

    let errors = TripsRouteFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].value.as_deref(), Some("R99"));
    assert_eq!(errors[0].field_name.as_deref(), Some("route_id"));
}

// ---------------------------------------------------------------------------
// trips.service_id → calendar / calendar_dates (CA4)
// ---------------------------------------------------------------------------

#[test]
fn trip_service_orphan() {
    let mut feed = valid_feed();
    feed.trips.push(make_trip("R1", "SVC99", "T2", None));

    let errors = TripsServiceFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].value.as_deref(), Some("SVC99"));
    assert_eq!(errors[0].field_name.as_deref(), Some("service_id"));
}

#[test]
fn trip_service_in_calendar_dates_only() {
    let mut feed = valid_feed();
    feed.calendar_dates.push(make_calendar_date("SVC_CD"));
    feed.trips.push(make_trip("R1", "SVC_CD", "T2", None));

    let errors = TripsServiceFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 0);
}

// ---------------------------------------------------------------------------
// trips.shape_id → shapes (CA5)
// ---------------------------------------------------------------------------

#[test]
fn trip_shape_orphan() {
    let mut feed = valid_feed();
    feed.trips.push(make_trip("R1", "SVC1", "T2", Some("SH99")));

    let errors = TripsShapeFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].value.as_deref(), Some("SH99"));
    assert_eq!(errors[0].field_name.as_deref(), Some("shape_id"));
}

#[test]
fn trip_shape_empty() {
    let feed = valid_feed(); // trips have shape_id = None
    let errors = TripsShapeFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 0);
}

// ---------------------------------------------------------------------------
// stop_times.trip_id → trips (CA6)
// ---------------------------------------------------------------------------

#[test]
fn stop_time_trip_orphan() {
    let mut feed = valid_feed();
    feed.stop_times.push(make_stop_time("T99", 1, "S1"));

    let errors = StopTimesTripFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].value.as_deref(), Some("T99"));
    assert_eq!(errors[0].field_name.as_deref(), Some("trip_id"));
    assert_eq!(errors[0].file_name.as_deref(), Some("stop_times.txt"));
}

// ---------------------------------------------------------------------------
// stop_times.stop_id → stops (CA7)
// ---------------------------------------------------------------------------

#[test]
fn stop_time_stop_orphan() {
    let mut feed = valid_feed();
    feed.stop_times.push(make_stop_time("T1", 3, "S99"));

    let errors = StopTimesStopFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].value.as_deref(), Some("S99"));
    assert_eq!(errors[0].field_name.as_deref(), Some("stop_id"));
}

// ---------------------------------------------------------------------------
// frequencies.trip_id → trips (CA9)
// ---------------------------------------------------------------------------

#[test]
fn frequency_trip_orphan() {
    let mut feed = valid_feed();
    feed.frequencies.push(make_frequency("T99"));

    let errors = FrequenciesTripFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].value.as_deref(), Some("T99"));
    assert_eq!(errors[0].field_name.as_deref(), Some("trip_id"));
    assert_eq!(errors[0].file_name.as_deref(), Some("frequencies.txt"));
}

// ---------------------------------------------------------------------------
// stops.parent_station → stops (CA10)
// ---------------------------------------------------------------------------

#[test]
fn parent_station_orphan() {
    let mut feed = valid_feed();
    feed.stops.push(make_stop(
        "S3",
        Some("Stop 3"),
        Some(45.2),
        Some(-73.2),
        None,
        Some("S99"),
    ));

    let errors = StopsParentStationFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].value.as_deref(), Some("S99"));
    assert_eq!(errors[0].field_name.as_deref(), Some("parent_station"));
}

#[test]
fn parent_station_wrong_type() {
    let mut feed = valid_feed();
    // S1 has location_type=None (defaults to StopOrPlatform=0)
    feed.stops.push(make_stop(
        "S3",
        Some("Stop 3"),
        Some(45.2),
        Some(-73.2),
        None,
        Some("S1"), // S1 is not a Station
    ));

    let errors = StopsParentStationFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert!(errors[0].message.contains("location_type=1"));
}

#[test]
fn parent_station_valid() {
    let mut feed = valid_feed();
    feed.stops.push(make_stop(
        "STATION1",
        Some("Station"),
        Some(45.2),
        Some(-73.2),
        Some(LocationType::Station),
        None,
    ));
    feed.stops.push(make_stop(
        "S3",
        Some("Platform"),
        Some(45.2),
        Some(-73.2),
        Some(LocationType::StopOrPlatform),
        Some("STATION1"),
    ));

    let errors = StopsParentStationFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 0);
}

// ---------------------------------------------------------------------------
// stops.level_id → levels (CA11)
// ---------------------------------------------------------------------------

#[test]
fn level_id_orphan() {
    let mut feed = valid_feed();
    feed.loaded_files.insert("levels.txt".to_string());
    feed.levels.push(make_level("L1"));
    feed.stops.push(make_stop_with_level("S3", Some("L99")));

    let errors = StopsLevelFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].value.as_deref(), Some("L99"));
    assert_eq!(errors[0].field_name.as_deref(), Some("level_id"));
}

// ---------------------------------------------------------------------------
// Multiple orphans / performance
// ---------------------------------------------------------------------------

#[test]
fn multiple_orphans() {
    let mut feed = valid_feed();
    feed.stop_times.push(make_stop_time("T99", 1, "S1"));
    feed.stop_times.push(make_stop_time("T98", 1, "S1"));
    feed.stop_times.push(make_stop_time("T97", 1, "S1"));

    let errors = StopTimesTripFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 3);
}

#[test]
#[ignore = "long-running performance test"]
fn perf_100k_stop_times() {
    let mut feed = valid_feed();

    // Create 1000 trips
    for i in 0..1000 {
        feed.trips
            .push(make_trip("R1", "SVC1", &format!("TRIP{i}"), None));
    }

    // Create 100k stop_times spread across those trips
    feed.stop_times.clear();
    for i in 0..100_000_u32 {
        let trip_id = format!("TRIP{}", i % 1000);
        feed.stop_times
            .push(make_stop_time(&trip_id, i % 100, "S1"));
    }

    let start = std::time::Instant::now();
    let errors = StopTimesTripFkRule.validate(&feed);
    let elapsed = start.elapsed();

    assert!(errors.is_empty());
    assert!(
        elapsed.as_secs() < 2,
        "FK validation took {elapsed:?}, expected < 2s",
    );
}

// ---------------------------------------------------------------------------
// calendar_dates.service_id → calendar (CA8)
// ---------------------------------------------------------------------------

#[test]
fn calendar_dates_service_warning_when_calendar_present() {
    let mut feed = valid_feed();
    feed.loaded_files.insert("calendar.txt".to_string());
    feed.calendar_dates.push(make_calendar_date("SVC_UNKNOWN"));

    let errors = CalendarDatesServiceFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Warning), 1);
    assert_eq!(errors[0].value.as_deref(), Some("SVC_UNKNOWN"));
    assert_eq!(errors[0].severity, Severity::Warning);
}

#[test]
fn calendar_dates_service_no_check_without_calendar() {
    let mut feed = valid_feed();
    // Do NOT insert "calendar.txt" into loaded_files
    feed.calendar_dates.push(make_calendar_date("SVC_UNKNOWN"));

    let errors = CalendarDatesServiceFkRule.validate(&feed);
    assert!(errors.is_empty());
}

#[test]
fn level_id_skipped_without_levels_file() {
    let mut feed = valid_feed();
    // Do NOT insert "levels.txt" into loaded_files
    feed.stops.push(make_stop_with_level("S3", Some("L99")));

    let errors = StopsLevelFkRule.validate(&feed);
    assert!(errors.is_empty());
}

#[test]
fn trip_shape_valid() {
    let mut feed = valid_feed();
    feed.shapes.push(make_shape("SH1"));
    feed.trips.push(make_trip("R1", "SVC1", "T2", Some("SH1")));

    let errors = TripsShapeFkRule.validate(&feed);
    assert!(errors.is_empty());
}

#[test]
fn frequency_trip_valid() {
    let mut feed = valid_feed();
    feed.frequencies.push(make_frequency("T1"));

    let errors = FrequenciesTripFkRule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// Error context (CA12, CA13)
// ---------------------------------------------------------------------------

#[test]
fn error_includes_correct_line_number() {
    let mut feed = valid_feed();
    // feed already has 2 stop_times (lines 2, 3). Add orphan at index 2 → line 4.
    feed.stop_times.push(make_stop_time("T99", 3, "S1"));

    let errors = StopTimesTripFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].line_number, Some(4));
}
