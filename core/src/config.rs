//! Application configuration.
//!
//! Mirrors the hierarchical TOML structure described in Architecture
//! Decision 5: six top-level sections (`[default]`, `[validation]`,
//! `[performance]`, `[output]`, `[batch]`, `[experimental]`), each
//! independently `Default`. The struct derives [`serde::Deserialize`]
//! so a future `Config::load()` can populate it from TOML — no loader
//! exists yet; the rest of the codebase still constructs `Config::default()`.
//!
//! ## Naming
//!
//! Threshold field names keep their unit suffix (`_m`, `_kmh`, `_days`,
//! `_hours`) for clarity at every call site. The TOML schema therefore
//! diverges from the unit-less names in the architecture document — this
//! is intentional and accepted.

use std::path::PathBuf;

use serde::Deserialize;

use crate::models::GtfsDate;
use crate::validation::Severity;

/// Top-level configuration. See module docs for the section layout.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub default: DefaultSection,
    pub validation: ValidationSection,
    pub performance: PerformanceSection,
    pub output: OutputSection,
    pub batch: BatchSection,
    pub experimental: ExperimentalSection,
}

// ============================================================================
// [default]
// ============================================================================

/// Default values for CLI arguments that can be omitted from the command line.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DefaultSection {
    /// Default feed path used when `--feed` is omitted.
    pub feed: Option<PathBuf>,
    /// Default output format. Validated by the CLI layer (kept as `String`
    /// here because [`OutputFormat`](../../cli/src/cli/parser.rs) lives in
    /// the `headway` crate, not in `headway-core`).
    pub format: Option<String>,
    /// Default output destination when `--output` is omitted.
    pub output: Option<PathBuf>,
}

// ============================================================================
// [validation]
// ============================================================================

/// Validation behaviour, rule selection, and numeric thresholds.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ValidationSection {
    /// Stop validation at the first error. Forward-compat — not yet wired.
    pub fail_fast: bool,
    /// Cap on the number of findings emitted per rule. Forward-compat.
    pub max_errors_per_rule: Option<usize>,
    /// Minimum severity to keep in the rendered report. Forward-compat.
    pub min_severity: Option<Severity>,
    /// Rule IDs to skip when registering. Forward-compat.
    pub disabled_rules: Vec<String>,
    /// If non-empty, only these rule IDs run. Forward-compat.
    pub enabled_rules: Vec<String>,
    /// Maximum allowed data rows per file. `None` disables the check.
    /// Consumed by [`TooManyRowsRule`](crate::validation::file_structure::TooManyRowsRule).
    /// Not exposed in the architecture spec but kept here so the existing
    /// rule keeps working without resurrecting a free-floating constant.
    pub max_rows: Option<usize>,
    pub thresholds: Thresholds,
}

/// Numeric thresholds grouped by domain — mirrors `[validation.thresholds.*]`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Thresholds {
    pub speed_limits: SpeedLimits,
    pub distances: Distances,
    pub time: Time,
    pub coordinates: Coordinates,
    pub calendar: Calendar,
    pub naming: Naming,
}

/// Per-route-type maximum speeds in km/h. Five route types are wired today;
/// step 8 of the broader plan extends this to all ten GTFS route types.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SpeedLimits {
    pub tram_kmh: f64,
    pub subway_kmh: f64,
    pub rail_kmh: f64,
    pub bus_kmh: f64,
    pub ferry_kmh: f64,
    /// Fallback used for any route type without a dedicated entry.
    pub default_kmh: f64,
}

impl Default for SpeedLimits {
    fn default() -> Self {
        Self {
            tram_kmh: 150.0,
            subway_kmh: 150.0,
            rail_kmh: 500.0,
            bus_kmh: 150.0,
            ferry_kmh: 150.0,
            default_kmh: 150.0,
        }
    }
}

/// Distance thresholds (all metric).
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Distances {
    pub max_stop_to_shape_distance_m: f64,
    pub min_shape_point_distance_m: f64,
    pub shape_dist_incoherence_ratio: f64,
    pub max_transfer_distance_m: f64,
    pub transfer_distance_warning_m: f64,
}

impl Default for Distances {
    fn default() -> Self {
        Self {
            max_stop_to_shape_distance_m: 100.0,
            min_shape_point_distance_m: 1.11,
            shape_dist_incoherence_ratio: 0.5,
            max_transfer_distance_m: 10_000.0,
            transfer_distance_warning_m: 2_000.0,
        }
    }
}

/// Time-domain thresholds.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Time {
    /// Maximum allowed trip duration in hours. `None` disables the check.
    pub max_trip_duration_hours: Option<u32>,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            max_trip_duration_hours: Some(24),
        }
    }
}

/// Geographic-coordinate sanity thresholds.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Coordinates {
    pub min_distance_from_origin_m: f64,
    pub min_distance_from_poles_m: f64,
}

impl Default for Coordinates {
    fn default() -> Self {
        Self {
            min_distance_from_origin_m: 1000.0,
            min_distance_from_poles_m: 1000.0,
        }
    }
}

/// Calendar coverage and expiration thresholds.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Calendar {
    pub min_feed_coverage_days: u32,
    pub feed_expiration_warning_days: i64,
    pub min_trip_activity_days: u32,
    /// Date used as "today" for expiration checks. `None` uses the system
    /// clock at validation time. Skipped by serde — kept programmatic so
    /// tests can pin the clock without having to make `GtfsDate` deserializable.
    #[serde(skip)]
    pub reference_date: Option<GtfsDate>,
}

impl Default for Calendar {
    fn default() -> Self {
        Self {
            min_feed_coverage_days: 30,
            feed_expiration_warning_days: 7,
            min_trip_activity_days: 7,
            reference_date: None,
        }
    }
}

/// String-length thresholds for human-facing identifiers.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Naming {
    pub max_route_short_name_length: usize,
}

impl Default for Naming {
    fn default() -> Self {
        Self {
            max_route_short_name_length: 12,
        }
    }
}

// ============================================================================
// [performance]
// ============================================================================

/// Performance-tuning knobs. Forward-compat — none of these are wired yet.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PerformanceSection {
    /// Worker thread count for the rayon global pool. `None` = auto-detect.
    pub num_threads: Option<usize>,
    /// CSV reader buffer size in bytes.
    pub csv_buffer_size: usize,
    pub parallel_parsing: bool,
    pub parallel_validation: bool,
}

impl Default for PerformanceSection {
    fn default() -> Self {
        Self {
            num_threads: None,
            csv_buffer_size: 8192,
            parallel_parsing: true,
            parallel_validation: true,
        }
    }
}

// ============================================================================
// [output]
// ============================================================================

/// Output formatting and progress display. `show_progress` replaces the old
/// flat `Config::quiet` field — the inversion is documented at every call site.
#[allow(clippy::struct_excessive_bools)] // mirrors the spec's TOML schema 1:1
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OutputSection {
    pub force_color: bool,
    pub no_color: bool,
    /// Whether progress bars/spinners should be drawn. Old `quiet = true`
    /// maps to `show_progress = false`.
    pub show_progress: bool,
    pub verbosity: Verbosity,
    pub timestamp_format: TimestampFormat,
    pub group_by_file: bool,
    pub group_by_rule: bool,
}

impl Default for OutputSection {
    fn default() -> Self {
        Self {
            force_color: false,
            no_color: false,
            show_progress: true,
            verbosity: Verbosity::default(),
            timestamp_format: TimestampFormat::default(),
            group_by_file: true,
            group_by_rule: false,
        }
    }
}

/// Verbosity level for human-facing output.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verbosity {
    Quiet,
    #[default]
    Normal,
    Verbose,
}

/// Timestamp rendering style for log/report headers.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimestampFormat {
    #[default]
    None,
    Iso8601,
    Unix,
    Relative,
}

// ============================================================================
// [batch]
// ============================================================================

/// `.hw` batch-runner behaviour. Forward-compat — not wired.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatchSection {
    pub continue_on_error: bool,
    pub echo_commands: bool,
}

impl Default for BatchSection {
    fn default() -> Self {
        Self {
            continue_on_error: false,
            echo_commands: true,
        }
    }
}

// ============================================================================
// [experimental]
// ============================================================================

/// Feature flags for post-MVP additions. Forward-compat — not wired.
#[allow(clippy::struct_excessive_bools)] // one bool per spec'd experimental flag
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ExperimentalSection {
    pub enabled: bool,
    pub validate_flex: bool,
    pub validate_fares_v2: bool,
    pub validate_geojson: bool,
}
