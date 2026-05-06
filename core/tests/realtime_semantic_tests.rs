//! Tests for section 12 — GTFS-Realtime semantic validation.

use std::sync::Arc;

use gapline_core::config::Config;
use gapline_core::models::rt::{
    Alert, EntitySelector, FeedEntity, FeedHeader, FeedMessage, GtfsRtFeed, Position,
    TripDescriptor, TripUpdate, VehiclePosition, feed_header,
    trip_update::{StopTimeEvent, StopTimeUpdate},
};
use gapline_core::models::{
    GtfsFeed, GtfsTime, Latitude, LocationType, Longitude, Route, RouteType, Stop, StopId,
    StopTime, Trip,
};
use gapline_core::validation::ValidationEngine;
use prost::Message;

const NOW: u64 = 1_700_000_000;
const NOW_I: i64 = 1_700_000_000;

fn header(version: &str, ts: Option<u64>) -> FeedHeader {
    FeedHeader {
        gtfs_realtime_version: version.to_string(),
        incrementality: Some(feed_header::Incrementality::FullDataset as i32),
        timestamp: ts,
        feed_version: None,
    }
}

fn empty_entity(id: &str) -> FeedEntity {
    FeedEntity {
        id: id.to_string(),
        is_deleted: None,
        trip_update: None,
        vehicle: None,
        alert: None,
        shape: None,
        stop: None,
        trip_modifications: None,
    }
}

fn trip_descriptor(trip_id: Option<&str>, route_id: Option<&str>) -> TripDescriptor {
    TripDescriptor {
        trip_id: trip_id.map(String::from),
        route_id: route_id.map(String::from),
        ..Default::default()
    }
}

fn ev(time: Option<i64>, delay: Option<i32>) -> StopTimeEvent {
    StopTimeEvent {
        time,
        delay,
        ..Default::default()
    }
}

fn stu(
    stop_id: Option<&str>,
    arrival: Option<StopTimeEvent>,
    departure: Option<StopTimeEvent>,
) -> StopTimeUpdate {
    StopTimeUpdate {
        stop_sequence: None,
        stop_id: stop_id.map(String::from),
        arrival,
        departure,
        ..Default::default()
    }
}

fn entity_with_trip_update(id: &str, tu: TripUpdate) -> FeedEntity {
    FeedEntity {
        trip_update: Some(tu),
        ..empty_entity(id)
    }
}

fn entity_with_vehicle(id: &str, vp: VehiclePosition) -> FeedEntity {
    FeedEntity {
        vehicle: Some(vp),
        ..empty_entity(id)
    }
}

fn entity_with_alert(id: &str, alert: Alert) -> FeedEntity {
    FeedEntity {
        alert: Some(alert),
        ..empty_entity(id)
    }
}

fn rt_feed(msg: &FeedMessage) -> GtfsRtFeed {
    let mut buf = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut buf).unwrap();
    GtfsRtFeed::from_bytes(&buf).unwrap()
}

// Schedule fixture: 2 stops near Paris, 1 trip "T1" on route "R1".
fn schedule_paris() -> GtfsFeed {
    let mut feed = GtfsFeed::default();
    feed.stops.push(Stop {
        stop_id: StopId::from("S1"),
        stop_code: None,
        stop_name: Some("Stop 1".into()),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(48.85)),
        stop_lon: Some(Longitude(2.34)),
        zone_id: None,
        stop_url: None,
        location_type: None,
        parent_station: None,
        stop_timezone: None,
        wheelchair_boarding: None,
        level_id: None,
        platform_code: None,
    });
    feed.stops.push(Stop {
        stop_id: StopId::from("S2"),
        stop_code: None,
        stop_name: Some("Stop 2".into()),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(48.86)),
        stop_lon: Some(Longitude(2.35)),
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
        route_id: "R1".into(),
        agency_id: None,
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
    feed.trips.push(Trip {
        route_id: "R1".into(),
        service_id: "SVC".into(),
        trip_id: "T1".into(),
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

fn engine() -> ValidationEngine {
    ValidationEngine::new(Arc::new(Config::default()))
}

fn run(
    rt: &GtfsRtFeed,
    schedule: Option<&GtfsFeed>,
) -> Vec<gapline_core::validation::ValidationError> {
    engine().validate_rt(rt, schedule, NOW).errors().to_vec()
}

fn realtime_only(
    errs: Vec<gapline_core::validation::ValidationError>,
) -> Vec<gapline_core::validation::ValidationError> {
    errs.into_iter().filter(|e| e.section == "12").collect()
}

fn rule_ids(errs: &[gapline_core::validation::ValidationError]) -> Vec<&str> {
    errs.iter().map(|e| e.rule_id.as_str()).collect()
}

#[test]
fn rt_valid_with_schedule_no_errors() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: trip_descriptor(Some("T1"), Some("R1")),
                stop_time_update: vec![stu(
                    Some("S1"),
                    Some(ev(Some(NOW_I + 60), Some(30))),
                    Some(ev(Some(NOW_I + 90), Some(30))),
                )],
                ..Default::default()
            },
        )],
    };
    let rt = rt_feed(&msg);
    let schedule = schedule_paris();
    let errs: Vec<_> = realtime_only(run(&rt, Some(&schedule)))
        .into_iter()
        .filter(|e| e.severity == gapline_core::validation::Severity::Error)
        .collect();
    assert!(
        errs.is_empty(),
        "expected 0 section-12 errors, got: {:?}",
        rule_ids(&errs)
    );
}

#[test]
fn missing_header_version() {
    let msg = FeedMessage {
        header: header("", Some(NOW)),
        entity: vec![],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    assert!(errs.iter().any(|e| e.rule_id == "missing_header"));
}

#[test]
fn unsupported_version() {
    let msg = FeedMessage {
        header: header("3.0", Some(NOW)),
        entity: vec![],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    let v: Vec<_> = errs
        .iter()
        .filter(|e| e.rule_id == "unsupported_version")
        .collect();
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].severity, gapline_core::validation::Severity::Warning);
}

#[test]
fn version_1_0_is_supported() {
    let msg = FeedMessage {
        header: header("1.0", Some(NOW)),
        entity: vec![],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    assert!(
        !errs.iter().any(|e| e.rule_id == "unsupported_version"),
        "version 1.0 must not trigger unsupported_version: {errs:?}"
    );
}

#[test]
fn version_2_0_is_supported() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    assert!(
        !errs.iter().any(|e| e.rule_id == "unsupported_version"),
        "version 2.0 must not trigger unsupported_version: {errs:?}"
    );
}

#[test]
fn missing_timestamp() {
    let msg = FeedMessage {
        header: header("2.0", None),
        entity: vec![],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    assert!(
        errs.iter()
            .any(|e| e.rule_id == "missing_or_zero_timestamp")
    );
}

#[test]
fn zero_timestamp() {
    let msg = FeedMessage {
        header: header("2.0", Some(0)),
        entity: vec![],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    assert!(
        errs.iter()
            .any(|e| e.rule_id == "missing_or_zero_timestamp")
    );
}

#[test]
fn future_timestamp() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW + 7200)),
        entity: vec![],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    assert!(errs.iter().any(|e| e.rule_id == "future_timestamp"));
}

#[test]
fn trip_id_orphan_in_trip_update() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: trip_descriptor(Some("T999"), None),
                ..Default::default()
            },
        )],
    };
    let schedule = schedule_paris();
    let errs = realtime_only(run(&rt_feed(&msg), Some(&schedule)));
    let hits: Vec<_> = errs
        .iter()
        .filter(|e| e.rule_id == "rt_trip_not_in_schedule")
        .collect();
    assert_eq!(hits.len(), 1);
    assert!(hits[0].message.contains("E1"));
}

#[test]
fn route_id_orphan() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: trip_descriptor(Some("T1"), Some("R999")),
                ..Default::default()
            },
        )],
    };
    let schedule = schedule_paris();
    let errs = realtime_only(run(&rt_feed(&msg), Some(&schedule)));
    assert!(errs.iter().any(|e| e.rule_id == "rt_route_not_in_schedule"));
}

#[test]
fn stop_id_orphan_in_trip_update() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: trip_descriptor(Some("T1"), None),
                stop_time_update: vec![stu(Some("S999"), None, None)],
                ..Default::default()
            },
        )],
    };
    let schedule = schedule_paris();
    let errs = realtime_only(run(&rt_feed(&msg), Some(&schedule)));
    assert!(errs.iter().any(|e| e.rule_id == "rt_stop_not_in_schedule"));
}

#[test]
fn trip_id_orphan_in_vehicle_position() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_vehicle(
            "E1",
            VehiclePosition {
                trip: Some(trip_descriptor(Some("T999"), None)),
                position: Some(Position {
                    latitude: 48.85,
                    longitude: 2.34,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )],
    };
    let schedule = schedule_paris();
    let errs = realtime_only(run(&rt_feed(&msg), Some(&schedule)));
    let hits: Vec<_> = errs
        .iter()
        .filter(|e| e.rule_id == "rt_trip_not_in_schedule")
        .collect();
    assert_eq!(hits.len(), 1);
    assert!(hits[0].message.contains("VehiclePosition"));
}

#[test]
fn stop_id_orphan_in_vehicle_position() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_vehicle(
            "E1",
            VehiclePosition {
                trip: Some(trip_descriptor(Some("T1"), None)),
                stop_id: Some("S999".into()),
                position: Some(Position {
                    latitude: 48.85,
                    longitude: 2.34,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )],
    };
    let schedule = schedule_paris();
    let errs = realtime_only(run(&rt_feed(&msg), Some(&schedule)));
    assert!(errs.iter().any(|e| e.rule_id == "rt_stop_not_in_schedule"));
}

#[test]
fn position_outside_bounds() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_vehicle(
            "E1",
            VehiclePosition {
                trip: Some(trip_descriptor(Some("T1"), None)),
                position: Some(Position {
                    latitude: 0.0,
                    longitude: 0.0,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )],
    };
    let schedule = schedule_paris();
    let errs = realtime_only(run(&rt_feed(&msg), Some(&schedule)));
    assert!(
        errs.iter()
            .any(|e| e.rule_id == "position_outside_feed_bounds")
    );
}

#[test]
fn unordered_stop_times() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: trip_descriptor(Some("T1"), None),
                stop_time_update: vec![
                    stu(Some("S1"), None, Some(ev(Some(NOW_I + 200), None))),
                    stu(Some("S2"), Some(ev(Some(NOW_I + 100), None)), None),
                ],
                ..Default::default()
            },
        )],
    };
    let schedule = schedule_paris();
    let errs = realtime_only(run(&rt_feed(&msg), Some(&schedule)));
    assert!(errs.iter().any(|e| e.rule_id == "unordered_stop_times"));
}

#[test]
fn excessive_delay() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: trip_descriptor(Some("T1"), None),
                stop_time_update: vec![stu(Some("S1"), Some(ev(None, Some(7200))), None)],
                ..Default::default()
            },
        )],
    };
    let schedule = schedule_paris();
    let errs = realtime_only(run(&rt_feed(&msg), Some(&schedule)));
    let hits: Vec<_> = errs
        .iter()
        .filter(|e| e.rule_id == "excessive_delay")
        .collect();
    assert_eq!(hits.len(), 1);
}

#[test]
fn alert_without_target() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_alert("E1", Alert::default())],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    assert!(errs.iter().any(|e| e.rule_id == "alert_without_target"));
}

#[test]
fn alert_target_orphan() {
    let alert = Alert {
        informed_entity: vec![EntitySelector {
            stop_id: Some("S999".into()),
            ..Default::default()
        }],
        ..Default::default()
    };
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_alert("E1", alert)],
    };
    let schedule = schedule_paris();
    let errs = realtime_only(run(&rt_feed(&msg), Some(&schedule)));
    assert!(
        errs.iter()
            .any(|e| e.rule_id == "alert_target_not_in_schedule")
    );
}

#[test]
fn duplicate_entity_id() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![empty_entity("E1"), empty_entity("E1")],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    let hits: Vec<_> = errs
        .iter()
        .filter(|e| e.rule_id == "duplicate_entity_id")
        .collect();
    assert_eq!(hits.len(), 1);
    assert!(hits[0].message.contains("E1"));
}

#[test]
fn error_has_section_and_rule_id_and_entity_context() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E42",
            TripUpdate {
                trip: trip_descriptor(Some("T999"), None),
                ..Default::default()
            },
        )],
    };
    let schedule = schedule_paris();
    let errs = realtime_only(run(&rt_feed(&msg), Some(&schedule)));
    let hit = errs
        .iter()
        .find(|e| e.rule_id == "rt_trip_not_in_schedule")
        .expect("trip-not-in-schedule expected");
    assert_eq!(hit.section, "12");
    assert!(hit.message.contains("E42"));
    assert!(hit.message.contains("T999"));
}

#[test]
fn rt_only_skips_cross_validation() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: trip_descriptor(Some("T999"), Some("R999")),
                stop_time_update: vec![stu(Some("S999"), None, None)],
                ..Default::default()
            },
        )],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    let cross_rules = [
        "rt_trip_not_in_schedule",
        "rt_route_not_in_schedule",
        "rt_stop_not_in_schedule",
        "position_outside_feed_bounds",
        "alert_target_not_in_schedule",
        "missing_stop_sequence_for_repeated_stop",
        "rt_stop_wrong_location_type",
        "start_time_mismatch_first_arrival",
    ];
    for rule in cross_rules {
        assert!(
            !errs.iter().any(|e| e.rule_id == rule),
            "rule `{rule}` should not fire without Schedule"
        );
    }
}

// ── New rules — coverage tests ──────────────────────────────────────────

fn st(trip_id: &str, seq: u32, stop_id: &str, arr: GtfsTime, dep: GtfsTime) -> StopTime {
    StopTime {
        trip_id: trip_id.into(),
        arrival_time: Some(arr),
        departure_time: Some(dep),
        stop_id: StopId::from(stop_id),
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

/// Schedule with `stop_times` for trip `T1` (`S1` @08:00, `S2` @08:05).
fn schedule_with_stop_times() -> GtfsFeed {
    let mut feed = schedule_paris();
    feed.stop_times.push(st(
        "T1",
        1,
        "S1",
        GtfsTime::from_hms(8, 0, 0),
        GtfsTime::from_hms(8, 0, 0),
    ));
    feed.stop_times.push(st(
        "T1",
        2,
        "S2",
        GtfsTime::from_hms(8, 5, 0),
        GtfsTime::from_hms(8, 5, 0),
    ));
    feed
}

#[test]
fn stop_time_sequence_unsorted_fires() {
    let mut a = stu(Some("S1"), None, None);
    a.stop_sequence = Some(2);
    let mut b = stu(Some("S2"), None, None);
    b.stop_sequence = Some(1);
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: trip_descriptor(Some("T1"), Some("R1")),
                stop_time_update: vec![a, b],
                ..Default::default()
            },
        )],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    assert!(
        errs.iter()
            .any(|e| e.rule_id == "stop_time_sequence_unsorted")
    );
}

#[test]
fn rt_trip_not_in_schedule_skips_added() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: TripDescriptor {
                    trip_id: Some("UNKNOWN".into()),
                    route_id: Some("R1".into()),
                    schedule_relationship: Some(
                        gapline_core::models::rt::trip_descriptor::ScheduleRelationship::Added
                            as i32,
                    ),
                    ..Default::default()
                },
                stop_time_update: vec![],
                ..Default::default()
            },
        )],
    };
    let schedule = schedule_paris();
    let errs = realtime_only(run(&rt_feed(&msg), Some(&schedule)));
    assert!(
        !errs.iter().any(|e| e.rule_id == "rt_trip_not_in_schedule"),
        "ADDED trip must not fire rt_trip_not_in_schedule: {:?}",
        rule_ids(&errs)
    );
}

#[test]
fn missing_stop_sequence_for_repeated_stop_fires() {
    let mut feed = schedule_with_stop_times();
    feed.stop_times.push(st(
        "T1",
        3,
        "S1",
        GtfsTime::from_hms(8, 10, 0),
        GtfsTime::from_hms(8, 10, 0),
    ));
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: trip_descriptor(Some("T1"), Some("R1")),
                stop_time_update: vec![stu(Some("S1"), None, None)],
                ..Default::default()
            },
        )],
    };
    let errs = realtime_only(run(&rt_feed(&msg), Some(&feed)));
    assert!(
        errs.iter()
            .any(|e| e.rule_id == "missing_stop_sequence_for_repeated_stop")
    );
}

#[test]
fn rt_stop_wrong_location_type_fires() {
    let mut feed = schedule_with_stop_times();
    feed.stops[0].location_type = Some(LocationType::Station);
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: trip_descriptor(Some("T1"), Some("R1")),
                stop_time_update: vec![stu(Some("S1"), None, None)],
                ..Default::default()
            },
        )],
    };
    let errs = realtime_only(run(&rt_feed(&msg), Some(&feed)));
    assert!(
        errs.iter()
            .any(|e| e.rule_id == "rt_stop_wrong_location_type")
    );
}

#[test]
fn stop_time_update_times_not_increasing_fires() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: trip_descriptor(Some("T1"), Some("R1")),
                stop_time_update: vec![
                    stu(Some("S1"), Some(ev(Some(NOW_I + 100), None)), None),
                    stu(Some("S2"), Some(ev(Some(NOW_I + 100), None)), None),
                ],
                ..Default::default()
            },
        )],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    assert!(
        errs.iter()
            .any(|e| e.rule_id == "stop_time_update_times_not_increasing")
    );
}

#[test]
fn start_time_mismatch_first_arrival_fires() {
    let feed = schedule_with_stop_times();
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: TripDescriptor {
                    trip_id: Some("T1".into()),
                    route_id: Some("R1".into()),
                    start_time: Some("09:00:00".into()),
                    ..Default::default()
                },
                stop_time_update: vec![],
                ..Default::default()
            },
        )],
    };
    let errs = realtime_only(run(&rt_feed(&msg), Some(&feed)));
    assert!(
        errs.iter()
            .any(|e| e.rule_id == "start_time_mismatch_first_arrival")
    );
}

#[test]
fn start_time_match_first_arrival_passes() {
    let feed = schedule_with_stop_times();
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: TripDescriptor {
                    trip_id: Some("T1".into()),
                    route_id: Some("R1".into()),
                    start_time: Some("08:00:00".into()),
                    ..Default::default()
                },
                stop_time_update: vec![],
                ..Default::default()
            },
        )],
    };
    let errs = realtime_only(run(&rt_feed(&msg), Some(&feed)));
    assert!(
        !errs
            .iter()
            .any(|e| e.rule_id == "start_time_mismatch_first_arrival")
    );
}

#[test]
fn consecutive_same_stop_id_fires() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: trip_descriptor(Some("T1"), Some("R1")),
                stop_time_update: vec![stu(Some("S1"), None, None), stu(Some("S1"), None, None)],
                ..Default::default()
            },
        )],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    assert!(errs.iter().any(|e| e.rule_id == "consecutive_same_stop_id"));
}

#[test]
fn missing_vehicle_id_fires() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: trip_descriptor(Some("T1"), Some("R1")),
                stop_time_update: vec![],
                ..Default::default()
            },
        )],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    assert!(errs.iter().any(|e| e.rule_id == "missing_vehicle_id"));
}

#[test]
fn feed_not_fresh_fires() {
    let stale_ts = NOW - 600; // 10 minutes old
    let msg = FeedMessage {
        header: header("2.0", Some(stale_ts)),
        entity: vec![],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    assert!(errs.iter().any(|e| e.rule_id == "feed_not_fresh"));
}

#[test]
fn feed_fresh_does_not_fire() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    assert!(!errs.iter().any(|e| e.rule_id == "feed_not_fresh"));
}

#[test]
fn missing_schedule_relationship_fires() {
    let msg = FeedMessage {
        header: header("2.0", Some(NOW)),
        entity: vec![entity_with_trip_update(
            "E1",
            TripUpdate {
                trip: trip_descriptor(Some("T1"), Some("R1")),
                stop_time_update: vec![stu(Some("S1"), None, None)],
                ..Default::default()
            },
        )],
    };
    let errs = realtime_only(run(&rt_feed(&msg), None));
    assert!(
        errs.iter()
            .any(|e| e.rule_id == "missing_schedule_relationship")
    );
}
