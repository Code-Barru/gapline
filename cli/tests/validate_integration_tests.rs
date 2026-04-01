//! Integration tests for the `headway validate` pipeline.

use std::io::Write;
use std::process::Command;
use std::sync::Arc;

use headway_core::config::Config;
use headway_core::parser::{FeedLoader, FeedSource};
use headway_core::validation::engine::ValidationEngine;
use headway_core::validation::{Severity, StructuralValidationRule, ValidationError};
use tempfile::NamedTempFile;

fn headway_bin() -> String {
    env!("CARGO_BIN_EXE_headway").to_string()
}

/// Creates a minimal valid GTFS zip with all required files + data rows.
fn create_valid_feed() -> NamedTempFile {
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
    zip.write_all(b"stop_id,stop_name,stop_lat,stop_lon\nST1,Stop One,40.0,-74.0\n")
        .unwrap();

    zip.start_file("stop_times.txt", opts).unwrap();
    zip.write_all(
        b"trip_id,arrival_time,departure_time,stop_id,stop_sequence\nT1,08:00:00,08:00:00,ST1,1\n",
    )
    .unwrap();

    zip.start_file("calendar.txt", opts).unwrap();
    zip.write_all(b"service_id,monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date\nS1,1,1,1,1,1,0,0,20240101,20241231\n").unwrap();

    zip.finish().unwrap();
    tmp
}

/// Creates a zip without agency.txt (missing required file).
fn create_feed_missing_agency() -> NamedTempFile {
    let tmp = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
    let file = std::fs::File::create(tmp.path()).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default();

    zip.start_file("routes.txt", opts).unwrap();
    zip.write_all(b"route_id,agency_id,route_short_name,route_long_name,route_type\n")
        .unwrap();
    zip.start_file("trips.txt", opts).unwrap();
    zip.write_all(b"route_id,service_id,trip_id\n").unwrap();
    zip.start_file("stops.txt", opts).unwrap();
    zip.write_all(b"stop_id,stop_name,stop_lat,stop_lon\n")
        .unwrap();
    zip.start_file("stop_times.txt", opts).unwrap();
    zip.write_all(b"trip_id,arrival_time,departure_time,stop_id,stop_sequence\n")
        .unwrap();
    zip.start_file("calendar.txt", opts).unwrap();
    zip.write_all(b"service_id,monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date\n").unwrap();

    zip.finish().unwrap();
    tmp
}

#[test]
fn engine_new_registers_sections_1_and_2() {
    let engine = ValidationEngine::new(Arc::new(Config::default()));
    let grouped = engine.group_rules_by_section();
    assert!(grouped.contains_key("1"), "Section 1 rules must be present");
    assert!(grouped.contains_key("2"), "Section 2 rules must be present");
    assert_eq!(
        grouped.len(),
        2,
        "Only sections 1 and 2 for pre-parsing rules"
    );
}

#[test]
fn engine_register_pre_rule_adds_custom_rule() {
    struct MockRule;
    impl StructuralValidationRule for MockRule {
        fn rule_id(&self) -> &'static str {
            "mock_rule"
        }
        fn section(&self) -> &'static str {
            "99"
        }
        fn severity(&self) -> Severity {
            Severity::Warning
        }
        fn validate(&self, _source: &FeedSource) -> Vec<ValidationError> {
            vec![ValidationError::new("mock_rule", "99", Severity::Warning).message("mock warning")]
        }
    }

    let mut engine = ValidationEngine::new(Arc::new(Config::default()));
    engine.register_pre_rule(Box::new(MockRule));

    let grouped = engine.group_rules_by_section();
    assert!(
        grouped.contains_key("99"),
        "Custom section must appear after register_pre_rule"
    );

    let feed = create_valid_feed();
    let source = FeedLoader::open(feed.path()).unwrap();
    let report = engine.validate_structural(&source);
    let has_mock = report.errors().iter().any(|e| e.rule_id == "mock_rule");
    assert!(has_mock, "Mock rule findings must appear in the report");
}

#[test]
fn engine_validate_structural_returns_report() {
    let engine = ValidationEngine::new(Arc::new(Config::default()));
    let feed = create_valid_feed();
    let source = FeedLoader::open(feed.path()).unwrap();
    let report = engine.validate_structural(&source);
    assert!(
        !report.has_errors(),
        "Minimal valid feed should produce 0 errors"
    );
}

#[test]
fn cli_validate_valid_feed_exit_0() {
    let feed = create_valid_feed();
    let output = Command::new(headway_bin())
        .args(["validate", "-f", feed.path().to_str().unwrap()])
        .output()
        .expect("failed to run headway");

    assert!(
        output.status.success(),
        "Exit code should be 0 for a valid feed.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("PASS"),
        "Output should indicate PASS for valid feed"
    );
}

#[test]
fn cli_validate_missing_required_file_exit_1() {
    let feed = create_feed_missing_agency();
    let output = Command::new(headway_bin())
        .args(["validate", "-f", feed.path().to_str().unwrap()])
        .output()
        .expect("failed to run headway");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Exit code should be 1 when required files are missing"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("missing_required_file"),
        "Report should contain missing_required_file error"
    );
}

#[test]
fn cli_validate_warnings_only_exit_0() {
    let feed = create_valid_feed();
    let output = Command::new(headway_bin())
        .args(["validate", "-f", feed.path().to_str().unwrap()])
        .output()
        .expect("failed to run headway");

    assert!(output.status.success(), "Warnings-only feed should exit 0");
}

#[test]
fn cli_validate_json_format() {
    let feed = create_valid_feed();
    let output = Command::new(headway_bin())
        .args([
            "validate",
            "-f",
            feed.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("failed to run headway");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("JSON output should be valid JSON");
    assert!(
        parsed.get("errors").is_some(),
        "JSON must have errors array"
    );
    assert!(
        parsed.get("summary").is_some(),
        "JSON must have summary object"
    );
}

#[test]
fn cli_validate_output_to_file() {
    let feed = create_valid_feed();
    let tmp = NamedTempFile::new().unwrap();
    let output = Command::new(headway_bin())
        .args([
            "validate",
            "-f",
            feed.path().to_str().unwrap(),
            "--format",
            "json",
            "-o",
            tmp.path().to_str().unwrap(),
        ])
        .output()
        .expect("failed to run headway");

    assert!(output.status.success());
    let content = std::fs::read_to_string(tmp.path()).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("File content should be valid JSON");
    assert!(parsed.get("summary").is_some());
}

#[test]
fn cli_validate_nonexistent_feed_exit_1() {
    let output = Command::new(headway_bin())
        .args(["validate", "-f", "/tmp/nonexistent_feed_xyz.zip"])
        .output()
        .expect("failed to run headway");

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn cli_validate_corrupted_zip_exit_1() {
    let tmp = NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"this is not a zip file").unwrap();

    let output = Command::new(headway_bin())
        .args(["validate", "-f", tmp.path().to_str().unwrap()])
        .output()
        .expect("failed to run headway");

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn engine_uses_arc_config() {
    let config = Arc::new(Config::default());
    let config_clone = Arc::clone(&config);
    let _engine = ValidationEngine::new(config);
    assert_eq!(config_clone.max_rows, None);
}
