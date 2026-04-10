//! End-to-end tests for `headway rules list`.

use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

fn headway_bin() -> String {
    env!("CARGO_BIN_EXE_headway").to_string()
}

fn write_config(dir: &TempDir, contents: &str) -> std::path::PathBuf {
    let path = dir.path().join("headway.toml");
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn rules_list_text_contains_known_rules() {
    let output = Command::new(headway_bin())
        .args(["rules", "list"])
        .output()
        .expect("spawn headway");

    assert!(
        output.status.success(),
        "rules list should exit 0, stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Both stages should be represented.
    assert!(
        stdout.contains("speed_validation"),
        "expected speed_validation in listing, got:\n{stdout}"
    );
    assert!(
        stdout.contains("missing_required_file"),
        "expected missing_required_file in listing, got:\n{stdout}"
    );
    assert!(
        stdout.contains("structural"),
        "expected at least one structural row, got:\n{stdout}"
    );
    assert!(
        stdout.contains("semantic"),
        "expected at least one semantic row, got:\n{stdout}"
    );
    assert!(
        stdout.contains("rules"),
        "expected the '<N> rules' footer, got:\n{stdout}"
    );
}

#[test]
fn rules_list_json_count_matches_array_length() {
    let output = Command::new(headway_bin())
        .args(["rules", "list", "--format", "json"])
        .output()
        .expect("spawn headway");
    assert!(output.status.success());

    let json: Value = serde_json::from_slice(&output.stdout).expect("parse json");
    let rules = json["rules"].as_array().expect("rules is array");
    let count = json["count"].as_u64().expect("count is u64");

    assert_eq!(rules.len() as u64, count);
    // Sanity floor: there should be tens of registered rules.
    assert!(count > 30, "fewer rules registered than expected: {count}");

    // Each entry has the three fields with the right shape.
    for entry in rules {
        assert!(entry["rule_id"].is_string());
        let sev = entry["severity"].as_str().expect("severity is str");
        assert!(matches!(sev, "error" | "warning" | "info"), "got: {sev}");
        let stage = entry["stage"].as_str().expect("stage is str");
        assert!(matches!(stage, "structural" | "semantic"), "got: {stage}");
    }
}

#[test]
fn severity_filter_only_returns_matching() {
    let output = Command::new(headway_bin())
        .args(["rules", "list", "--severity", "error", "--format", "json"])
        .output()
        .expect("spawn headway");
    assert!(output.status.success());

    let json: Value = serde_json::from_slice(&output.stdout).expect("parse json");
    let rules = json["rules"].as_array().expect("rules is array");
    assert!(!rules.is_empty(), "filter must not return empty");
    for entry in rules {
        assert_eq!(
            entry["severity"].as_str(),
            Some("error"),
            "non-error rule leaked through filter: {entry}"
        );
    }
}

/// The listing must NOT respect `[validation] disabled_rules` from the
/// user config — otherwise it can't help discover rule IDs.
#[test]
fn rules_list_ignores_disabled_rules_config() {
    let dir = tempfile::tempdir().unwrap();
    let config = write_config(
        &dir,
        r#"
            [validation]
            disabled_rules = ["speed_validation"]
        "#,
    );

    let output = Command::new(headway_bin())
        .args([
            "--config",
            config.to_str().unwrap(),
            "rules",
            "list",
            "--format",
            "json",
        ])
        .output()
        .expect("spawn headway");
    assert!(output.status.success());

    let json: Value = serde_json::from_slice(&output.stdout).expect("parse json");
    let rules = json["rules"].as_array().expect("rules is array");
    let has_speed = rules
        .iter()
        .any(|e| e["rule_id"].as_str() == Some("speed_validation"));
    assert!(
        has_speed,
        "speed_validation must remain visible despite being in disabled_rules"
    );
}
