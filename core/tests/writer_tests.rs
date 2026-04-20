use std::collections::HashSet;
use std::io::{Cursor, Read};

use gapline_core::models::*;
use gapline_core::parser::FeedLoader;
use gapline_core::writer::write_feed;

use chrono::NaiveDate;
use tempfile::TempDir;

fn gtfs_date(y: i32, m: u32, d: u32) -> GtfsDate {
    GtfsDate(NaiveDate::from_ymd_opt(y, m, d).unwrap())
}

fn minimal_feed() -> GtfsFeed {
    let mut feed = GtfsFeed::default();
    feed.agencies.push(Agency {
        agency_id: Some(AgencyId::from("A1")),
        agency_name: "Test Agency".into(),
        agency_url: Url::from("http://example.com"),
        agency_timezone: Timezone::from("America/Montreal"),
        agency_lang: None,
        agency_phone: None,
        agency_fare_url: None,
        agency_email: None,
    });
    feed.stops.push(Stop {
        stop_id: StopId::from("S1"),
        stop_code: None,
        stop_name: Some("Gare A".into()),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(45.5)),
        stop_lon: Some(Longitude(-73.6)),
        zone_id: None,
        stop_url: None,
        location_type: None,
        parent_station: None,
        stop_timezone: None,
        wheelchair_boarding: None,
        level_id: None,
        platform_code: None,
    });
    feed.routes.push(Route {
        route_id: RouteId::from("R1"),
        agency_id: Some(AgencyId::from("A1")),
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
    });
    feed.calendars.push(Calendar {
        service_id: ServiceId::from("SVC1"),
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        start_date: gtfs_date(2025, 1, 1),
        end_date: gtfs_date(2025, 12, 31),
    });
    feed.trips.push(Trip {
        route_id: RouteId::from("R1"),
        service_id: ServiceId::from("SVC1"),
        trip_id: TripId::from("T1"),
        trip_headsign: None,
        trip_short_name: None,
        direction_id: None,
        block_id: None,
        shape_id: None,
        wheelchair_accessible: None,
        bikes_allowed: None,
    });
    feed
}

#[test]
fn write_feed_roundtrip() {
    let feed = minimal_feed();
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("feed.zip");

    write_feed(&feed, &path).unwrap();

    // Re-read the written zip
    let source = FeedLoader::open(&path).unwrap();
    let (reloaded, errors) = FeedLoader::load(&source);

    assert!(errors.is_empty(), "Parse errors: {errors:?}");
    assert_eq!(reloaded.agencies.len(), 1);
    assert_eq!(reloaded.agencies[0].agency_name, "Test Agency");
    assert_eq!(reloaded.stops.len(), 1);
    assert_eq!(reloaded.stops[0].stop_id.as_ref(), "S1");
    assert_eq!(reloaded.routes.len(), 1);
    assert_eq!(reloaded.calendars.len(), 1);
    assert!(reloaded.calendars[0].monday);
    assert!(!reloaded.calendars[0].saturday);
    assert_eq!(reloaded.trips.len(), 1);
}

#[test]
fn write_empty_collections_not_included() {
    let feed = GtfsFeed::default();
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("feed.zip");

    write_feed(&feed, &path).unwrap();

    // Read the ZIP and check file list
    let mut buf = Vec::new();
    std::fs::File::open(&path)
        .unwrap()
        .read_to_end(&mut buf)
        .unwrap();
    let reader = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();

    let file_names: HashSet<String> = (0..reader.len())
        .map(|i| reader.name_for_index(i).unwrap().to_string())
        .collect();

    assert!(
        file_names.is_empty(),
        "Expected empty ZIP but found: {file_names:?}"
    );
}

#[test]
fn write_preserves_optional_fields() {
    let mut feed = GtfsFeed::default();
    feed.stops.push(Stop {
        stop_id: StopId::from("S1"),
        stop_code: None,
        stop_name: Some("Test".into()),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(45.5)),
        stop_lon: None, // intentionally None
        zone_id: None,
        stop_url: None,
        location_type: None,
        parent_station: None,
        stop_timezone: None,
        wheelchair_boarding: None,
        level_id: None,
        platform_code: None,
    });

    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("feed.zip");

    write_feed(&feed, &path).unwrap();

    let source = FeedLoader::open(&path).unwrap();
    let (reloaded, _) = FeedLoader::load(&source);

    assert_eq!(reloaded.stops.len(), 1);
    assert_eq!(reloaded.stops[0].stop_name.as_deref(), Some("Test"));
    assert!(reloaded.stops[0].stop_lon.is_none());
}

fn add_schedule_records(feed: &mut GtfsFeed) {
    feed.stop_times.push(StopTime {
        trip_id: TripId::from("T1"),
        arrival_time: Some(GtfsTime::from_hms(8, 0, 0)),
        departure_time: Some(GtfsTime::from_hms(8, 1, 0)),
        stop_id: StopId::from("S1"),
        stop_sequence: 1,
        stop_headsign: None,
        pickup_type: None,
        drop_off_type: None,
        continuous_pickup: None,
        continuous_drop_off: None,
        shape_dist_traveled: None,
        timepoint: None,
    });
    feed.calendar_dates.push(CalendarDate {
        service_id: ServiceId::from("SVC1"),
        date: gtfs_date(2025, 12, 25),
        exception_type: ExceptionType::Removed,
    });
    feed.shapes.push(Shape {
        shape_id: ShapeId::from("SH1"),
        shape_pt_lat: Latitude(45.5),
        shape_pt_lon: Longitude(-73.6),
        shape_pt_sequence: 1,
        shape_dist_traveled: None,
    });
    feed.frequencies.push(Frequency {
        trip_id: TripId::from("T1"),
        start_time: GtfsTime::from_hms(6, 0, 0),
        end_time: GtfsTime::from_hms(9, 0, 0),
        headway_secs: 600,
        exact_times: None,
    });
    feed.transfers.push(Transfer {
        from_stop_id: Some(StopId::from("S1")),
        to_stop_id: Some(StopId::from("S1")),
        from_route_id: None,
        to_route_id: None,
        from_trip_id: None,
        to_trip_id: None,
        transfer_type: TransferType::Recommended,
        min_transfer_time: None,
    });
    feed.pathways.push(Pathway {
        pathway_id: PathwayId::from("PW1"),
        from_stop_id: StopId::from("S1"),
        to_stop_id: StopId::from("S1"),
        pathway_mode: PathwayMode::Walkway,
        is_bidirectional: IsBidirectional::Bidirectional,
        length: None,
        traversal_time: None,
        stair_count: None,
        max_slope: None,
        min_width: None,
        signposted_as: None,
        reversed_signposted_as: None,
    });
}

fn add_meta_records(feed: &mut GtfsFeed) {
    feed.levels.push(Level {
        level_id: LevelId::from("L1"),
        level_index: 0.0,
        level_name: None,
    });
    feed.feed_info = Some(FeedInfo {
        feed_publisher_name: "Test".into(),
        feed_publisher_url: Url::from("http://example.com"),
        feed_lang: LanguageCode::from("en"),
        default_lang: None,
        feed_start_date: None,
        feed_end_date: None,
        feed_version: None,
        feed_contact_email: None,
        feed_contact_url: None,
    });
    feed.fare_attributes.push(FareAttribute {
        fare_id: FareId::from("F1"),
        price: 2.50,
        currency_type: CurrencyCode::from("CAD"),
        payment_method: 0,
        transfers: Some(0),
        agency_id: None,
        transfer_duration: None,
    });
    feed.fare_rules.push(FareRule {
        fare_id: FareId::from("F1"),
        route_id: None,
        origin_id: None,
        destination_id: None,
        contains_id: None,
    });
    feed.translations.push(Translation {
        table_name: "stops".into(),
        field_name: "stop_name".into(),
        language: LanguageCode::from("fr"),
        translation: "Gare A".into(),
        record_id: None,
        record_sub_id: None,
        field_value: None,
    });
    feed.attributions.push(Attribution {
        attribution_id: None,
        agency_id: None,
        route_id: None,
        trip_id: None,
        organization_name: "Test Org".into(),
        is_producer: None,
        is_operator: None,
        is_authority: None,
        attribution_url: None,
        attribution_email: None,
        attribution_phone: None,
    });
}

fn full_feed() -> GtfsFeed {
    let mut feed = minimal_feed();
    add_schedule_records(&mut feed);
    add_meta_records(&mut feed);
    feed
}

#[test]
fn write_all_17_targets() {
    let feed = full_feed();
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("feed.zip");

    write_feed(&feed, &path).unwrap();

    // Verify all files are present in the ZIP
    let mut buf = Vec::new();
    std::fs::File::open(&path)
        .unwrap()
        .read_to_end(&mut buf)
        .unwrap();
    let reader = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();

    let file_names: HashSet<String> = (0..reader.len())
        .map(|i| reader.name_for_index(i).unwrap().to_string())
        .collect();

    let expected = [
        "agency.txt",
        "stops.txt",
        "routes.txt",
        "trips.txt",
        "stop_times.txt",
        "calendar.txt",
        "calendar_dates.txt",
        "shapes.txt",
        "frequencies.txt",
        "transfers.txt",
        "pathways.txt",
        "levels.txt",
        "feed_info.txt",
        "fare_attributes.txt",
        "fare_rules.txt",
        "translations.txt",
        "attributions.txt",
    ];

    for name in &expected {
        assert!(file_names.contains(*name), "Missing file: {name}");
    }
}
