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

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

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

criterion_group!(benches, bench_feed_loading, bench_validation);
criterion_main!(benches);
