//! Integration tests for RT auto-detection in `gapline validate`.

use std::io::Write;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use gapline_core::models::rt::transit_realtime::{
    FeedEntity, FeedHeader, FeedMessage, TripDescriptor, TripUpdate, feed_header,
};
use prost::Message;
use tempfile::NamedTempFile;

fn gapline_bin() -> String {
    env!("CARGO_BIN_EXE_gapline").to_string()
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Builds a minimal valid GTFS-RT `FeedMessage` (`gtfs_realtime_version=2.0`,
/// `FULL_DATASET`, current timestamp) and writes it to a `.pb` tempfile.
fn create_minimal_rt_pb() -> NamedTempFile {
    let msg = FeedMessage {
        header: FeedHeader {
            gtfs_realtime_version: "2.0".to_string(),
            incrementality: Some(feed_header::Incrementality::FullDataset as i32),
            timestamp: Some(now_unix()),
            feed_version: None,
        },
        entity: Vec::new(),
    };
    let bytes = msg.encode_to_vec();
    let tmp = tempfile::Builder::new().suffix(".pb").tempfile().unwrap();
    std::fs::write(tmp.path(), &bytes).unwrap();
    tmp
}

/// RT feed referencing a `trip_id` — matches a trip in `create_minimal_schedule`.
fn create_rt_with_trip(trip_id: &str) -> NamedTempFile {
    let msg = FeedMessage {
        header: FeedHeader {
            gtfs_realtime_version: "2.0".to_string(),
            incrementality: Some(feed_header::Incrementality::FullDataset as i32),
            timestamp: Some(now_unix()),
            feed_version: None,
        },
        entity: vec![FeedEntity {
            id: "e1".to_string(),
            trip_update: Some(TripUpdate {
                trip: TripDescriptor {
                    trip_id: Some(trip_id.to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        }],
    };
    let bytes = msg.encode_to_vec();
    let tmp = tempfile::Builder::new().suffix(".pb").tempfile().unwrap();
    std::fs::write(tmp.path(), &bytes).unwrap();
    tmp
}

fn create_minimal_schedule() -> NamedTempFile {
    let tmp = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
    let file = std::fs::File::create(tmp.path()).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default();

    zip.start_file("agency.txt", opts).unwrap();
    zip.write_all(b"agency_id,agency_name,agency_url,agency_timezone\nA1,Agency,http://a.com,America/New_York\n").unwrap();

    zip.start_file("routes.txt", opts).unwrap();
    zip.write_all(
        b"route_id,agency_id,route_short_name,route_long_name,route_type\nR1,A1,1,Route One,3\n",
    )
    .unwrap();

    zip.start_file("trips.txt", opts).unwrap();
    zip.write_all(b"route_id,service_id,trip_id\nR1,S1,T1\n")
        .unwrap();

    zip.start_file("stops.txt", opts).unwrap();
    zip.write_all(b"stop_id,stop_name,stop_lat,stop_lon\nST1,Stop One,40.0,-74.0\nST2,Stop Two,40.01,-74.01\n").unwrap();

    zip.start_file("stop_times.txt", opts).unwrap();
    zip.write_all(b"trip_id,arrival_time,departure_time,stop_id,stop_sequence\nT1,08:00:00,08:00:00,ST1,1\nT1,08:05:00,08:05:00,ST2,2\n").unwrap();

    zip.start_file("calendar.txt", opts).unwrap();
    zip.write_all(b"service_id,monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date\nS1,1,1,1,1,1,0,0,20240101,20241231\n").unwrap();

    zip.finish().unwrap();
    tmp
}

#[test]
fn rt_only_validates_clean() {
    let rt = create_minimal_rt_pb();
    let out = Command::new(gapline_bin())
        .args(["validate", "-f"])
        .arg(rt.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("cross-validation rules skipped"),
        "expected skip notice, got stderr: {stderr}"
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "stdout: {}\nstderr: {stderr}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn rt_plus_schedule_validates_clean() {
    let rt = create_rt_with_trip("T1");
    let sched = create_minimal_schedule();
    let out = Command::new(gapline_bin())
        .args(["validate", "-f"])
        .arg(rt.path())
        .arg("-f")
        .arg(sched.path())
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn order_of_feeds_does_not_matter() {
    let rt = create_rt_with_trip("T1");
    let sched = create_minimal_schedule();
    let out = Command::new(gapline_bin())
        .args(["validate", "-f"])
        .arg(sched.path())
        .arg("-f")
        .arg(rt.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn rt_with_orphan_trip_exits_with_error() {
    let rt = create_rt_with_trip("ORPHAN_TRIP");
    let sched = create_minimal_schedule();
    let out = Command::new(gapline_bin())
        .args(["validate", "-f"])
        .arg(rt.path())
        .arg("-f")
        .arg(sched.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn invalid_protobuf_returns_input_error() {
    let tmp = tempfile::Builder::new().suffix(".pb").tempfile().unwrap();
    std::fs::write(tmp.path(), b"not a protobuf at all").unwrap();
    let out = Command::new(gapline_bin())
        .args(["validate", "-f"])
        .arg(tmp.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("invalid protobuf"),
        "expected protobuf error, got: {stderr}"
    );
    assert_eq!(out.status.code(), Some(3));
}

#[test]
fn two_rt_feeds_rejected() {
    let rt1 = create_minimal_rt_pb();
    let rt2 = create_minimal_rt_pb();
    let out = Command::new(gapline_bin())
        .args(["validate", "-f"])
        .arg(rt1.path())
        .arg("-f")
        .arg(rt2.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("two GTFS-RT feeds"),
        "expected duplicate-RT error, got: {stderr}"
    );
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn schedule_only_pipeline_unchanged() {
    let sched = create_minimal_schedule();
    let out = Command::new(gapline_bin())
        .args(["validate", "-f"])
        .arg(sched.path())
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn rt_json_output_is_valid_json() {
    let rt = create_minimal_rt_pb();
    let out = Command::new(gapline_bin())
        .args(["validate", "-f"])
        .arg(rt.path())
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let parsed: serde_json::Value = serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout was not valid JSON: {e}; raw: {:?}", out.stdout));
    assert!(parsed.is_object(), "expected JSON object root");
}
