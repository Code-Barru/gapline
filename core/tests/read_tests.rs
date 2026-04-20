use gapline_core::crud::query::parse;
use gapline_core::crud::read::{GtfsTarget, ReadError, read_records};
use gapline_core::models::*;

use chrono::NaiveDate;

// ===========================================================================
// Test helpers
// ===========================================================================

fn make_stop(id: &str, name: &str, lat: f64, lon: f64) -> Stop {
    Stop {
        stop_id: StopId::from(id),
        stop_code: None,
        stop_name: Some(name.into()),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(lat)),
        stop_lon: Some(Longitude(lon)),
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

fn make_trip(route_id: &str, service_id: &str, trip_id: &str) -> Trip {
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

fn make_stop_time(trip_id: &str, stop_id: &str, seq: u32) -> StopTime {
    StopTime {
        trip_id: TripId::from(trip_id),
        arrival_time: None,
        departure_time: None,
        stop_id: StopId::from(stop_id),
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
        service_id: ServiceId::from(service_id),
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        start_date: gtfs_date(2025, 1, 1),
        end_date: gtfs_date(2025, 12, 31),
    }
}

fn make_calendar_date(service_id: &str, date: GtfsDate, exception: ExceptionType) -> CalendarDate {
    CalendarDate {
        service_id: ServiceId::from(service_id),
        date,
        exception_type: exception,
    }
}

fn gtfs_date(y: i32, m: u32, d: u32) -> GtfsDate {
    GtfsDate(NaiveDate::from_ymd_opt(y, m, d).unwrap())
}

fn empty_feed() -> GtfsFeed {
    GtfsFeed::default()
}

// ===========================================================================
// Test 1: Read all stops (no filter)
// ===========================================================================

#[test]
fn read_all_stops_no_filter() {
    let mut feed = empty_feed();
    feed.stops = vec![
        make_stop("S1", "Gare A", 48.85, 2.35),
        make_stop("S2", "Gare B", 48.86, 2.36),
        make_stop("S3", "Gare C", 48.87, 2.37),
    ];

    let result = read_records(&feed, GtfsTarget::Stops, None).unwrap();
    assert_eq!(result.rows.len(), 3);
    assert_eq!(result.file_name, "stops.txt");
    assert!(result.headers.contains(&"stop_id"));
    assert!(result.headers.contains(&"stop_name"));
}

// ===========================================================================
// Test 2: Filter by equality
// ===========================================================================

#[test]
fn read_trips_filter_equality() {
    let mut feed = empty_feed();
    feed.trips = vec![
        make_trip("R1", "SVC1", "T1"),
        make_trip("R1", "SVC1", "T2"),
        make_trip("R2", "SVC1", "T3"),
    ];

    let query = parse("route_id=R1").unwrap();
    let result = read_records(&feed, GtfsTarget::Trips, Some(&query)).unwrap();
    assert_eq!(result.rows.len(), 2);
}

// ===========================================================================
// Test 3: Filter by comparison
// ===========================================================================

#[test]
fn read_stop_times_filter_comparison() {
    let mut feed = empty_feed();
    feed.stop_times = vec![
        make_stop_time("T1", "S1", 5),
        make_stop_time("T1", "S2", 10),
        make_stop_time("T1", "S3", 15),
        make_stop_time("T1", "S4", 20),
    ];

    let query = parse("stop_sequence>10").unwrap();
    let result = read_records(&feed, GtfsTarget::StopTimes, Some(&query)).unwrap();
    assert_eq!(result.rows.len(), 2);
}

// ===========================================================================
// Test 4: Filter AND
// ===========================================================================

#[test]
fn read_stop_times_filter_and() {
    let mut feed = empty_feed();
    feed.stop_times = vec![
        make_stop_time("T1", "S1", 1),
        make_stop_time("T1", "S2", 3),
        make_stop_time("T1", "S3", 7),
        make_stop_time("T2", "S1", 2),
    ];

    let query = parse("trip_id=T1 AND stop_sequence<5").unwrap();
    let result = read_records(&feed, GtfsTarget::StopTimes, Some(&query)).unwrap();
    assert_eq!(result.rows.len(), 2);
}

// ===========================================================================
// Test 5: No results
// ===========================================================================

#[test]
fn read_stops_no_match() {
    let mut feed = empty_feed();
    feed.stops = vec![make_stop("S1", "Gare A", 48.85, 2.35)];

    let query = parse("stop_id=INEXISTANT").unwrap();
    let result = read_records(&feed, GtfsTarget::Stops, Some(&query)).unwrap();
    assert_eq!(result.rows.len(), 0);
}

// ===========================================================================
// Test 6: JSON-compatible output (headers + rows form valid objects)
// ===========================================================================

#[test]
fn read_result_rows_match_headers_length() {
    let mut feed = empty_feed();
    feed.stops = vec![make_stop("S1", "Gare A", 48.85, 2.35)];

    let result = read_records(&feed, GtfsTarget::Stops, None).unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].len(), result.headers.len());
}

// ===========================================================================
// Test 7: Read all calendar entries
// ===========================================================================

#[test]
fn read_all_calendar() {
    let mut feed = empty_feed();
    feed.calendars = vec![make_calendar("SVC1"), make_calendar("SVC2")];

    let result = read_records(&feed, GtfsTarget::Calendar, None).unwrap();
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.file_name, "calendar.txt");
}

// ===========================================================================
// Test 8: Calendar dates filtered by service_id
// ===========================================================================

#[test]
fn read_calendar_dates_filter_service() {
    let mut feed = empty_feed();
    let date = gtfs_date(2025, 6, 15);
    feed.calendar_dates = vec![
        make_calendar_date("SVC1", date, ExceptionType::Added),
        make_calendar_date("SVC2", date, ExceptionType::Removed),
        make_calendar_date("SVC1", gtfs_date(2025, 7, 1), ExceptionType::Added),
    ];

    let query = parse("service_id=SVC1").unwrap();
    let result = read_records(&feed, GtfsTarget::CalendarDates, Some(&query)).unwrap();
    assert_eq!(result.rows.len(), 2);
}

// ===========================================================================
// Test 9: Empty feed returns 0 rows
// ===========================================================================

#[test]
fn read_empty_feed_returns_zero_rows() {
    let feed = empty_feed();
    let result = read_records(&feed, GtfsTarget::Stops, None).unwrap();
    assert_eq!(result.rows.len(), 0);
    assert!(!result.headers.is_empty());
}

// ===========================================================================
// Test 10: Unknown field in query produces error
// ===========================================================================

#[test]
fn read_unknown_field_error() {
    let feed = empty_feed();
    let query = parse("nonexistent_field=X").unwrap();
    let err = read_records(&feed, GtfsTarget::Stops, Some(&query)).unwrap_err();
    assert!(matches!(err, ReadError::QueryError(_)));
}

// ===========================================================================
// Test 11: All 17 targets return correct headers
// ===========================================================================

#[test]
fn all_targets_return_headers() {
    let feed = empty_feed();
    let targets = [
        (GtfsTarget::Agency, "agency.txt"),
        (GtfsTarget::Stops, "stops.txt"),
        (GtfsTarget::Routes, "routes.txt"),
        (GtfsTarget::Trips, "trips.txt"),
        (GtfsTarget::StopTimes, "stop_times.txt"),
        (GtfsTarget::Calendar, "calendar.txt"),
        (GtfsTarget::CalendarDates, "calendar_dates.txt"),
        (GtfsTarget::Shapes, "shapes.txt"),
        (GtfsTarget::Frequencies, "frequencies.txt"),
        (GtfsTarget::Transfers, "transfers.txt"),
        (GtfsTarget::Pathways, "pathways.txt"),
        (GtfsTarget::Levels, "levels.txt"),
        (GtfsTarget::FeedInfo, "feed_info.txt"),
        (GtfsTarget::FareAttributes, "fare_attributes.txt"),
        (GtfsTarget::FareRules, "fare_rules.txt"),
        (GtfsTarget::Translations, "translations.txt"),
        (GtfsTarget::Attributions, "attributions.txt"),
    ];

    for (target, expected_file) in targets {
        let result = read_records(&feed, target, None).unwrap();
        assert_eq!(
            result.file_name, expected_file,
            "file_name mismatch for {expected_file}"
        );
        assert!(
            !result.headers.is_empty(),
            "headers empty for {expected_file}"
        );
        assert_eq!(result.rows.len(), 0, "expected 0 rows for {expected_file}");
    }
}

// ===========================================================================
// Test 12: feed_info (Option -> as_slice)
// ===========================================================================

#[test]
fn read_feed_info_present() {
    let mut feed = empty_feed();
    feed.feed_info = Some(FeedInfo {
        feed_publisher_name: "Test Publisher".into(),
        feed_publisher_url: Url::from("https://example.com"),
        feed_lang: LanguageCode::from("fr"),
        default_lang: None,
        feed_start_date: None,
        feed_end_date: None,
        feed_version: Some("1.0".into()),
        feed_contact_email: None,
        feed_contact_url: None,
    });

    let result = read_records(&feed, GtfsTarget::FeedInfo, None).unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.file_name, "feed_info.txt");
}

#[test]
fn read_feed_info_absent() {
    let feed = empty_feed();
    let result = read_records(&feed, GtfsTarget::FeedInfo, None).unwrap();
    assert_eq!(result.rows.len(), 0);
}
