//! Tests for section 6 - Primary Key Uniqueness.

use chrono::NaiveDate;
use gapline_core::models::*;
use gapline_core::validation::primary_key::PrimaryKeyUniquenessRule;
use gapline_core::validation::{Severity, ValidationError, ValidationRule};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_agency(id: Option<&str>) -> Agency {
    Agency {
        agency_id: id.map(AgencyId::from),
        agency_name: "Agency".to_string(),
        agency_url: Url::from("https://example.com"),
        agency_timezone: Timezone::from("America/Montreal"),
        agency_lang: None,
        agency_phone: None,
        agency_fare_url: None,
        agency_email: None,
    }
}

fn make_stop(id: &str) -> Stop {
    Stop {
        stop_id: StopId::from(id),
        stop_code: None,
        stop_name: Some("Stop".to_string()),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(45.0)),
        stop_lon: Some(Longitude(-73.0)),
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
        agency_id: Some(AgencyId::from("A1")),
        route_short_name: Some("1".to_string()),
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

fn make_trip(trip_id: &str) -> Trip {
    Trip {
        route_id: RouteId::from("R1"),
        service_id: ServiceId::from("SVC1"),
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

fn make_stop_time(trip_id: &str, seq: u32) -> StopTime {
    StopTime {
        trip_id: TripId::from(trip_id),
        arrival_time: Some(GtfsTime::from_hms(8, 0, 0)),
        departure_time: Some(GtfsTime::from_hms(8, 0, 0)),
        stop_id: StopId::from(format!("S{seq}")),
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

fn make_calendar(service_id: &str) -> Calendar {
    Calendar {
        service_id: ServiceId::from(service_id),
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        start_date: gtfs_date(2026, 1, 1),
        end_date: gtfs_date(2026, 12, 31),
    }
}

fn make_calendar_date(service_id: &str, date: GtfsDate) -> CalendarDate {
    CalendarDate {
        service_id: ServiceId::from(service_id),
        date,
        exception_type: ExceptionType::Added,
    }
}

fn make_shape(shape_id: &str, seq: u32) -> Shape {
    Shape {
        shape_id: ShapeId::from(shape_id),
        shape_pt_lat: Latitude(45.0),
        shape_pt_lon: Longitude(-73.0),
        shape_pt_sequence: seq,
        shape_dist_traveled: None,
    }
}

fn make_frequency(trip_id: &str, start: GtfsTime) -> Frequency {
    Frequency {
        trip_id: TripId::from(trip_id),
        start_time: start,
        end_time: GtfsTime::from_hms(23, 0, 0),
        headway_secs: 600,
        exact_times: None,
    }
}

fn make_transfer(
    from_stop: Option<&str>,
    to_stop: Option<&str>,
    from_trip: Option<&str>,
    to_trip: Option<&str>,
) -> Transfer {
    Transfer {
        from_stop_id: from_stop.map(StopId::from),
        to_stop_id: to_stop.map(StopId::from),
        from_route_id: None,
        to_route_id: None,
        from_trip_id: from_trip.map(TripId::from),
        to_trip_id: to_trip.map(TripId::from),
        transfer_type: TransferType::Recommended,
        min_transfer_time: None,
    }
}

fn make_pathway(id: &str) -> Pathway {
    Pathway {
        pathway_id: PathwayId::from(id),
        from_stop_id: StopId::from("S1"),
        to_stop_id: StopId::from("S2"),
        pathway_mode: PathwayMode::Walkway,
        is_bidirectional: IsBidirectional::Bidirectional,
        length: None,
        traversal_time: None,
        stair_count: None,
        max_slope: None,
        min_width: None,
        signposted_as: None,
        reversed_signposted_as: None,
    }
}

fn make_level(id: &str) -> Level {
    Level {
        level_id: LevelId::from(id),
        level_index: 0.0,
        level_name: None,
    }
}

fn make_fare_attribute(id: &str) -> FareAttribute {
    FareAttribute {
        fare_id: FareId::from(id),
        price: 2.50,
        currency_type: CurrencyCode::from("USD"),
        payment_method: 0,
        transfers: None,
        agency_id: None,
        transfer_duration: None,
    }
}

fn make_attribution(id: Option<&str>) -> Attribution {
    Attribution {
        attribution_id: id.map(std::string::ToString::to_string),
        agency_id: None,
        route_id: None,
        trip_id: None,
        organization_name: "Org".to_string(),
        is_producer: Some(1),
        is_operator: None,
        is_authority: None,
        attribution_url: None,
        attribution_email: None,
        attribution_phone: None,
    }
}

fn gtfs_date(y: i32, m: u32, d: u32) -> GtfsDate {
    GtfsDate(NaiveDate::from_ymd_opt(y, m, d).unwrap())
}

fn valid_feed() -> GtfsFeed {
    let mut feed = GtfsFeed::default();
    feed.loaded_files.insert("agency.txt".to_string());
    feed.loaded_files.insert("stops.txt".to_string());
    feed.loaded_files.insert("routes.txt".to_string());
    feed.loaded_files.insert("trips.txt".to_string());
    feed.loaded_files.insert("stop_times.txt".to_string());
    feed.loaded_files.insert("calendar.txt".to_string());

    feed.agencies.push(make_agency(Some("A1")));
    feed.stops.push(make_stop("S1"));
    feed.stops.push(make_stop("S2"));
    feed.routes.push(make_route("R1"));
    feed.trips.push(make_trip("T1"));
    feed.stop_times.push(make_stop_time("T1", 1));
    feed.stop_times.push(make_stop_time("T1", 2));
    feed.calendars.push(make_calendar("SVC1"));
    feed
}

fn errors_for_file<'a>(errors: &'a [ValidationError], file: &str) -> Vec<&'a ValidationError> {
    errors
        .iter()
        .filter(|e| e.file_name.as_deref() == Some(file))
        .collect()
}

fn errors_for_field<'a>(errors: &'a [ValidationError], field: &str) -> Vec<&'a ValidationError> {
    errors
        .iter()
        .filter(|e| e.field_name.as_deref() == Some(field))
        .collect()
}

fn assert_duplicate_error(error: &ValidationError) {
    assert_eq!(error.rule_id, "duplicate_key");
    assert_eq!(error.section, "6");
    assert_eq!(error.severity, Severity::Error);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn no_duplicates() {
    let feed = valid_feed();
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    assert!(errors.is_empty(), "Expected 0 errors, got: {errors:?}");
}

#[test]
fn duplicate_stop_id() {
    let mut feed = valid_feed();
    feed.stops.push(make_stop("S1"));
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_field(&errors, "stop_id");
    assert_eq!(errs.len(), 1);
    assert_duplicate_error(errs[0]);
    assert_eq!(errs[0].file_name.as_deref(), Some("stops.txt"));
    assert_eq!(errs[0].value.as_deref(), Some("S1"));
    assert_eq!(errs[0].line_number, Some(4)); // S1, S2, S1 → duplicate at line 4
}

#[test]
fn duplicate_route_id() {
    let mut feed = valid_feed();
    feed.routes.push(make_route("R1"));
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_field(&errors, "route_id");
    assert_eq!(errs.len(), 1);
    assert_duplicate_error(errs[0]);
    assert_eq!(errs[0].value.as_deref(), Some("R1"));
}

#[test]
fn duplicate_trip_id() {
    let mut feed = valid_feed();
    feed.trips.push(make_trip("T1"));
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_field(&errors, "trip_id");
    assert_eq!(errs.len(), 1);
    assert_duplicate_error(errs[0]);
    assert_eq!(errs[0].value.as_deref(), Some("T1"));
}

#[test]
fn duplicate_stop_times_composite() {
    let mut feed = valid_feed();
    feed.stop_times.push(make_stop_time("T1", 1)); // duplicate (T1, 1)
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_field(&errors, "trip_id, stop_sequence");
    assert_eq!(errs.len(), 1);
    assert_duplicate_error(errs[0]);
    assert_eq!(errs[0].file_name.as_deref(), Some("stop_times.txt"));
    assert_eq!(errs[0].value.as_deref(), Some("(T1, 1)"));
}

#[test]
fn valid_stop_times_different_sequence() {
    let feed = valid_feed(); // (T1,1) and (T1,2) - no duplicate
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_field(&errors, "trip_id, stop_sequence");
    assert!(errs.is_empty());
}

#[test]
fn duplicate_calendar_dates() {
    let mut feed = valid_feed();
    feed.loaded_files.insert("calendar_dates.txt".to_string());
    let date = gtfs_date(2026, 3, 1);
    feed.calendar_dates.push(make_calendar_date("SVC1", date));
    feed.calendar_dates.push(make_calendar_date("SVC1", date));
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_field(&errors, "service_id, date");
    assert_eq!(errs.len(), 1);
    assert_duplicate_error(errs[0]);
}

#[test]
fn duplicate_shapes() {
    let mut feed = valid_feed();
    feed.loaded_files.insert("shapes.txt".to_string());
    feed.shapes.push(make_shape("SH1", 5));
    feed.shapes.push(make_shape("SH1", 5));
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_field(&errors, "shape_id, shape_pt_sequence");
    assert_eq!(errs.len(), 1);
    assert_duplicate_error(errs[0]);
}

#[test]
fn duplicate_frequencies() {
    let mut feed = valid_feed();
    feed.loaded_files.insert("frequencies.txt".to_string());
    let start = GtfsTime::from_hms(6, 0, 0);
    feed.frequencies.push(make_frequency("T1", start));
    feed.frequencies.push(make_frequency("T1", start));
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_field(&errors, "trip_id, start_time");
    assert_eq!(errs.len(), 1);
    assert_duplicate_error(errs[0]);
}

#[test]
fn duplicate_transfers() {
    let mut feed = valid_feed();
    feed.loaded_files.insert("transfers.txt".to_string());
    feed.transfers.push(make_transfer(
        Some("S1"),
        Some("S2"),
        Some("T1"),
        Some("T2"),
    ));
    feed.transfers.push(make_transfer(
        Some("S1"),
        Some("S2"),
        Some("T1"),
        Some("T2"),
    ));
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_field(
        &errors,
        "from_stop_id, to_stop_id, from_trip_id, to_trip_id",
    );
    assert_eq!(errs.len(), 1);
    assert_duplicate_error(errs[0]);
}

#[test]
fn feed_info_two_lines() {
    let mut feed = valid_feed();
    feed.loaded_files.insert("feed_info.txt".to_string());
    feed.feed_info = Some(FeedInfo {
        feed_publisher_name: "Publisher".to_string(),
        feed_publisher_url: Url::from("https://example.com"),
        feed_lang: LanguageCode::from("en"),
        default_lang: None,
        feed_start_date: None,
        feed_end_date: None,
        feed_version: None,
        feed_contact_email: None,
        feed_contact_url: None,
    });
    feed.feed_info_line_count = 2;
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_file(&errors, "feed_info.txt");
    assert_eq!(errs.len(), 1);
    assert_duplicate_error(errs[0]);
}

#[test]
fn feed_info_one_line() {
    let mut feed = valid_feed();
    feed.loaded_files.insert("feed_info.txt".to_string());
    feed.feed_info_line_count = 1;
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_file(&errors, "feed_info.txt");
    assert!(errs.is_empty());
}

#[test]
fn duplicate_attribution_id() {
    let mut feed = valid_feed();
    feed.loaded_files.insert("attributions.txt".to_string());
    feed.attributions.push(make_attribution(Some("A1")));
    feed.attributions.push(make_attribution(Some("A1")));
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_field(&errors, "attribution_id");
    assert_eq!(errs.len(), 1);
    assert_duplicate_error(errs[0]);
}

#[test]
fn attributions_no_id_column() {
    let mut feed = valid_feed();
    feed.loaded_files.insert("attributions.txt".to_string());
    feed.attributions.push(make_attribution(None));
    feed.attributions.push(make_attribution(None));
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_field(&errors, "attribution_id");
    assert!(errs.is_empty());
}

#[test]
fn triple_duplicate() {
    let mut feed = valid_feed();
    // feed already has S1 and S2; add two more S1
    feed.stops.push(make_stop("S1"));
    feed.stops.push(make_stop("S1"));
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_field(&errors, "stop_id");
    assert_eq!(errs.len(), 2); // 2nd and 3rd S1 flagged
    assert_eq!(errs[0].line_number, Some(4));
    assert_eq!(errs[1].line_number, Some(5));
}

#[test]
fn multi_file_duplicates() {
    let mut feed = valid_feed();
    feed.stops.push(make_stop("S1")); // duplicate stop_id
    feed.routes.push(make_route("R1")); // duplicate route_id
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let stop_errs = errors_for_field(&errors, "stop_id");
    let route_errs = errors_for_field(&errors, "route_id");
    assert_eq!(stop_errs.len(), 1);
    assert_eq!(route_errs.len(), 1);
}

#[test]
fn duplicate_pathway_id() {
    let mut feed = valid_feed();
    feed.loaded_files.insert("pathways.txt".to_string());
    feed.pathways.push(make_pathway("PW1"));
    feed.pathways.push(make_pathway("PW1"));
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_field(&errors, "pathway_id");
    assert_eq!(errs.len(), 1);
    assert_duplicate_error(errs[0]);
}

#[test]
fn duplicate_level_id() {
    let mut feed = valid_feed();
    feed.loaded_files.insert("levels.txt".to_string());
    feed.levels.push(make_level("L1"));
    feed.levels.push(make_level("L1"));
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_field(&errors, "level_id");
    assert_eq!(errs.len(), 1);
    assert_duplicate_error(errs[0]);
}

#[test]
fn duplicate_fare_id() {
    let mut feed = valid_feed();
    feed.loaded_files.insert("fare_attributes.txt".to_string());
    feed.fare_attributes.push(make_fare_attribute("F1"));
    feed.fare_attributes.push(make_fare_attribute("F1"));
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_field(&errors, "fare_id");
    assert_eq!(errs.len(), 1);
    assert_duplicate_error(errs[0]);
}

#[test]
fn absent_file_no_errors() {
    let mut feed = valid_feed();
    // shapes.txt NOT in loaded_files, but add shape data anyway
    feed.shapes.push(make_shape("SH1", 1));
    feed.shapes.push(make_shape("SH1", 1));
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let errs = errors_for_file(&errors, "shapes.txt");
    assert!(errs.is_empty());
}

#[test]
#[ignore = "run with: cargo test --release -p gapline-core -- --ignored performance"]
fn performance_stop_times() {
    let mut feed = GtfsFeed::default();
    feed.loaded_files.insert("stop_times.txt".to_string());
    for i in 0..1_000_000u32 {
        feed.stop_times.push(StopTime {
            trip_id: TripId::from(format!("T{}", i / 100)),
            arrival_time: None,
            departure_time: None,
            stop_id: StopId::from("S1"),
            stop_sequence: i % 100,
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
        });
    }

    let start = std::time::Instant::now();
    let errors = PrimaryKeyUniquenessRule.validate(&feed);
    let elapsed = start.elapsed();

    assert!(errors.is_empty());
    assert!(
        elapsed.as_secs() < 2,
        "Took {elapsed:?}, expected < 2 seconds"
    );
}
