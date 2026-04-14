use headway_core::crud::delete::{apply_delete, validate_delete};
use headway_core::crud::query::parse;
use headway_core::crud::read::GtfsTarget;
use headway_core::models::*;

use chrono::NaiveDate;

fn s(val: &str) -> String {
    val.to_string()
}

fn empty_feed() -> GtfsFeed {
    GtfsFeed::default()
}

fn gtfs_date(y: i32, m: u32, d: u32) -> GtfsDate {
    GtfsDate(NaiveDate::from_ymd_opt(y, m, d).unwrap())
}

fn make_agency(id: &str) -> Agency {
    Agency {
        agency_id: Some(AgencyId::from(id)),
        agency_name: "Test Agency".into(),
        agency_url: Url::from("http://example.com"),
        agency_timezone: Timezone::from("America/Montreal"),
        agency_lang: None,
        agency_phone: None,
        agency_fare_url: None,
        agency_email: None,
    }
}

fn make_stop(id: &str, name: &str, lat: f64, lon: f64) -> Stop {
    Stop {
        stop_id: StopId::from(id),
        stop_code: None,
        stop_name: Some(name.to_string()),
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

fn make_stop_with_parent(id: &str, name: &str, parent: &str) -> Stop {
    Stop {
        parent_station: Some(StopId::from(parent)),
        ..make_stop(id, name, 45.5, -73.6)
    }
}

fn make_route(id: &str, agency_id: &str) -> Route {
    Route {
        route_id: RouteId::from(id),
        agency_id: Some(AgencyId::from(agency_id)),
        route_short_name: Some("1".into()),
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

fn make_calendar_date(service_id: &str, date: GtfsDate) -> CalendarDate {
    CalendarDate {
        service_id: ServiceId::from(service_id),
        date,
        exception_type: ExceptionType::Added,
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

// agency A1 -> routes R1,R2 -> trips T1(R1),T2(R1),T3(R2)
// calendar SVC1 -> trips T1,T2,T3
// calendar_dates SVC1/2025-06-01, SVC1/2025-07-01
// stops S01,S02,S03(parent=S01),S_UNUSED
// stop_times: T1/S01/1, T1/S02/2, T2/S01/1, T2/S02/2, T3/S01/1
// frequencies: T1/06:00, T1/12:00
fn feed_with_cascade() -> GtfsFeed {
    let mut feed = empty_feed();

    feed.agencies.push(make_agency("A1"));

    feed.stops.push(make_stop("S01", "Station", 45.5, -73.6));
    feed.stops.push(make_stop("S02", "Stop Two", 45.6, -73.5));
    feed.stops
        .push(make_stop_with_parent("S03", "Platform", "S01"));
    feed.stops
        .push(make_stop("S_UNUSED", "Unused", 45.7, -73.4));

    feed.routes.push(make_route("R1", "A1"));
    feed.routes.push(make_route("R2", "A1"));

    feed.calendars.push(make_calendar("SVC1"));
    feed.calendar_dates
        .push(make_calendar_date("SVC1", gtfs_date(2025, 6, 1)));
    feed.calendar_dates
        .push(make_calendar_date("SVC1", gtfs_date(2025, 7, 1)));

    feed.trips.push(make_trip("R1", "SVC1", "T1"));
    feed.trips.push(make_trip("R1", "SVC1", "T2"));
    feed.trips.push(make_trip("R2", "SVC1", "T3"));

    // T1: 2 stop_times
    feed.stop_times.push(make_stop_time("T1", "S01", 1));
    feed.stop_times.push(make_stop_time("T1", "S02", 2));
    // T2: 2 stop_times
    feed.stop_times.push(make_stop_time("T2", "S01", 1));
    feed.stop_times.push(make_stop_time("T2", "S02", 2));
    // T3: 1 stop_time
    feed.stop_times.push(make_stop_time("T3", "S01", 1));

    // T1: 2 frequencies
    feed.frequencies
        .push(make_frequency("T1", GtfsTime::from_hms(6, 0, 0)));
    feed.frequencies
        .push(make_frequency("T1", GtfsTime::from_hms(12, 0, 0)));

    feed
}

#[test]
fn delete_calendar_date_without_dependents() {
    let mut feed = feed_with_cascade();
    let query = parse("service_id=SVC1 AND date=20250601").unwrap();
    let plan = validate_delete(&feed, GtfsTarget::CalendarDates, &query).unwrap();
    assert_eq!(plan.matched_count, 1);
    assert!(plan.cascade.is_none());
    let result = apply_delete(&mut feed, &plan);
    assert_eq!(result.primary_count, 1);
    assert_eq!(feed.calendar_dates.len(), 1);
    assert_eq!(feed.calendar_dates[0].date, gtfs_date(2025, 7, 1));
}

#[test]
fn delete_stop_no_dependents() {
    let mut feed = feed_with_cascade();
    let query = parse("stop_id=S_UNUSED").unwrap();
    let plan = validate_delete(&feed, GtfsTarget::Stops, &query).unwrap();
    assert_eq!(plan.matched_count, 1);
    assert!(plan.cascade.is_none());
    let result = apply_delete(&mut feed, &plan);
    assert_eq!(result.primary_count, 1);
    assert_eq!(feed.stops.len(), 3);
}

#[test]
fn validate_delete_accepts_all_targets() {
    let feed = empty_feed();
    let query = parse("stop_id=NOPE").unwrap();

    // Targets that use stop_id
    let plan = validate_delete(&feed, GtfsTarget::Stops, &query).unwrap();
    assert_eq!(plan.matched_count, 0);

    // Spot-check a few more targets with appropriate fields
    let q = parse("trip_id=NOPE").unwrap();
    assert!(validate_delete(&feed, GtfsTarget::Trips, &q).is_ok());
    assert!(validate_delete(&feed, GtfsTarget::StopTimes, &q).is_ok());
    assert!(validate_delete(&feed, GtfsTarget::Frequencies, &q).is_ok());

    let q = parse("route_id=NOPE").unwrap();
    assert!(validate_delete(&feed, GtfsTarget::Routes, &q).is_ok());

    let q = parse("service_id=NOPE").unwrap();
    assert!(validate_delete(&feed, GtfsTarget::Calendar, &q).is_ok());
    assert!(validate_delete(&feed, GtfsTarget::CalendarDates, &q).is_ok());

    let q = parse("pathway_id=NOPE").unwrap();
    assert!(validate_delete(&feed, GtfsTarget::Pathways, &q).is_ok());

    let q = parse("level_id=NOPE").unwrap();
    assert!(validate_delete(&feed, GtfsTarget::Levels, &q).is_ok());

    let q = parse("fare_id=NOPE").unwrap();
    assert!(validate_delete(&feed, GtfsTarget::FareAttributes, &q).is_ok());
    assert!(validate_delete(&feed, GtfsTarget::FareRules, &q).is_ok());
}

#[test]
fn delete_trip_detects_cascade() {
    let feed = feed_with_cascade();
    let query = parse("trip_id=T1").unwrap();
    let plan = validate_delete(&feed, GtfsTarget::Trips, &query).unwrap();
    assert_eq!(plan.matched_count, 1);

    let cascade = plan.cascade.as_ref().expect("should have cascade");
    // T1 has 2 stop_times + 2 frequencies
    let total: usize = cascade.entries.iter().map(|e| e.count).sum();
    assert_eq!(total, 4);

    let st_count = cascade
        .entries
        .iter()
        .find(|e| e.dependent == GtfsTarget::StopTimes)
        .map(|e| e.count);
    assert_eq!(st_count, Some(2));

    let freq_count = cascade
        .entries
        .iter()
        .find(|e| e.dependent == GtfsTarget::Frequencies)
        .map(|e| e.count);
    assert_eq!(freq_count, Some(2));
}

#[test]
fn delete_plan_not_applied_leaves_feed_unchanged() {
    let feed = feed_with_cascade();
    let original_trips = feed.trips.len();
    let original_stop_times = feed.stop_times.len();

    let query = parse("trip_id=T1").unwrap();
    let _plan = validate_delete(&feed, GtfsTarget::Trips, &query).unwrap();

    assert_eq!(feed.trips.len(), original_trips);
    assert_eq!(feed.stop_times.len(), original_stop_times);
}

#[test]
fn delete_route_cascades_to_trips_and_stop_times() {
    let mut feed = feed_with_cascade();
    let query = parse("route_id=R1").unwrap();
    let plan = validate_delete(&feed, GtfsTarget::Routes, &query).unwrap();
    assert_eq!(plan.matched_count, 1);

    let cascade = plan.cascade.as_ref().expect("should have cascade");
    // R1 has trips T1,T2 → T1 has 2 st + 2 freq, T2 has 2 st
    let trip_count = cascade
        .entries
        .iter()
        .find(|e| e.dependent == GtfsTarget::Trips)
        .map(|e| e.count);
    assert_eq!(trip_count, Some(2));

    let st_count = cascade
        .entries
        .iter()
        .find(|e| e.dependent == GtfsTarget::StopTimes)
        .map(|e| e.count);
    assert_eq!(st_count, Some(4));

    let freq_count = cascade
        .entries
        .iter()
        .find(|e| e.dependent == GtfsTarget::Frequencies)
        .map(|e| e.count);
    assert_eq!(freq_count, Some(2));

    let result = apply_delete(&mut feed, &plan);
    assert_eq!(result.primary_count, 1);

    // R2 and its trip T3 should remain
    assert_eq!(feed.routes.len(), 1);
    assert_eq!(feed.routes[0].route_id.as_ref(), "R2");
    assert_eq!(feed.trips.len(), 1);
    assert_eq!(feed.trips[0].trip_id.as_ref(), "T3");
    // Only T3's stop_time remains
    assert_eq!(feed.stop_times.len(), 1);
    assert_eq!(feed.frequencies.len(), 0);
}

#[test]
fn delete_calendar_cascades_to_trips_and_calendar_dates() {
    let mut feed = feed_with_cascade();
    let query = parse("service_id=SVC1").unwrap();
    let plan = validate_delete(&feed, GtfsTarget::Calendar, &query).unwrap();
    assert_eq!(plan.matched_count, 1);

    let cascade = plan.cascade.as_ref().expect("should have cascade");
    // SVC1 referenced by 3 trips + 2 calendar_dates
    // Recursive: 3 trips → 5 stop_times + 2 frequencies
    let total: usize = cascade.entries.iter().map(|e| e.count).sum();
    // 3 trips + 5 stop_times + 2 frequencies + 2 calendar_dates = 12
    assert_eq!(total, 12);

    let result = apply_delete(&mut feed, &plan);
    assert_eq!(feed.calendars.len(), 0);
    assert_eq!(feed.trips.len(), 0);
    assert_eq!(feed.stop_times.len(), 0);
    assert_eq!(feed.frequencies.len(), 0);
    assert_eq!(feed.calendar_dates.len(), 0);
    assert!(result.primary_count == 1);
}

#[test]
fn delete_stop_cascades_to_child_stations() {
    let mut feed = feed_with_cascade();
    // S01 has child S03 (parent_station=S01), plus stop_times referencing S01
    let query = parse("stop_id=S01").unwrap();
    let plan = validate_delete(&feed, GtfsTarget::Stops, &query).unwrap();
    assert_eq!(plan.matched_count, 1);

    let cascade = plan.cascade.as_ref().expect("should have cascade");
    // S01 is referenced by: stop_times (3: T1/1, T2/1, T3/1) + child stop S03
    let stop_deps = cascade
        .entries
        .iter()
        .find(|e| e.dependent == GtfsTarget::Stops)
        .map(|e| e.count);
    assert_eq!(stop_deps, Some(1)); // S03

    let st_deps = cascade
        .entries
        .iter()
        .find(|e| e.dependent == GtfsTarget::StopTimes)
        .map(|e| e.count);
    assert_eq!(st_deps, Some(3));

    let result = apply_delete(&mut feed, &plan);
    // S01 (matched) + S03 (cascade) removed, S02 + S_UNUSED remain
    assert_eq!(feed.stops.len(), 2);
    assert_eq!(result.primary_count, 1);
    // 3 stop_times that referenced S01 are gone, 2 that referenced S02 remain
    assert_eq!(feed.stop_times.len(), 2);
}

#[test]
fn delete_no_match_returns_zero() {
    let feed = feed_with_cascade();
    let query = parse("stop_id=INEXISTANT").unwrap();
    let plan = validate_delete(&feed, GtfsTarget::Stops, &query).unwrap();
    assert_eq!(plan.matched_count, 0);
    assert!(plan.cascade.is_none());
}

#[test]
fn delete_result_has_correct_counts() {
    let mut feed = feed_with_cascade();
    let query = parse("trip_id=T1").unwrap();
    let plan = validate_delete(&feed, GtfsTarget::Trips, &query).unwrap();
    let result = apply_delete(&mut feed, &plan);

    assert_eq!(result.primary_count, 1);
    let total: usize =
        result.primary_count + result.cascade_counts.iter().map(|(_, c)| c).sum::<usize>();
    // 1 trip + 2 stop_times + 2 frequencies = 5
    assert_eq!(total, 5);
    assert!(result.modified_targets.contains(&GtfsTarget::Trips));
    assert!(result.modified_targets.contains(&GtfsTarget::StopTimes));
    assert!(result.modified_targets.contains(&GtfsTarget::Frequencies));
}

#[test]
fn delete_stop_times_has_no_cascade() {
    let mut feed = feed_with_cascade();
    let query = parse("trip_id=T1").unwrap();
    let plan = validate_delete(&feed, GtfsTarget::StopTimes, &query).unwrap();
    assert_eq!(plan.matched_count, 2);
    assert!(plan.cascade.is_none());

    let result = apply_delete(&mut feed, &plan);
    assert_eq!(result.primary_count, 2);
    assert!(result.cascade_counts.is_empty());
    // Remaining stop_times: T2/S01/1, T2/S02/2, T3/S01/1
    assert_eq!(feed.stop_times.len(), 3);
}

#[test]
fn delete_multiple_records_same_target() {
    let mut feed = feed_with_cascade();
    // Both T1 and T2 share route_id=R1
    let query = parse("route_id=R1").unwrap();
    let plan = validate_delete(&feed, GtfsTarget::Trips, &query).unwrap();
    assert_eq!(plan.matched_count, 2);

    // Cascade: T1 has 2st+2freq, T2 has 2st
    let cascade = plan.cascade.as_ref().expect("should have cascade");
    let st_count = cascade
        .entries
        .iter()
        .find(|e| e.dependent == GtfsTarget::StopTimes)
        .map(|e| e.count);
    assert_eq!(st_count, Some(4));

    let result = apply_delete(&mut feed, &plan);
    assert_eq!(result.primary_count, 2);
    assert_eq!(feed.trips.len(), 1); // T3 remains
    assert_eq!(feed.stop_times.len(), 1); // T3's single stop_time remains
}

#[test]
fn delete_feed_info_singleton() {
    let mut feed = empty_feed();
    feed.feed_info = Some(FeedInfo {
        feed_publisher_name: s("Test Publisher"),
        feed_publisher_url: Url::from("http://example.com"),
        feed_lang: LanguageCode::from("en"),
        default_lang: None,
        feed_start_date: None,
        feed_end_date: None,
        feed_version: None,
        feed_contact_email: None,
        feed_contact_url: None,
    });

    let query = parse("feed_publisher_name=Test Publisher").unwrap();
    let plan = validate_delete(&feed, GtfsTarget::FeedInfo, &query).unwrap();
    assert_eq!(plan.matched_count, 1);

    apply_delete(&mut feed, &plan);
    assert!(feed.feed_info.is_none());
}

#[test]
fn delete_translations_by_index() {
    let mut feed = empty_feed();
    feed.translations.push(Translation {
        table_name: s("stops"),
        field_name: s("stop_name"),
        language: LanguageCode::from("fr"),
        translation: s("Arret Un"),
        record_id: Some(s("S01")),
        record_sub_id: None,
        field_value: None,
    });
    feed.translations.push(Translation {
        table_name: s("stops"),
        field_name: s("stop_name"),
        language: LanguageCode::from("es"),
        translation: s("Parada Uno"),
        record_id: Some(s("S01")),
        record_sub_id: None,
        field_value: None,
    });

    let query = parse("language=fr").unwrap();
    let plan = validate_delete(&feed, GtfsTarget::Translations, &query).unwrap();
    assert_eq!(plan.matched_count, 1);

    apply_delete(&mut feed, &plan);
    assert_eq!(feed.translations.len(), 1);
    assert_eq!(feed.translations[0].language, LanguageCode::from("es"));
}

#[test]
fn delete_plan_has_pk_display() {
    let feed = feed_with_cascade();
    let query = parse("trip_id=T1").unwrap();
    let plan = validate_delete(&feed, GtfsTarget::Trips, &query).unwrap();
    assert_eq!(plan.matched_pks, vec!["trip_id=T1"]);
}

#[test]
fn delete_composite_pk_display() {
    let feed = feed_with_cascade();
    let query = parse("trip_id=T1 AND stop_sequence=1").unwrap();
    let plan = validate_delete(&feed, GtfsTarget::StopTimes, &query).unwrap();
    assert_eq!(plan.matched_count, 1);
    assert_eq!(plan.matched_pks[0], "trip_id=T1, stop_sequence=1");
}
