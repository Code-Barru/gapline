use std::io::Write;
use std::path::Path;

use gapline_core::parser::FeedLoader;

fn create_minimal_feed(dir: &Path) {
    let files: &[(&str, &str)] = &[
        (
            "agency.txt",
            "agency_id,agency_name,agency_url,agency_timezone\nSTM,STM,http://stm.info,America/Montreal\n",
        ),
        (
            "stops.txt",
            "stop_id,stop_name,stop_lat,stop_lon\nS1,Gare,45.5,-73.6\n",
        ),
        ("routes.txt", "route_id,route_type\nR1,3\n"),
        ("trips.txt", "route_id,service_id,trip_id\nR1,SVC1,T1\n"),
        (
            "stop_times.txt",
            "trip_id,arrival_time,departure_time,stop_id,stop_sequence\nT1,08:00:00,08:01:00,S1,1\n",
        ),
        (
            "calendar.txt",
            "service_id,monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date\nSVC1,1,1,1,1,1,0,0,20240101,20241231\n",
        ),
    ];

    for (name, content) in files {
        let mut f = std::fs::File::create(dir.join(name)).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }
}

// -- TC9: absent optional file -> empty vec
#[test]
fn absent_optional_file_empty_vec() {
    let tmp = tempfile::tempdir().unwrap();
    create_minimal_feed(tmp.path());

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);

    assert!(feed.shapes.is_empty());
    assert!(feed.frequencies.is_empty());
    assert!(feed.feed_info.is_none());
    // No errors related to absent optional files
    let shape_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.file_name == "shapes.txt")
        .collect();
    assert!(shape_errors.is_empty());
}

// -- TC10: all required files parsed
#[test]
fn all_required_files_parsed() {
    let tmp = tempfile::tempdir().unwrap();
    create_minimal_feed(tmp.path());

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);

    assert_eq!(feed.agencies.len(), 1);
    assert_eq!(feed.stops.len(), 1);
    assert_eq!(feed.routes.len(), 1);
    assert_eq!(feed.trips.len(), 1);
    assert_eq!(feed.stop_times.len(), 1);
    assert_eq!(feed.calendars.len(), 1);
    assert!(errors.is_empty());
}

// -- TC14: parallel parsing of a full feed
#[test]
fn parallel_parsing_full_feed() {
    let tmp = tempfile::tempdir().unwrap();
    create_minimal_feed(tmp.path());

    // Add optional files
    let extras: &[(&str, &str)] = &[
        (
            "calendar_dates.txt",
            "service_id,date,exception_type\nSVC1,20240701,1\n",
        ),
        (
            "shapes.txt",
            "shape_id,shape_pt_lat,shape_pt_lon,shape_pt_sequence\nSH1,45.5,-73.6,1\n",
        ),
        (
            "frequencies.txt",
            "trip_id,start_time,end_time,headway_secs\nT1,06:00:00,09:00:00,300\n",
        ),
        (
            "transfers.txt",
            "from_stop_id,to_stop_id,transfer_type\nS1,S1,0\n",
        ),
        (
            "feed_info.txt",
            "feed_publisher_name,feed_publisher_url,feed_lang\nACME,http://acme.com,en\n",
        ),
        (
            "fare_attributes.txt",
            "fare_id,price,currency_type,payment_method\nF1,2.50,CAD,0\n",
        ),
        ("fare_rules.txt", "fare_id,route_id\nF1,R1\n"),
    ];

    for (name, content) in extras {
        let mut f = std::fs::File::create(tmp.path().join(name)).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);

    assert!(errors.is_empty());
    assert_eq!(feed.agencies.len(), 1);
    assert_eq!(feed.calendar_dates.len(), 1);
    assert_eq!(feed.shapes.len(), 1);
    assert_eq!(feed.frequencies.len(), 1);
    assert_eq!(feed.transfers.len(), 1);
    assert!(feed.feed_info.is_some());
    assert_eq!(feed.fare_attributes.len(), 1);
    assert_eq!(feed.fare_rules.len(), 1);
}

// -- TC16/17: feed_info present/absent
#[test]
fn feed_info_present() {
    let tmp = tempfile::tempdir().unwrap();
    create_minimal_feed(tmp.path());

    let mut f = std::fs::File::create(tmp.path().join("feed_info.txt")).unwrap();
    f.write_all(b"feed_publisher_name,feed_publisher_url,feed_lang\nACME,http://acme.com,en\n")
        .unwrap();

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, _) = FeedLoader::load(&source);

    let info = feed.feed_info.unwrap();
    assert_eq!(info.feed_publisher_name, "ACME");
}

#[test]
fn feed_info_absent() {
    let tmp = tempfile::tempdir().unwrap();
    create_minimal_feed(tmp.path());

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, _) = FeedLoader::load(&source);

    assert!(feed.feed_info.is_none());
}

// -- Errors collected separately
#[test]
fn errors_collected_separately() {
    let tmp = tempfile::tempdir().unwrap();

    let mut f = std::fs::File::create(tmp.path().join("agency.txt")).unwrap();
    f.write_all(b"agency_id,agency_name,agency_url,agency_timezone\nSTM,,http://stm.info,America/Montreal\n").unwrap();

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);

    assert_eq!(feed.agencies.len(), 1);
    assert_eq!(feed.agencies[0].agency_name, "");
    assert!(!errors.is_empty());
    assert_eq!(errors[0].field_name, "agency_name");
}
