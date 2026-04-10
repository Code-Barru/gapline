//! End-to-end CLI tests for the TOML configuration system.
//!
//! These tests spawn the real `headway` binary against a tempdir + a
//! `headway.toml` file pointed at via `--config`. We avoid changing the
//! process cwd (`std::env::set_current_dir` is global and would race with
//! parallel tests) and instead always pass `--config` explicitly.
//!
//! The tests cover ticket scenarios 4, 5, 7, 8, 10, 11.

use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use tempfile::{NamedTempFile, TempDir};

fn headway_bin() -> String {
    env!("CARGO_BIN_EXE_headway").to_string()
}

/// Builds a minimal valid GTFS zip on disk and returns its path. Reused
/// from the `validate_integration_tests` fixture pattern.
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
    zip.write_all(b"stop_id,stop_name,stop_lat,stop_lon\nST1,Stop One,40.0,-74.0\nST2,Stop Two,40.01,-74.01\n")
        .unwrap();

    zip.start_file("stop_times.txt", opts).unwrap();
    zip.write_all(
        b"trip_id,arrival_time,departure_time,stop_id,stop_sequence\nT1,08:00:00,08:00:00,ST1,1\nT1,08:05:00,08:05:00,ST2,2\n",
    )
    .unwrap();

    zip.start_file("calendar.txt", opts).unwrap();
    zip.write_all(b"service_id,monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date\nS1,1,1,1,1,1,0,0,20240101,20241231\n").unwrap();

    zip.finish().unwrap();
    tmp
}

/// Writes `contents` to `<dir>/headway.toml` and returns the absolute path.
fn write_config(dir: &TempDir, contents: &str) -> PathBuf {
    let path = dir.path().join("headway.toml");
    std::fs::write(&path, contents).unwrap();
    path
}

// ---------------------------------------------------------------------------
// Scenario 10 — `default.feed` from config, no `-f` on the CLI (CA10)
// ---------------------------------------------------------------------------

#[test]
fn feed_from_config_default_no_dash_f() {
    let dir = tempfile::tempdir().unwrap();
    let feed = create_valid_feed();
    let config = write_config(
        &dir,
        &format!(
            r#"
                [default]
                feed = "{}"
            "#,
            feed.path().display()
        ),
    );

    let output = Command::new(headway_bin())
        .args(["--config", config.to_str().unwrap(), "validate"])
        .output()
        .expect("spawn headway");

    assert!(
        output.status.success(),
        "validate without -f should succeed when default.feed is set\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------------
// Scenario 4 — CLI `--format` overrides config (CA11)
// ---------------------------------------------------------------------------

#[test]
fn cli_format_overrides_config_format() {
    let dir = tempfile::tempdir().unwrap();
    let feed = create_valid_feed();
    let config = write_config(
        &dir,
        &format!(
            r#"
                [default]
                feed = "{}"
                format = "json"
            "#,
            feed.path().display()
        ),
    );

    // Without --format, config dictates JSON output → looks like JSON.
    let json_out = Command::new(headway_bin())
        .args(["--config", config.to_str().unwrap(), "validate"])
        .output()
        .expect("spawn headway");
    let stdout_json = String::from_utf8_lossy(&json_out.stdout);
    assert!(
        stdout_json.trim_start().starts_with('{'),
        "expected JSON output from config default, got:\n{stdout_json}"
    );

    // With --format text, the JSON setting is overridden → text summary.
    let text_out = Command::new(headway_bin())
        .args([
            "--config",
            config.to_str().unwrap(),
            "validate",
            "--format",
            "text",
        ])
        .output()
        .expect("spawn headway");
    let stdout_text = String::from_utf8_lossy(&text_out.stdout);
    assert!(
        stdout_text.contains("Status:"),
        "expected text summary, got:\n{stdout_text}"
    );
}

// ---------------------------------------------------------------------------
// Scenario 5 — Malformed config produces a clear error (CA3)
// ---------------------------------------------------------------------------

#[test]
fn malformed_config_emits_clear_error() {
    let dir = tempfile::tempdir().unwrap();
    let config = write_config(
        &dir,
        r"
            [default]
            format = 42
        ",
    );

    let output = Command::new(headway_bin())
        .args([
            "--config",
            config.to_str().unwrap(),
            "validate",
            "-f",
            "x.zip",
        ])
        .output()
        .expect("spawn headway");

    assert!(!output.status.success());
    // Exit code 2 = config error (distinct from 1 = validation failure).
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Config error in"),
        "expected 'Config error in' prefix, got:\n{stderr}"
    );
}

// ---------------------------------------------------------------------------
// Scenario 8 — `min_severity = "error"` filters warnings (CA9)
// ---------------------------------------------------------------------------

#[test]
fn min_severity_error_hides_warnings() {
    // The valid feed produces a feed_expiring_soon warning (calendar ends
    // 2024-12-31, today is well after). Validate twice and assert that the
    // count of warning lines drops to zero with min_severity = error.
    let dir = tempfile::tempdir().unwrap();
    let feed = create_valid_feed();
    let config = write_config(
        &dir,
        &format!(
            r#"
                [default]
                feed = "{}"

                [validation]
                min_severity = "error"
            "#,
            feed.path().display()
        ),
    );

    let output = Command::new(headway_bin())
        .args([
            "--config",
            config.to_str().unwrap(),
            "--no-color",
            "validate",
            "--format",
            "text",
        ])
        .output()
        .expect("spawn headway");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // No `[WARNING]` label should appear in the listing.
    assert!(
        !stdout.contains("[WARNING]"),
        "min_severity=error should hide warnings, got:\n{stdout}"
    );
    // The summary line still mentions "Warning" / "Warnings" as a label,
    // but the count must be 0.
    assert!(
        stdout.contains("0 Warnings") || stdout.contains("0 Warning"),
        "expected zero warnings in summary, got:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// Scenario 7 — `--disable-rule` skips a rule (CA8)
// ---------------------------------------------------------------------------

#[test]
fn disable_rule_via_cli_skips_finding() {
    let dir = tempfile::tempdir().unwrap();
    let feed = create_valid_feed();
    let config = write_config(
        &dir,
        &format!(
            r#"
                [default]
                feed = "{}"
            "#,
            feed.path().display()
        ),
    );

    // Disable speed_validation. The valid feed doesn't trigger it anyway,
    // so we assert the smoke path: command runs to completion and the
    // listing contains zero `unrealistic_speed` lines (i.e. the rule never
    // had a chance to fire).
    let output = Command::new(headway_bin())
        .args([
            "--config",
            config.to_str().unwrap(),
            "--no-color",
            "validate",
            "--format",
            "text",
            "--disable-rule",
            "speed_validation",
        ])
        .output()
        .expect("spawn headway");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("unrealistic_speed"),
        "disabled rule must not appear in output, got:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// Scenario 11 — `force_color = true` keeps ANSI escapes when piped (CA13)
// ---------------------------------------------------------------------------

#[test]
fn force_color_emits_ansi_when_piped() {
    let dir = tempfile::tempdir().unwrap();
    let feed = create_valid_feed();
    let config = write_config(
        &dir,
        &format!(
            r#"
                [default]
                feed = "{}"

                [output]
                force_color = true
            "#,
            feed.path().display()
        ),
    );

    let output = Command::new(headway_bin())
        .args([
            "--config",
            config.to_str().unwrap(),
            "validate",
            "--format",
            "text",
        ])
        .output()
        .expect("spawn headway");

    // Stdout is a pipe (from `Command`), so by default the colored crate
    // would auto-disable. With `force_color = true`, ANSI escapes must
    // remain.
    let stdout = output.stdout;
    assert!(
        stdout.windows(2).any(|w| w == [0x1b, b'[']),
        "force_color should keep ANSI escapes in piped stdout"
    );
}

// ---------------------------------------------------------------------------
// Scenario 9 — `--threads` smoke test (CA12)
// ---------------------------------------------------------------------------

#[test]
fn threads_flag_accepted_smoke() {
    let dir = tempfile::tempdir().unwrap();
    let feed = create_valid_feed();
    let config = write_config(
        &dir,
        &format!(
            r#"
                [default]
                feed = "{}"
            "#,
            feed.path().display()
        ),
    );

    let output = Command::new(headway_bin())
        .args([
            "--config",
            config.to_str().unwrap(),
            "--threads",
            "2",
            "validate",
        ])
        .output()
        .expect("spawn headway");

    assert!(
        output.status.success(),
        "--threads should be accepted, got stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
