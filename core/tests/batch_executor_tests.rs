use std::sync::Arc;

use gapline_core::Dataset;
use gapline_core::batch::{BatchCommand, BatchCommandResult, BatchExecutor};
use gapline_core::config::Config;
use gapline_core::crud::query::parse;
use gapline_core::crud::read::GtfsTarget;
use gapline_core::models::*;

fn sets(pairs: &[&str]) -> Vec<String> {
    pairs.iter().map(|&p| p.to_string()).collect()
}

fn make_stop(id: &str, name: &str) -> Stop {
    Stop {
        stop_id: StopId::from(id),
        stop_code: None,
        stop_name: Some(name.to_string()),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(0.0)),
        stop_lon: Some(Longitude(0.0)),
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

fn feed_with_stops() -> GtfsFeed {
    let mut feed = GtfsFeed::default();
    feed.stops.push(make_stop("S1", "Stop One"));
    feed.stops.push(make_stop("S2", "Stop Two"));
    feed
}

fn executor_from_feed(feed: GtfsFeed) -> BatchExecutor {
    BatchExecutor::new(Dataset::from_feed(feed), vec![])
}

fn stop_name_for(result: &gapline_core::crud::read::ReadResult, stop_id: &str) -> Option<String> {
    let id_idx = result.headers.iter().position(|&h| h == "stop_id")?;
    let name_idx = result.headers.iter().position(|&h| h == "stop_name")?;
    result
        .rows
        .iter()
        .find(|row| row.get(id_idx).and_then(|v| v.as_deref()) == Some(stop_id))?
        .get(name_idx)
        .and_then(Clone::clone)
}

#[test]
fn sequential_read_then_update() {
    let mut exec = executor_from_feed(feed_with_stops());

    let r = exec
        .execute_one(&BatchCommand::Read {
            target: GtfsTarget::Stops,
            query: None,
        })
        .unwrap();
    assert!(matches!(r, BatchCommandResult::Read(_)));

    let r = exec
        .execute_one(&BatchCommand::Update {
            target: GtfsTarget::Stops,
            query: parse("stop_id=S1").unwrap(),
            assignments: sets(&["stop_name=Renamed"]),
            cascade: false,
        })
        .unwrap();
    assert!(matches!(r, BatchCommandResult::Updated(_)));

    let result = exec.dataset().read(GtfsTarget::Stops, None).unwrap();
    assert_eq!(stop_name_for(&result, "S1").as_deref(), Some("Renamed"));
}

#[test]
fn sequential_create_increases_count() {
    let mut exec = executor_from_feed(feed_with_stops());
    let initial = exec.dataset().feed().stops.len();

    exec.execute_one(&BatchCommand::Create {
        target: GtfsTarget::Stops,
        assignments: sets(&[
            "stop_id=S3",
            "stop_name=New Stop",
            "stop_lat=1.0",
            "stop_lon=1.0",
        ]),
    })
    .unwrap();

    assert_eq!(exec.dataset().feed().stops.len(), initial + 1);
}

#[test]
fn sequential_delete_decreases_count() {
    let mut exec = executor_from_feed(feed_with_stops());
    let initial = exec.dataset().feed().stops.len();

    exec.execute_one(&BatchCommand::Delete {
        target: GtfsTarget::Stops,
        query: parse("stop_id=S1").unwrap(),
    })
    .unwrap();

    assert_eq!(exec.dataset().feed().stops.len(), initial - 1);
    assert!(
        !exec
            .dataset()
            .feed()
            .stops
            .iter()
            .any(|s| s.stop_id == StopId::from("S1"))
    );
}

#[test]
fn run_stops_on_first_error_reports_index() {
    let mut exec = executor_from_feed(feed_with_stops());

    let commands = vec![
        // cmd 0: valid
        BatchCommand::Update {
            target: GtfsTarget::Stops,
            query: parse("stop_id=S1").unwrap(),
            assignments: sets(&["stop_name=AfterCmd0"]),
            cascade: false,
        },
        // cmd 1: unknown field → error
        BatchCommand::Update {
            target: GtfsTarget::Stops,
            query: parse("stop_id=S1").unwrap(),
            assignments: sets(&["nonexistent_field=X"]),
            cascade: false,
        },
        // cmd 2: must never run
        BatchCommand::Update {
            target: GtfsTarget::Stops,
            query: parse("stop_id=S1").unwrap(),
            assignments: sets(&["stop_name=ShouldNeverRun"]),
            cascade: false,
        },
    ];

    let (idx, _err) = exec.run(&commands).unwrap_err();
    assert_eq!(idx, 1);

    let result = exec.dataset().read(GtfsTarget::Stops, None).unwrap();
    assert_eq!(
        stop_name_for(&result, "S1").as_deref(),
        Some("AfterCmd0"),
        "cmd 2 must not have run"
    );
}

#[test]
fn update_no_match_returns_no_changes() {
    let mut exec = executor_from_feed(feed_with_stops());

    let r = exec
        .execute_one(&BatchCommand::Update {
            target: GtfsTarget::Stops,
            query: parse("stop_id=NONEXISTENT").unwrap(),
            assignments: sets(&["stop_name=X"]),
            cascade: false,
        })
        .unwrap();

    assert!(matches!(r, BatchCommandResult::NoChanges));
}

#[test]
fn delete_no_match_returns_no_changes() {
    let mut exec = executor_from_feed(feed_with_stops());

    let r = exec
        .execute_one(&BatchCommand::Delete {
            target: GtfsTarget::Stops,
            query: parse("stop_id=NONEXISTENT").unwrap(),
        })
        .unwrap();

    assert!(matches!(r, BatchCommandResult::NoChanges));
}

#[test]
fn validate_returns_report() {
    let mut exec = executor_from_feed(feed_with_stops());

    let r = exec
        .execute_one(&BatchCommand::Validate {
            config: Arc::new(Config::default()),
        })
        .unwrap();

    assert!(matches!(r, BatchCommandResult::Validated(_)));
}

#[test]
fn modified_targets_tracks_updated_target() {
    let mut exec = executor_from_feed(feed_with_stops());
    assert!(exec.modified_targets().is_empty());

    exec.execute_one(&BatchCommand::Update {
        target: GtfsTarget::Stops,
        query: parse("stop_id=S1").unwrap(),
        assignments: sets(&["stop_name=X"]),
        cascade: false,
    })
    .unwrap();

    assert!(exec.modified_targets().contains(&GtfsTarget::Stops));
}

#[test]
fn run_returns_results_for_all_successful_commands() {
    let mut exec = executor_from_feed(feed_with_stops());

    let commands = vec![
        BatchCommand::Update {
            target: GtfsTarget::Stops,
            query: parse("stop_id=S1").unwrap(),
            assignments: sets(&["stop_name=A"]),
            cascade: false,
        },
        BatchCommand::Update {
            target: GtfsTarget::Stops,
            query: parse("stop_id=S2").unwrap(),
            assignments: sets(&["stop_name=B"]),
            cascade: false,
        },
    ];

    let results = exec.run(&commands).unwrap();
    assert_eq!(results.len(), 2);
    assert!(
        results
            .iter()
            .all(|(_, r)| matches!(r, BatchCommandResult::Updated(_)))
    );
}
