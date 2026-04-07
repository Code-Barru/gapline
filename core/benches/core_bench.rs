//! Criterion benchmarks for headway-core.
//!
//! Covers the two main hot paths:
//! - **Feed loading** — `FeedLoader::open()` + `FeedLoader::load()` from a ZIP archive.
//! - **Validation**  — `ValidationEngine::validate_structural()` + `validate_feed()`.
//!
//! Run with: `cargo bench -p headway-core`

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use headway_core::config::Config;
use headway_core::parser::FeedLoader;
use headway_core::validation::engine::ValidationEngine;

/// Creates a minimal valid GTFS zip on disk and returns its path.
///
/// The file is written to a temporary directory that lives as long as the
/// returned `tempfile::TempDir` handle.
fn create_valid_feed() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let zip_path = dir.path().join("feed.zip");

    let file = std::fs::File::create(&zip_path).expect("failed to create zip file");
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default();

    zip.start_file("agency.txt", opts).unwrap();
    zip.write_all(
        b"agency_id,agency_name,agency_url,agency_timezone\n\
          A1,Agency,http://a.com,America/New_York\n",
    )
    .unwrap();

    zip.start_file("routes.txt", opts).unwrap();
    zip.write_all(
        b"route_id,agency_id,route_short_name,route_long_name,route_type\n\
          R1,A1,1,Route One,3\n",
    )
    .unwrap();

    zip.start_file("trips.txt", opts).unwrap();
    zip.write_all(b"route_id,service_id,trip_id\nR1,S1,T1\n")
        .unwrap();

    zip.start_file("stops.txt", opts).unwrap();
    zip.write_all(b"stop_id,stop_name,stop_lat,stop_lon\nST1,Stop One,40.0,-74.0\n")
        .unwrap();

    zip.start_file("stop_times.txt", opts).unwrap();
    zip.write_all(
        b"trip_id,arrival_time,departure_time,stop_id,stop_sequence\n\
          T1,08:00:00,08:00:00,ST1,1\n",
    )
    .unwrap();

    zip.start_file("calendar.txt", opts).unwrap();
    zip.write_all(
        b"service_id,monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date\n\
          S1,1,1,1,1,1,0,0,20240101,20241231\n",
    )
    .unwrap();

    zip.finish().unwrap();
    (dir, zip_path)
}

fn bench_feed_loading(c: &mut Criterion) {
    let (_dir, zip_path) = create_valid_feed();

    c.bench_function("feed_loading/open", |b| {
        b.iter(|| {
            let source = FeedLoader::open(black_box(&zip_path)).expect("open failed");
            black_box(source);
        });
    });

    c.bench_function("feed_loading/open+load", |b| {
        b.iter(|| {
            let source = FeedLoader::open(black_box(&zip_path)).expect("open failed");
            let (feed, errors) = FeedLoader::load(&source);
            black_box((feed, errors));
        });
    });
}

fn bench_validation(c: &mut Criterion) {
    let (_dir, zip_path) = create_valid_feed();
    let source = FeedLoader::open(&zip_path).expect("open failed");
    let config = Arc::new(Config {
        quiet: true,
        ..Config::default()
    });

    c.bench_function("validation/structural", |b| {
        let engine = ValidationEngine::new(Arc::clone(&config));
        b.iter(|| {
            let report = engine.validate_structural(black_box(&source));
            black_box(report);
        });
    });

    c.bench_function("validation/full", |b| {
        b.iter(|| {
            let source = FeedLoader::open(black_box(&zip_path)).expect("open failed");
            let engine = ValidationEngine::new(Arc::clone(&config));
            let _structural = engine.validate_structural(&source);
            let (feed, parse_errors) = FeedLoader::load(&source);
            let report = engine.validate_feed(&feed, &parse_errors);
            black_box(report);
        });
    });
}

/// Create a large synthetic feed: 1 agency, 10 routes, 1 calendar,
/// 200 stops, 1000 trips with 100 `stop_times` each (100k total).
fn create_large_feed() -> headway_core::models::GtfsFeed {
    use headway_core::models::{
        Agency, AgencyId, Calendar, GtfsDate, GtfsFeed, Route, RouteId, RouteType, ServiceId, Stop,
        StopId, StopTime, Timezone, Trip, TripId, Url,
    };

    let mut feed = GtfsFeed::default();

    feed.agencies.push(Agency {
        agency_id: Some(AgencyId::from("A1")),
        agency_name: "Bench Agency".into(),
        agency_url: Url::from("http://bench.test"),
        agency_timezone: Timezone::from("UTC"),
        agency_lang: None,
        agency_phone: None,
        agency_fare_url: None,
        agency_email: None,
    });

    for route_index in 0..10 {
        feed.routes.push(Route {
            route_id: RouteId::from(format!("R{route_index}")),
            agency_id: Some(AgencyId::from("A1")),
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
        });
    }

    feed.calendars.push(Calendar {
        service_id: ServiceId::from("SVC1"),
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        start_date: GtfsDate::default(),
        end_date: GtfsDate::default(),
    });

    for stop_index in 0..200 {
        feed.stops.push(Stop {
            stop_id: StopId::from(format!("S{stop_index}")),
            stop_code: None,
            stop_name: None,
            tts_stop_name: None,
            stop_desc: None,
            stop_lat: None,
            stop_lon: None,
            zone_id: None,
            stop_url: None,
            location_type: None,
            parent_station: None,
            stop_timezone: None,
            wheelchair_boarding: None,
            level_id: None,
            platform_code: None,
        });
    }

    for trip_index in 0..1000 {
        let route_id = format!("R{}", trip_index % 10);
        let trip_id = format!("T{trip_index}");
        feed.trips.push(Trip {
            route_id: RouteId::from(route_id),
            service_id: ServiceId::from("SVC1"),
            trip_id: TripId::from(trip_id.clone()),
            trip_headsign: None,
            trip_short_name: None,
            direction_id: None,
            block_id: None,
            shape_id: None,
            wheelchair_accessible: None,
            bikes_allowed: None,
        });

        for sequence in 0..100 {
            let stop_id = format!("S{}", sequence % 200);
            feed.stop_times.push(StopTime {
                trip_id: TripId::from(trip_id.clone()),
                arrival_time: None,
                departure_time: None,
                stop_id: StopId::from(stop_id),
                stop_sequence: sequence,
                stop_headsign: None,
                pickup_type: None,
                drop_off_type: None,
                continuous_pickup: None,
                continuous_drop_off: None,
                shape_dist_traveled: None,
                timepoint: None,
            });
        }
    }

    feed
}

fn bench_integrity_index(c: &mut Criterion) {
    let (_dir, zip_path) = create_valid_feed();
    let source = FeedLoader::open(&zip_path).expect("open failed");
    let (small_feed, _) = FeedLoader::load(&source);

    c.bench_function("integrity/build_small", |b| {
        b.iter(|| {
            let index =
                headway_core::integrity::IntegrityIndex::build_from_feed(black_box(&small_feed));
            black_box(index);
        });
    });

    let large_feed = create_large_feed();

    c.bench_function("integrity/build_100k_stop_times", |b| {
        b.iter(|| {
            let index =
                headway_core::integrity::IntegrityIndex::build_from_feed(black_box(&large_feed));
            black_box(index);
        });
    });

    let large_index = headway_core::integrity::IntegrityIndex::build_from_feed(&large_feed);

    c.bench_function("integrity/find_dependents", |b| {
        let route =
            headway_core::integrity::EntityRef::Route(headway_core::models::RouteId::from("R0"));
        b.iter(|| {
            let dependents = large_index.find_dependents(black_box(&route));
            black_box(dependents);
        });
    });

    c.bench_function("integrity/find_dependents_recursive", |b| {
        let route =
            headway_core::integrity::EntityRef::Route(headway_core::models::RouteId::from("R0"));
        b.iter(|| {
            let dependents = large_index.find_dependents_recursive(black_box(&route));
            black_box(dependents);
        });
    });
}

criterion_group!(
    benches,
    bench_feed_loading,
    bench_validation,
    bench_integrity_index
);
criterion_main!(benches);
