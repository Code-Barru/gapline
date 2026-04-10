//! Sanity tests for the [`headway_core::config::Config`] TOML schema.
//!
//! `#[allow(clippy::float_cmp)]` — these tests compare against the *exact*
//! literal defaults declared in `core/src/config.rs`. There is no rounding
//! source between the literal and the deserialized value, so a strict `==`
//! is the assertion we actually want.
#![allow(clippy::float_cmp)]

//!
//! Step 1 of the configuration ticket only restructures the `Config` struct
//! into the nested layout from Architecture Decision 5; the actual loader
//! (`Config::load()`) lands later. These tests verify that the struct is
//! "TOML-ready" — defaults, partial overrides, and `deny_unknown_fields` —
//! so step 2 can plug in the loader without redoing schema work.

use headway_core::config::Config;

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
