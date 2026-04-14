//! Tests for the [`headway_core::config::Config`] TOML schema and loader.
//!
//! Covers:
//! - Schema sanity (empty TOML, partial overrides, `deny_unknown_fields`)
//! - Loader hierarchy: defaults → local → CLI overrides
//! - Error reporting for malformed files
//! - CLI override semantics (replace vs append for `disabled_rules`)
//!
//! The global `~/.config/headway/config.toml` is intentionally **not**
//! exercised here — these tests use [`Config::load_from`] with an explicit
//! base directory plus `cli.config_path`, which together pin the local
//! lookup to a tempdir. The user-global file is still consulted by
//! `load_from`, but tests asserting on the merged result must tolerate the
//! possibility that a developer machine has one. We work around this by
//! only asserting on fields the test sets explicitly.
//!
//! `#[allow(clippy::float_cmp)]` — assertions compare against the *exact*
//! literal defaults declared in `core/src/config.rs`. There is no rounding
//! source between the literal and the deserialized value, so a strict `==`
//! is the assertion we want.
#![allow(clippy::float_cmp)]

use std::path::PathBuf;

use headway_core::config::{CliOverrides, Config};
use headway_core::validation::Severity;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn write_local(dir: &TempDir, contents: &str) -> PathBuf {
    let path = dir.path().join("headway.toml");
    std::fs::write(&path, contents).expect("write headway.toml");
    path
}

/// `Config::load_from` with an isolated tempdir base, no CLI overrides
/// (other than `config_path` pinning).
fn load_local(dir: &TempDir) -> Config {
    let local = dir.path().join("headway.toml");
    let cli = CliOverrides {
        config_path: Some(local),
        ..CliOverrides::default()
    };
    Config::load_from(Some(dir.path()), cli).expect("load_from")
}

#[test]
fn empty_toml_yields_full_defaults() {
    let config: Config = toml::from_str("").expect("empty TOML must deserialize");
    let defaults = Config::default();

    // Spot-check one representative field per section to confirm every
    // section was populated by the `#[serde(default)]` cascade.
    assert_eq!(
        config
            .validation
            .thresholds
            .distances
            .max_stop_to_shape_distance_m,
        defaults
            .validation
            .thresholds
            .distances
            .max_stop_to_shape_distance_m
    );
    assert_eq!(
        config.validation.thresholds.speed_limits.bus_kmh,
        defaults.validation.thresholds.speed_limits.bus_kmh
    );
    assert_eq!(
        config.validation.thresholds.calendar.min_feed_coverage_days,
        defaults
            .validation
            .thresholds
            .calendar
            .min_feed_coverage_days
    );
    assert_eq!(
        config.performance.csv_buffer_size,
        defaults.performance.csv_buffer_size
    );
    assert_eq!(config.output.show_progress, defaults.output.show_progress);
    assert_eq!(config.batch.echo_commands, defaults.batch.echo_commands);
    assert!(!config.experimental.enabled);
}

#[test]
fn partial_section_overrides_only_named_fields() {
    let toml = r"
        [validation.thresholds.speed_limits]
        bus_kmh = 200.0
    ";

    let config: Config = toml::from_str(toml).expect("partial TOML must deserialize");

    // Override applied.
    assert_eq!(config.validation.thresholds.speed_limits.bus_kmh, 200.0);
    // Sibling fields keep their defaults.
    assert_eq!(config.validation.thresholds.speed_limits.tram_kmh, 150.0);
    assert_eq!(config.validation.thresholds.speed_limits.rail_kmh, 500.0);
    // Other sections still come from defaults.
    assert_eq!(
        config
            .validation
            .thresholds
            .distances
            .max_stop_to_shape_distance_m,
        100.0
    );
    assert!(config.output.show_progress);
}

#[test]
fn unknown_field_rejected() {
    // `max_stop_to_shape` is the architecture-spec name without the `_m`
    // suffix — a typo against our actual schema. The loader must reject it
    // instead of silently ignoring the line.
    let toml = r"
        [validation.thresholds.distances]
        max_stop_to_shape = 50.0
    ";

    let err = toml::from_str::<Config>(toml).expect_err("unknown field must be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("unknown field") || msg.contains("max_stop_to_shape"),
        "expected unknown-field error, got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// Loader tests — exercise Config::load_from end-to-end
// ---------------------------------------------------------------------------

/// Ticket scenario 1: no config file at all → built-in defaults are used.
#[test]
fn load_no_local_file_returns_defaults() {
    let dir = tempfile::tempdir().unwrap();
    // Note: no headway.toml is written. The global config may or may not
    // exist on the dev machine — assert only on a defaults-only sanity field
    // that no realistic global config would override.
    let cli = CliOverrides {
        config_path: Some(dir.path().join("headway.toml")),
        ..CliOverrides::default()
    };
    let config = Config::load_from(Some(dir.path()), cli).expect("load_from");
    assert!(config.default.feed.is_none());
    // The defaults `Config::default()` exposes for these fields are pinned
    // in the source — if a stray global config sets them, the test would
    // need to be reworked, but in practice none of these are commonly set.
    assert_eq!(
        config
            .validation
            .thresholds
            .distances
            .max_stop_to_shape_distance_m,
        100.0
    );
}

/// Ticket scenario 2: only `./headway.toml` exists, with `default.feed`.
#[test]
fn load_local_only_resolves_feed() {
    let dir = tempfile::tempdir().unwrap();
    write_local(
        &dir,
        r#"
            [default]
            feed = "./data/feed.zip"
        "#,
    );

    let config = load_local(&dir);
    assert_eq!(
        config.default.feed.as_deref(),
        Some(std::path::Path::new("./data/feed.zip"))
    );
    // Other sections must remain at defaults.
    assert_eq!(config.validation.thresholds.speed_limits.bus_kmh, 150.0);
}

/// Ticket scenario 4: CLI flag overrides a config-file value.
#[test]
fn cli_overrides_beat_local_file() {
    let dir = tempfile::tempdir().unwrap();
    write_local(
        &dir,
        r#"
            [default]
            format = "json"
        "#,
    );
    let cli = CliOverrides {
        config_path: Some(dir.path().join("headway.toml")),
        format: Some("text".into()),
        ..CliOverrides::default()
    };
    let config = Config::load_from(Some(dir.path()), cli).expect("load_from");
    assert_eq!(config.default.format.as_deref(), Some("text"));
}

/// Ticket scenario 5: malformed TOML produces a clear error mentioning
/// the path and a position indicator.
#[test]
fn malformed_toml_emits_path_and_position_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_local(
        &dir,
        // `format = 42` is a type mismatch (expected string).
        r"
            [default]
            format = 42
        ",
    );

    let cli = CliOverrides {
        config_path: Some(path.clone()),
        ..CliOverrides::default()
    };
    let err = Config::load_from(Some(dir.path()), cli).expect_err("malformed TOML must error");
    let msg = err.to_string();
    assert!(msg.contains("Config error in"), "got: {msg}");
    // The serde error mentions the offending field name; do a lax check.
    assert!(
        msg.contains("format") || msg.contains("invalid"),
        "got: {msg}"
    );
}

/// Ticket scenario 6: speed-limit override applied through the loader.
#[test]
fn load_speed_limit_override() {
    let dir = tempfile::tempdir().unwrap();
    write_local(
        &dir,
        r"
            [validation.thresholds.speed_limits]
            bus_kmh = 200.0
        ",
    );
    let config = load_local(&dir);
    assert_eq!(config.validation.thresholds.speed_limits.bus_kmh, 200.0);
    // Tram unchanged.
    assert_eq!(config.validation.thresholds.speed_limits.tram_kmh, 150.0);
}

/// Ticket scenario 12: a config file with only one section keeps every
/// other section at defaults.
#[test]
fn load_partial_config_keeps_other_sections_default() {
    let dir = tempfile::tempdir().unwrap();
    write_local(
        &dir,
        r#"
            [default]
            feed = "./feed.zip"
        "#,
    );
    let config = load_local(&dir);
    assert!(config.default.feed.is_some());
    assert!(config.output.show_progress);
    assert!(config.batch.echo_commands);
    assert_eq!(config.performance.csv_buffer_size, 8192);
}

/// `disabled_rules` from the CLI are appended to whatever the file
/// contained, not replacing it. Documented behavior.
#[test]
fn cli_disabled_rules_append_to_file_list() {
    let dir = tempfile::tempdir().unwrap();
    write_local(
        &dir,
        r#"
            [validation]
            disabled_rules = ["from_file"]
        "#,
    );
    let cli = CliOverrides {
        config_path: Some(dir.path().join("headway.toml")),
        disabled_rules: vec!["from_cli".into()],
        ..CliOverrides::default()
    };
    let config = Config::load_from(Some(dir.path()), cli).expect("load_from");
    assert_eq!(config.validation.disabled_rules.len(), 2);
    assert!(
        config
            .validation
            .disabled_rules
            .contains(&"from_file".into())
    );
    assert!(
        config
            .validation
            .disabled_rules
            .contains(&"from_cli".into())
    );
}

/// Ticket scenario 8: `min_severity` from the file is preserved when no
/// CLI override is set.
#[test]
fn min_severity_loaded_from_file() {
    let dir = tempfile::tempdir().unwrap();
    write_local(
        &dir,
        r#"
            [validation]
            min_severity = "error"
        "#,
    );
    let config = load_local(&dir);
    assert_eq!(config.validation.min_severity, Some(Severity::Error));
}

// ---------------------------------------------------------------------------
// Semantic validation (Config::validate)
// ---------------------------------------------------------------------------

fn load_expecting_invalid(dir: &TempDir) -> String {
    let local = dir.path().join("headway.toml");
    let cli = CliOverrides {
        config_path: Some(local),
        ..CliOverrides::default()
    };
    match Config::load_from(Some(dir.path()), cli) {
        Err(headway_core::config::ConfigError::Invalid(msg)) => msg,
        Err(other) => panic!("expected Invalid, got {other:?}"),
        Ok(_) => panic!("expected Invalid, got Ok"),
    }
}

#[test]
fn validate_rejects_negative_speed_limit() {
    let dir = tempfile::tempdir().unwrap();
    write_local(
        &dir,
        "
            [validation.thresholds.speed_limits]
            tram_kmh = -100.0
        ",
    );
    let msg = load_expecting_invalid(&dir);
    assert!(msg.contains("tram_kmh"), "got: {msg}");
}

#[test]
fn validate_rejects_incoherent_transfer_distances() {
    let dir = tempfile::tempdir().unwrap();
    write_local(
        &dir,
        "
            [validation.thresholds.distances]
            max_transfer_distance_m = 500.0
            transfer_distance_warning_m = 1000.0
        ",
    );
    let msg = load_expecting_invalid(&dir);
    assert!(msg.contains("transfer_distance_warning_m"), "got: {msg}");
}

#[test]
fn validate_rejects_zero_coverage_days() {
    let dir = tempfile::tempdir().unwrap();
    write_local(
        &dir,
        "
            [validation.thresholds.calendar]
            min_feed_coverage_days = 0
        ",
    );
    let msg = load_expecting_invalid(&dir);
    assert!(msg.contains("min_feed_coverage_days"), "got: {msg}");
}

#[test]
fn validate_accepts_default_config() {
    let dir = tempfile::tempdir().unwrap();
    write_local(&dir, "");
    // Should not return ConfigError::Invalid.
    let _ = load_local(&dir);
}
