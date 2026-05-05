//! Tests for section 7.10 - pathway validation.

use gapline_core::models::*;
use gapline_core::validation::schedule_time_validation::pathways::PathwayValidationRule;
use gapline_core::validation::{Severity, ValidationRule};

// ---------------------------------------------------------------------------
// Builders
// ---------------------------------------------------------------------------

fn make_pathway(
    id: &str,
    from: &str,
    to: &str,
    bidir: IsBidirectional,
    traversal_time: Option<u32>,
) -> Pathway {
    Pathway {
        pathway_id: PathwayId::from(id),
        from_stop_id: StopId::from(from),
        to_stop_id: StopId::from(to),
        pathway_mode: PathwayMode::Walkway,
        is_bidirectional: bidir,
        length: None,
        traversal_time,
        stair_count: None,
        max_slope: None,
        min_width: None,
        signposted_as: None,
        reversed_signposted_as: None,
    }
}

fn make_stop_with_type(id: &str, loc_type: LocationType, parent: Option<&str>) -> Stop {
    Stop {
        stop_id: StopId::from(id),
        stop_code: None,
        stop_name: None,
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(45.5)),
        stop_lon: Some(Longitude(-73.5)),
        zone_id: None,
        stop_url: None,
        location_type: Some(loc_type),
        parent_station: parent.map(StopId::from),
        stop_timezone: None,
        wheelchair_boarding: None,
        level_id: None,
        platform_code: None,
    }
}

fn rule() -> PathwayValidationRule {
    PathwayValidationRule
}

// ---------------------------------------------------------------------------
// invalid_traversal_time
// ---------------------------------------------------------------------------

#[test]
fn traversal_time_zero_error() {
    let feed = GtfsFeed {
        pathways: vec![make_pathway(
            "PW1",
            "A",
            "B",
            IsBidirectional::Bidirectional,
            Some(0),
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "invalid_traversal_time")
        .collect();
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].severity, Severity::Error);
}

#[test]
fn traversal_time_positive_ok() {
    let feed = GtfsFeed {
        pathways: vec![make_pathway(
            "PW1",
            "A",
            "B",
            IsBidirectional::Bidirectional,
            Some(120),
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "invalid_traversal_time")
        .collect();
    assert!(matched.is_empty());
}

#[test]
fn traversal_time_none_ok() {
    let feed = GtfsFeed {
        pathways: vec![make_pathway(
            "PW1",
            "A",
            "B",
            IsBidirectional::Bidirectional,
            None,
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "invalid_traversal_time")
        .collect();
    assert!(matched.is_empty());
}

// ---------------------------------------------------------------------------
// one_way_pathway_without_return
// ---------------------------------------------------------------------------

#[test]
fn unidirectional_without_return() {
    let feed = GtfsFeed {
        pathways: vec![make_pathway(
            "PW1",
            "A",
            "B",
            IsBidirectional::Unidirectional,
            Some(60),
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "one_way_pathway_without_return")
        .collect();
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].severity, Severity::Warning);
}

#[test]
fn unidirectional_with_return_ok() {
    let feed = GtfsFeed {
        pathways: vec![
            make_pathway("PW1", "A", "B", IsBidirectional::Unidirectional, Some(60)),
            make_pathway("PW2", "B", "A", IsBidirectional::Unidirectional, Some(60)),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "one_way_pathway_without_return")
        .collect();
    assert!(matched.is_empty());
}

#[test]
fn bidirectional_ok() {
    let feed = GtfsFeed {
        pathways: vec![make_pathway(
            "PW1",
            "A",
            "B",
            IsBidirectional::Bidirectional,
            Some(60),
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "one_way_pathway_without_return")
        .collect();
    assert!(matched.is_empty());
}

#[test]
fn bidirectional_covers_unidirectional_return() {
    // A bidirectional pathway B→A should cover A→B unidirectional's return.
    let feed = GtfsFeed {
        pathways: vec![
            make_pathway("PW1", "A", "B", IsBidirectional::Unidirectional, Some(60)),
            make_pathway("PW2", "B", "A", IsBidirectional::Bidirectional, Some(60)),
        ],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "one_way_pathway_without_return")
        .collect();
    assert!(matched.is_empty());
}

// ---------------------------------------------------------------------------
// station_without_entrance_pathway
// ---------------------------------------------------------------------------

#[test]
fn station_without_entrance_pathway() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop_with_type("S1", LocationType::Station, None),
            make_stop_with_type("N1", LocationType::GenericNode, Some("S1")),
            make_stop_with_type("N2", LocationType::GenericNode, Some("S1")),
        ],
        // Pathways exist but connect only generic nodes, not entrances.
        pathways: vec![make_pathway(
            "PW1",
            "N1",
            "N2",
            IsBidirectional::Bidirectional,
            Some(30),
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "station_without_entrance_pathway")
        .collect();
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].severity, Severity::Warning);
    assert_eq!(matched[0].file_name.as_deref(), Some("stops.txt"));
    assert_eq!(matched[0].value.as_deref(), Some("S1"));
}

#[test]
fn station_with_entrance_pathway_ok() {
    let feed = GtfsFeed {
        stops: vec![
            make_stop_with_type("S1", LocationType::Station, None),
            make_stop_with_type("E1", LocationType::EntranceExit, Some("S1")),
            make_stop_with_type("N1", LocationType::GenericNode, Some("S1")),
        ],
        pathways: vec![make_pathway(
            "PW1",
            "E1",
            "N1",
            IsBidirectional::Bidirectional,
            Some(30),
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "station_without_entrance_pathway")
        .collect();
    assert!(matched.is_empty());
}

#[test]
fn no_pathways_skips_ca7() {
    let feed = GtfsFeed {
        stops: vec![make_stop_with_type("S1", LocationType::Station, None)],
        pathways: vec![],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let matched: Vec<_> = errors
        .iter()
        .filter(|e| e.rule_id == "station_without_entrance_pathway")
        .collect();
    assert!(matched.is_empty());
}

// ---------------------------------------------------------------------------
// error context completeness
// ---------------------------------------------------------------------------

#[test]
fn error_context_complete() {
    let feed = GtfsFeed {
        pathways: vec![make_pathway(
            "PW1",
            "A",
            "B",
            IsBidirectional::Bidirectional,
            Some(0),
        )],
        ..Default::default()
    };
    let errors = rule().validate(&feed);
    let err = errors
        .iter()
        .find(|e| e.rule_id == "invalid_traversal_time")
        .expect("expected error");
    assert_eq!(err.section, "7");
    assert_eq!(err.severity, Severity::Error);
    assert_eq!(err.file_name.as_deref(), Some("pathways.txt"));
    assert_eq!(err.line_number, Some(2));
    assert_eq!(err.field_name.as_deref(), Some("traversal_time"));
    assert_eq!(err.value.as_deref(), Some("0"));
    assert!(!err.message.is_empty());
}
