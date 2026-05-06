use std::io::Write;

use gapline_core::models::rt::{
    Alert, FeedEntity, FeedHeader, FeedMessage, GtfsRtFeed, Position, RtError, TripDescriptor,
    TripUpdate, VehiclePosition, feed_header,
};
use prost::Message;
use tempfile::NamedTempFile;

fn header(
    version: &str,
    incrementality: feed_header::Incrementality,
    ts: Option<u64>,
) -> FeedHeader {
    FeedHeader {
        gtfs_realtime_version: version.to_string(),
        incrementality: Some(incrementality as i32),
        timestamp: ts,
        feed_version: None,
    }
}

fn entity_with_trip_update(id: &str, trip_id: &str) -> FeedEntity {
    FeedEntity {
        id: id.to_string(),
        is_deleted: None,
        trip_update: Some(TripUpdate {
            trip: TripDescriptor {
                trip_id: Some(trip_id.to_string()),
                ..Default::default()
            },
            ..Default::default()
        }),
        vehicle: None,
        alert: None,
        shape: None,
        stop: None,
        trip_modifications: None,
    }
}

fn entity_with_vehicle(id: &str, lat: f32, lon: f32) -> FeedEntity {
    FeedEntity {
        id: id.to_string(),
        is_deleted: None,
        trip_update: None,
        vehicle: Some(VehiclePosition {
            position: Some(Position {
                latitude: lat,
                longitude: lon,
                ..Default::default()
            }),
            ..Default::default()
        }),
        alert: None,
        shape: None,
        stop: None,
        trip_modifications: None,
    }
}

fn entity_with_alert(id: &str) -> FeedEntity {
    FeedEntity {
        id: id.to_string(),
        is_deleted: None,
        trip_update: None,
        vehicle: None,
        alert: Some(Alert::default()),
        shape: None,
        stop: None,
        trip_modifications: None,
    }
}

fn encode(msg: &FeedMessage) -> Vec<u8> {
    let mut buf = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut buf).unwrap();
    buf
}

#[test]
fn from_bytes_decodes_trip_updates_and_vehicles() {
    let msg = FeedMessage {
        header: header(
            "2.0",
            feed_header::Incrementality::FullDataset,
            Some(1_700_000_000),
        ),
        entity: vec![
            entity_with_trip_update("e1", "trip-A"),
            entity_with_trip_update("e2", "trip-B"),
            entity_with_vehicle("e3", 45.5, -73.6),
        ],
    };
    let feed = GtfsRtFeed::from_bytes(&encode(&msg)).unwrap();

    assert_eq!(feed.trip_updates().len(), 2);
    assert_eq!(feed.vehicle_positions().len(), 1);
    assert!(feed.alerts().is_empty());
    assert_eq!(
        feed.trip_updates()[0].trip.trip_id.as_deref(),
        Some("trip-A")
    );
}

#[test]
fn alerts_only_feed() {
    let msg = FeedMessage {
        header: header("2.0", feed_header::Incrementality::FullDataset, None),
        entity: vec![entity_with_alert("a1"), entity_with_alert("a2")],
    };
    let feed = GtfsRtFeed::from_bytes(&encode(&msg)).unwrap();

    assert_eq!(feed.alerts().len(), 2);
    assert!(feed.trip_updates().is_empty());
    assert!(feed.vehicle_positions().is_empty());
}

#[test]
fn not_protobuf_returns_decode_error() {
    let garbage = b"this is plain text, not a protobuf message at all";
    let err = GtfsRtFeed::from_bytes(garbage).unwrap_err();
    assert!(matches!(err, RtError::Decode(_)));
}

#[test]
fn truncated_protobuf_returns_decode_error() {
    let msg = FeedMessage {
        header: header("2.0", feed_header::Incrementality::FullDataset, Some(1)),
        entity: vec![entity_with_trip_update("e1", "trip-A")],
    };
    let bytes = encode(&msg);
    let half = &bytes[..bytes.len() / 2];
    let err = GtfsRtFeed::from_bytes(half).unwrap_err();
    assert!(matches!(err, RtError::Decode(_)));
}

#[test]
fn empty_feed_parses_with_empty_collections() {
    let msg = FeedMessage {
        header: header("2.0", feed_header::Incrementality::FullDataset, None),
        entity: vec![],
    };
    let feed = GtfsRtFeed::from_bytes(&encode(&msg)).unwrap();

    assert!(feed.trip_updates().is_empty());
    assert!(feed.vehicle_positions().is_empty());
    assert!(feed.alerts().is_empty());
}

#[test]
fn header_fields_accessible() {
    let msg = FeedMessage {
        header: header("2.0", feed_header::Incrementality::FullDataset, Some(42)),
        entity: vec![],
    };
    let feed = GtfsRtFeed::from_bytes(&encode(&msg)).unwrap();

    assert_eq!(feed.gtfs_realtime_version(), "2.0");
    assert_eq!(
        feed.incrementality(),
        feed_header::Incrementality::FullDataset
    );
    assert_eq!(feed.timestamp(), Some(42));
}

#[test]
fn from_file_decodes_pb() {
    let msg = FeedMessage {
        header: header("2.0", feed_header::Incrementality::FullDataset, None),
        entity: vec![entity_with_trip_update("e1", "trip-X")],
    };
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&encode(&msg)).unwrap();
    tmp.flush().unwrap();

    let feed = GtfsRtFeed::from_file(tmp.path()).unwrap();
    assert_eq!(feed.trip_updates().len(), 1);
}

#[test]
fn unknown_version_still_parses() {
    let msg = FeedMessage {
        header: header("3.0", feed_header::Incrementality::FullDataset, None),
        entity: vec![],
    };
    let feed = GtfsRtFeed::from_bytes(&encode(&msg)).unwrap();
    assert_eq!(feed.gtfs_realtime_version(), "3.0");
}

#[test]
fn trip_update_without_stop_time_updates() {
    let msg = FeedMessage {
        header: header("2.0", feed_header::Incrementality::FullDataset, None),
        entity: vec![entity_with_trip_update("e1", "trip-A")],
    };
    let feed = GtfsRtFeed::from_bytes(&encode(&msg)).unwrap();
    assert!(feed.trip_updates()[0].stop_time_update.is_empty());
}

#[test]
fn from_file_io_error_on_missing_path() {
    let err = GtfsRtFeed::from_file(std::path::Path::new("/nonexistent/path.pb")).unwrap_err();
    assert!(matches!(err, RtError::Io(_)));
}
