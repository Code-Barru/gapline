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
// HW-018 extended FK rules
use headway_core::validation::foreign_key::attributions_refs::AttributionsRefsFkRule;
use headway_core::validation::foreign_key::fare_attributes_agency::FareAttributesAgencyFkRule;
use headway_core::validation::foreign_key::fare_rules_fare::FareRulesFareFkRule;
use headway_core::validation::foreign_key::fare_rules_route::FareRulesRouteFkRule;
use headway_core::validation::foreign_key::fare_rules_zones::FareRulesZonesFkRule;
use headway_core::validation::foreign_key::pathways_stops::PathwaysStopsFkRule;
use headway_core::validation::foreign_key::transfers_from_route::TransfersFromRouteFkRule;
use headway_core::validation::foreign_key::transfers_from_stop::TransfersFromStopFkRule;
use headway_core::validation::foreign_key::transfers_from_trip::TransfersFromTripFkRule;
use headway_core::validation::foreign_key::transfers_to_route::TransfersToRouteFkRule;
use headway_core::validation::foreign_key::transfers_to_stop::TransfersToStopFkRule;
use headway_core::validation::foreign_key::transfers_to_trip::TransfersToTripFkRule;
use headway_core::validation::foreign_key::translations_record::TranslationsRecordFkRule;
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
fn parent_station_wrong_type_not_checked_by_fk_rule() {
    let mut feed = valid_feed();
    // S1 has location_type=None (defaults to StopOrPlatform=0)
    // FK rule only checks existence, not parent type (type checks are in section 7).
    feed.stops.push(make_stop(
        "S3",
        Some("Stop 3"),
        Some(45.2),
        Some(-73.2),
        None,
        Some("S1"), // S1 is not a Station, but FK rule doesn't check type
    ));

    let errors = StopsParentStationFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 0);
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
    assert_eq!(errors[0].rule_id, "calendar_dates_service_not_in_calendar");
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

// ===========================================================================
// HW-018 — Extended FK rules
// ===========================================================================

// ---------------------------------------------------------------------------
// Additional helpers
// ---------------------------------------------------------------------------

fn make_transfer(
    from_stop: Option<&str>,
    to_stop: Option<&str>,
    from_trip: Option<&str>,
    to_trip: Option<&str>,
    from_route: Option<&str>,
    to_route: Option<&str>,
) -> Transfer {
    Transfer {
        from_stop_id: from_stop.map(|s| StopId::from(s.to_string())),
        to_stop_id: to_stop.map(|s| StopId::from(s.to_string())),
        from_route_id: from_route.map(|s| RouteId::from(s.to_string())),
        to_route_id: to_route.map(|s| RouteId::from(s.to_string())),
        from_trip_id: from_trip.map(|s| TripId::from(s.to_string())),
        to_trip_id: to_trip.map(|s| TripId::from(s.to_string())),
        transfer_type: TransferType::Recommended,
        min_transfer_time: None,
    }
}

fn make_pathway(id: &str, from_stop: &str, to_stop: &str) -> Pathway {
    Pathway {
        pathway_id: PathwayId::from(id.to_string()),
        from_stop_id: StopId::from(from_stop.to_string()),
        to_stop_id: StopId::from(to_stop.to_string()),
        pathway_mode: PathwayMode::Walkway,
        is_bidirectional: IsBidirectional::Unidirectional,
        length: None,
        traversal_time: None,
        stair_count: None,
        max_slope: None,
        min_width: None,
        signposted_as: None,
        reversed_signposted_as: None,
    }
}

fn make_fare_attribute(fare_id: &str, agency_id: Option<&str>) -> FareAttribute {
    FareAttribute {
        fare_id: FareId::from(fare_id.to_string()),
        price: 2.50,
        currency_type: CurrencyCode::from("CAD".to_string()),
        payment_method: 0,
        transfers: Some(0),
        agency_id: agency_id.map(|s| AgencyId::from(s.to_string())),
        transfer_duration: None,
    }
}

fn make_fare_rule(
    fare_id: &str,
    route_id: Option<&str>,
    origin: Option<&str>,
    dest: Option<&str>,
    contains: Option<&str>,
) -> FareRule {
    FareRule {
        fare_id: FareId::from(fare_id.to_string()),
        route_id: route_id.map(|s| RouteId::from(s.to_string())),
        origin_id: origin.map(ToString::to_string),
        destination_id: dest.map(ToString::to_string),
        contains_id: contains.map(ToString::to_string),
    }
}

fn make_translation(
    table_name: &str,
    record_id: Option<&str>,
    record_sub_id: Option<&str>,
) -> Translation {
    Translation {
        table_name: table_name.to_string(),
        field_name: "stop_name".to_string(),
        language: LanguageCode::from("fr".to_string()),
        translation: "Traduction".to_string(),
        record_id: record_id.map(ToString::to_string),
        record_sub_id: record_sub_id.map(ToString::to_string),
        field_value: None,
    }
}

fn make_attribution(
    agency_id: Option<&str>,
    route_id: Option<&str>,
    trip_id: Option<&str>,
) -> Attribution {
    Attribution {
        attribution_id: None,
        agency_id: agency_id.map(|s| AgencyId::from(s.to_string())),
        route_id: route_id.map(|s| RouteId::from(s.to_string())),
        trip_id: trip_id.map(|s| TripId::from(s.to_string())),
        organization_name: "Test Org".to_string(),
        is_producer: None,
        is_operator: None,
        is_authority: None,
        attribution_url: None,
        attribution_email: None,
        attribution_phone: None,
    }
}

fn make_stop_with_zone(id: &str, zone_id: Option<&str>) -> Stop {
    let mut stop = make_stop(id, Some("Stop"), Some(45.0), Some(-73.0), None, None);
    stop.zone_id = zone_id.map(ToString::to_string);
    stop
}

/// Extended valid feed with all optional files populated.
fn valid_feed_extended() -> GtfsFeed {
    let mut feed = valid_feed();

    // Stops with zone_id and special location types for pathways
    feed.stops[0].zone_id = Some("Z1".to_string());
    feed.stops.push(make_stop(
        "ENT1",
        Some("Entrance"),
        Some(45.0),
        Some(-73.0),
        Some(LocationType::EntranceExit),
        None,
    ));
    feed.stops.push(make_stop(
        "NODE1",
        Some("Node"),
        Some(45.0),
        Some(-73.0),
        Some(LocationType::GenericNode),
        None,
    ));

    // Transfers
    feed.transfers.push(make_transfer(
        Some("S1"),
        Some("S2"),
        Some("T1"),
        None,
        Some("R1"),
        None,
    ));

    // Pathways
    feed.pathways.push(make_pathway("PW1", "ENT1", "NODE1"));

    // Fare attributes + rules
    feed.fare_attributes
        .push(make_fare_attribute("F1", Some("A1")));
    feed.fare_rules
        .push(make_fare_rule("F1", Some("R1"), Some("Z1"), None, None));

    // Translations
    feed.translations
        .push(make_translation("stops", Some("S1"), None));

    // Attributions
    feed.attributions
        .push(make_attribution(Some("A1"), Some("R1"), Some("T1")));

    feed
}

// ---------------------------------------------------------------------------
// All extended rules — valid feed (Cas #1)
// ---------------------------------------------------------------------------

#[test]
fn valid_feed_extended_no_fk_errors() {
    let feed = valid_feed_extended();
    let rules: Vec<Box<dyn ValidationRule>> = vec![
        Box::new(TransfersFromStopFkRule),
        Box::new(TransfersToStopFkRule),
        Box::new(TransfersFromTripFkRule),
        Box::new(TransfersToTripFkRule),
        Box::new(TransfersFromRouteFkRule),
        Box::new(TransfersToRouteFkRule),
        Box::new(PathwaysStopsFkRule),
        Box::new(FareRulesFareFkRule),
        Box::new(FareRulesRouteFkRule),
        Box::new(FareRulesZonesFkRule),
        Box::new(FareAttributesAgencyFkRule),
        Box::new(TranslationsRecordFkRule),
        Box::new(AttributionsRefsFkRule),
    ];
    let all_errors: Vec<_> = rules.iter().flat_map(|r| r.validate(&feed)).collect();
    assert!(
        all_errors.is_empty(),
        "Expected 0 errors, got: {all_errors:?}"
    );
}

// ---------------------------------------------------------------------------
// transfers.from_stop_id / to_stop_id → stops (CA1, Cas #2)
// ---------------------------------------------------------------------------

#[test]
fn transfer_from_stop_orphan() {
    let mut feed = valid_feed();
    feed.transfers.push(make_transfer(
        Some("S99"),
        Some("S1"),
        None,
        None,
        None,
        None,
    ));

    let errors = TransfersFromStopFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].file_name.as_deref(), Some("transfers.txt"));
    assert_eq!(errors[0].field_name.as_deref(), Some("from_stop_id"));
    assert_eq!(errors[0].value.as_deref(), Some("S99"));
    assert_eq!(errors[0].rule_id, "foreign_key_violation");
    assert_eq!(errors[0].section, "5");
}

#[test]
fn transfer_to_stop_orphan() {
    let mut feed = valid_feed();
    feed.transfers.push(make_transfer(
        Some("S1"),
        Some("S99"),
        None,
        None,
        None,
        None,
    ));

    let errors = TransfersToStopFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("to_stop_id"));
    assert_eq!(errors[0].value.as_deref(), Some("S99"));
}

// ---------------------------------------------------------------------------
// transfers.from_trip_id / to_trip_id → trips (CA2, Cas #3, #4)
// ---------------------------------------------------------------------------

#[test]
fn transfer_from_trip_orphan() {
    let mut feed = valid_feed();
    feed.transfers
        .push(make_transfer(None, None, Some("T99"), None, None, None));

    let errors = TransfersFromTripFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("from_trip_id"));
    assert_eq!(errors[0].value.as_deref(), Some("T99"));
}

#[test]
fn transfer_from_trip_empty_no_error() {
    let mut feed = valid_feed();
    feed.transfers.push(make_transfer(
        Some("S1"),
        Some("S2"),
        None,
        None,
        None,
        None,
    ));

    let errors = TransfersFromTripFkRule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// transfers.from_route_id / to_route_id → routes (CA3)
// ---------------------------------------------------------------------------

#[test]
fn transfer_from_route_orphan() {
    let mut feed = valid_feed();
    feed.transfers
        .push(make_transfer(None, None, None, None, Some("R99"), None));

    let errors = TransfersFromRouteFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("from_route_id"));
    assert_eq!(errors[0].value.as_deref(), Some("R99"));
}

#[test]
fn transfer_to_route_orphan() {
    let mut feed = valid_feed();
    feed.transfers
        .push(make_transfer(None, None, None, None, None, Some("R99")));

    let errors = TransfersToRouteFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("to_route_id"));
    assert_eq!(errors[0].value.as_deref(), Some("R99"));
}

// ---------------------------------------------------------------------------
// pathways.from_stop_id / to_stop_id → stops with location_type (CA4, Cas #5, #6)
// ---------------------------------------------------------------------------

#[test]
fn pathway_stop_wrong_type() {
    let mut feed = valid_feed();
    // S1 has location_type=None (StopOrPlatform=0), not valid for pathways
    feed.pathways.push(make_pathway("PW1", "S1", "S2"));

    let errors = PathwaysStopsFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 2); // both endpoints wrong type
    assert!(errors[0].message.contains("location_type 2, 3, or 4"));
}

#[test]
fn pathway_stop_nonexistent() {
    let mut feed = valid_feed();
    feed.pathways.push(make_pathway("PW1", "S99", "S98"));

    let errors = PathwaysStopsFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 2);
    assert!(errors[0].message.contains("non-existent"));
}

#[test]
fn pathway_stop_valid_type2() {
    let mut feed = valid_feed();
    feed.stops.push(make_stop(
        "ENT1",
        Some("Entrance"),
        Some(45.0),
        Some(-73.0),
        Some(LocationType::EntranceExit),
        None,
    ));
    feed.stops.push(make_stop(
        "NODE1",
        Some("Node"),
        Some(45.0),
        Some(-73.0),
        Some(LocationType::GenericNode),
        None,
    ));
    feed.pathways.push(make_pathway("PW1", "ENT1", "NODE1"));

    let errors = PathwaysStopsFkRule.validate(&feed);
    assert!(errors.is_empty());
}

#[test]
fn pathway_stop_valid_type4() {
    let mut feed = valid_feed();
    feed.stops.push(make_stop(
        "BA1",
        Some("Boarding Area"),
        Some(45.0),
        Some(-73.0),
        Some(LocationType::BoardingArea),
        None,
    ));
    feed.stops.push(make_stop(
        "ENT1",
        Some("Entrance"),
        Some(45.0),
        Some(-73.0),
        Some(LocationType::EntranceExit),
        None,
    ));
    feed.pathways.push(make_pathway("PW1", "BA1", "ENT1"));

    let errors = PathwaysStopsFkRule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// fare_rules.fare_id → fare_attributes (CA5, Cas #7)
// ---------------------------------------------------------------------------

#[test]
fn fare_rule_fare_orphan() {
    let mut feed = valid_feed();
    feed.fare_rules
        .push(make_fare_rule("F99", None, None, None, None));

    let errors = FareRulesFareFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("fare_id"));
    assert_eq!(errors[0].value.as_deref(), Some("F99"));
    assert_eq!(errors[0].file_name.as_deref(), Some("fare_rules.txt"));
}

#[test]
fn fare_rule_fare_valid() {
    let mut feed = valid_feed();
    feed.fare_attributes.push(make_fare_attribute("F1", None));
    feed.fare_rules
        .push(make_fare_rule("F1", None, None, None, None));

    let errors = FareRulesFareFkRule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// fare_rules.route_id → routes (CA6)
// ---------------------------------------------------------------------------

#[test]
fn fare_rule_route_orphan() {
    let mut feed = valid_feed();
    feed.fare_attributes.push(make_fare_attribute("F1", None));
    feed.fare_rules
        .push(make_fare_rule("F1", Some("R99"), None, None, None));

    let errors = FareRulesRouteFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("route_id"));
    assert_eq!(errors[0].value.as_deref(), Some("R99"));
}

// ---------------------------------------------------------------------------
// fare_rules.origin_id / destination_id / contains_id → stops.zone_id (CA7, Cas #8, #9)
// ---------------------------------------------------------------------------

#[test]
fn fare_rule_zone_orphan() {
    let mut feed = valid_feed();
    feed.fare_attributes.push(make_fare_attribute("F1", None));
    feed.fare_rules
        .push(make_fare_rule("F1", None, Some("Z99"), None, None));

    let errors = FareRulesZonesFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("origin_id"));
    assert_eq!(errors[0].value.as_deref(), Some("Z99"));
}

#[test]
fn fare_rule_zone_valid() {
    let mut feed = valid_feed();
    feed.stops.push(make_stop_with_zone("S3", Some("Z1")));
    feed.fare_attributes.push(make_fare_attribute("F1", None));
    feed.fare_rules
        .push(make_fare_rule("F1", None, Some("Z1"), None, None));

    let errors = FareRulesZonesFkRule.validate(&feed);
    assert!(errors.is_empty());
}

#[test]
fn fare_rule_zone_multiple_fields_orphan() {
    let mut feed = valid_feed();
    feed.fare_attributes.push(make_fare_attribute("F1", None));
    feed.fare_rules.push(make_fare_rule(
        "F1",
        None,
        Some("Z99"),
        Some("Z98"),
        Some("Z97"),
    ));

    let errors = FareRulesZonesFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 3);
}

// ---------------------------------------------------------------------------
// fare_attributes.agency_id → agency
// ---------------------------------------------------------------------------

#[test]
fn fare_attributes_agency_orphan() {
    let mut feed = valid_feed();
    feed.fare_attributes
        .push(make_fare_attribute("F1", Some("AG99")));

    let errors = FareAttributesAgencyFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("agency_id"));
    assert_eq!(errors[0].value.as_deref(), Some("AG99"));
    assert_eq!(errors[0].file_name.as_deref(), Some("fare_attributes.txt"));
}

#[test]
fn fare_attributes_agency_valid() {
    let mut feed = valid_feed();
    feed.fare_attributes
        .push(make_fare_attribute("F1", Some("A1")));

    let errors = FareAttributesAgencyFkRule.validate(&feed);
    assert!(errors.is_empty());
}

#[test]
fn fare_attributes_agency_empty_no_error() {
    let mut feed = valid_feed();
    feed.fare_attributes.push(make_fare_attribute("F1", None));

    let errors = FareAttributesAgencyFkRule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// translations.record_id → target table (CA8, Cas #10, #11)
// ---------------------------------------------------------------------------

#[test]
fn translation_record_orphan() {
    let mut feed = valid_feed();
    feed.translations
        .push(make_translation("stops", Some("S99"), None));

    let errors = TranslationsRecordFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("record_id"));
    assert_eq!(errors[0].value.as_deref(), Some("S99"));
    assert_eq!(errors[0].file_name.as_deref(), Some("translations.txt"));
}

#[test]
fn translation_record_valid() {
    let mut feed = valid_feed();
    feed.translations
        .push(make_translation("stops", Some("S1"), None));

    let errors = TranslationsRecordFkRule.validate(&feed);
    assert!(errors.is_empty());
}

#[test]
fn translation_record_id_empty_no_error() {
    let mut feed = valid_feed();
    feed.translations
        .push(make_translation("stops", None, None));

    let errors = TranslationsRecordFkRule.validate(&feed);
    assert!(errors.is_empty());
}

#[test]
fn translation_record_routes() {
    let mut feed = valid_feed();
    feed.translations
        .push(make_translation("routes", Some("R99"), None));

    let errors = TranslationsRecordFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].value.as_deref(), Some("R99"));
}

#[test]
fn translation_record_sub_id_stop_times_valid() {
    let mut feed = valid_feed();
    // feed has stop_time (T1, seq=1)
    feed.translations
        .push(make_translation("stop_times", Some("T1"), Some("1")));

    let errors = TranslationsRecordFkRule.validate(&feed);
    assert!(errors.is_empty());
}

#[test]
fn translation_record_sub_id_stop_times_orphan() {
    let mut feed = valid_feed();
    // feed has stop_time (T1, seq=1, seq=2) but not seq=99
    feed.translations
        .push(make_translation("stop_times", Some("T1"), Some("99")));

    let errors = TranslationsRecordFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("record_sub_id"));
    assert_eq!(errors[0].value.as_deref(), Some("99"));
}

// ---------------------------------------------------------------------------
// attributions.agency_id / route_id / trip_id (CA9, Cas #12)
// ---------------------------------------------------------------------------

#[test]
fn attribution_agency_orphan() {
    let mut feed = valid_feed();
    feed.attributions
        .push(make_attribution(Some("AG99"), None, None));

    let errors = AttributionsRefsFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("agency_id"));
    assert_eq!(errors[0].value.as_deref(), Some("AG99"));
    assert_eq!(errors[0].file_name.as_deref(), Some("attributions.txt"));
}

#[test]
fn attribution_route_orphan() {
    let mut feed = valid_feed();
    feed.attributions
        .push(make_attribution(None, Some("R99"), None));

    let errors = AttributionsRefsFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("route_id"));
}

#[test]
fn attribution_trip_orphan() {
    let mut feed = valid_feed();
    feed.attributions
        .push(make_attribution(None, None, Some("T99")));

    let errors = AttributionsRefsFkRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("trip_id"));
}

#[test]
fn attribution_all_valid() {
    let mut feed = valid_feed();
    feed.attributions
        .push(make_attribution(Some("A1"), Some("R1"), Some("T1")));

    let errors = AttributionsRefsFkRule.validate(&feed);
    assert!(errors.is_empty());
}

#[test]
fn attribution_all_empty_no_error() {
    let mut feed = valid_feed();
    feed.attributions.push(make_attribution(None, None, None));

    let errors = AttributionsRefsFkRule.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// Absent files generate no errors (CA11, Cas #13)
// ---------------------------------------------------------------------------

#[test]
fn absent_files_no_errors() {
    let feed = valid_feed(); // no transfers, pathways, fare_rules, etc.
    let rules: Vec<Box<dyn ValidationRule>> = vec![
        Box::new(TransfersFromStopFkRule),
        Box::new(TransfersToStopFkRule),
        Box::new(TransfersFromTripFkRule),
        Box::new(TransfersToTripFkRule),
        Box::new(TransfersFromRouteFkRule),
        Box::new(TransfersToRouteFkRule),
        Box::new(PathwaysStopsFkRule),
        Box::new(FareRulesFareFkRule),
        Box::new(FareRulesRouteFkRule),
        Box::new(FareRulesZonesFkRule),
        Box::new(FareAttributesAgencyFkRule),
        Box::new(TranslationsRecordFkRule),
        Box::new(AttributionsRefsFkRule),
    ];
    let all_errors: Vec<_> = rules.iter().flat_map(|r| r.validate(&feed)).collect();
    assert!(
        all_errors.is_empty(),
        "Empty optional files should produce no errors, got: {all_errors:?}"
    );
}

// ---------------------------------------------------------------------------
// Cumul multi-fichiers (Cas #14)
// ---------------------------------------------------------------------------

#[test]
fn multi_file_cumul_errors() {
    let mut feed = valid_feed();
    // Transfer orphan
    feed.transfers
        .push(make_transfer(Some("S99"), None, None, None, None, None));
    // Fare rule orphan
    feed.fare_rules
        .push(make_fare_rule("F99", None, None, None, None));
    // Pathway orphan (nonexistent stops)
    feed.pathways.push(make_pathway("PW1", "S99", "S98"));

    let transfer_errors = TransfersFromStopFkRule.validate(&feed);
    let fare_errors = FareRulesFareFkRule.validate(&feed);
    let pathway_errors = PathwaysStopsFkRule.validate(&feed);

    assert_eq!(transfer_errors.len(), 1);
    assert_eq!(fare_errors.len(), 1);
    assert_eq!(pathway_errors.len(), 2); // 2 endpoints

    // All are section 5
    for e in transfer_errors
        .iter()
        .chain(&fare_errors)
        .chain(&pathway_errors)
    {
        assert_eq!(e.section, "5");
        assert_eq!(e.rule_id, "foreign_key_violation");
    }
}

// ---------------------------------------------------------------------------
// Error metadata (CA12)
// ---------------------------------------------------------------------------

#[test]
fn extended_error_includes_correct_metadata() {
    let mut feed = valid_feed();
    // Add a valid transfer first, then an orphan at index 1 → line 3
    feed.transfers.push(make_transfer(
        Some("S1"),
        Some("S2"),
        None,
        None,
        None,
        None,
    ));
    feed.transfers
        .push(make_transfer(Some("S99"), None, None, None, None, None));

    let errors = TransfersFromStopFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "foreign_key_violation");
    assert_eq!(errors[0].section, "5");
    assert_eq!(errors[0].file_name.as_deref(), Some("transfers.txt"));
    assert_eq!(errors[0].line_number, Some(3));
    assert_eq!(errors[0].field_name.as_deref(), Some("from_stop_id"));
    assert_eq!(errors[0].value.as_deref(), Some("S99"));
}
