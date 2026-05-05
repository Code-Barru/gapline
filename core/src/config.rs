//! Application configuration.
//!
//! Mirrors the hierarchical TOML structure described in Architecture
//! Decision 5: six top-level sections (`[default]`, `[validation]`,
//! `[performance]`, `[output]`, `[batch]`, `[experimental]`), each
//! independently `Default`. The struct derives [`serde::Deserialize`]
//! and is loaded by [`Config::load`] / [`Config::load_from`] from a
//! priority chain: defaults → global → local → CLI overrides.
//!
//! ## Naming
//!
//! Threshold field names keep their unit suffix (`_m`, `_kmh`, `_days`,
//! `_hours`) for clarity at every call site. The TOML schema therefore
//! diverges from the unit-less names in the architecture document - this
//! is intentional and accepted.

use std::path::{Path, PathBuf};

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

/// Default values for CLI arguments that can be omitted from the command line.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DefaultSection {
    /// Default feed path used when `--feed` is omitted.
    pub feed: Option<PathBuf>,
    /// Default output format. Validated by the CLI layer (kept as `String`
    /// here because [`OutputFormat`](../../cli/src/cli/parser.rs) lives in
    /// the `gapline` crate, not in `gapline-core`).
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
    /// Stop validation at the first error. Forward-compat - not yet wired.
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

/// Numeric thresholds grouped by domain - mirrors `[validation.thresholds.*]`.
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

/// Per-route-type maximum speeds in km/h. Covers all ten GTFS basic route
/// types; extended types (`Hvt`, `Unknown`) fall back to `default_kmh`.
///
/// Field naming follows [`crate::models::RouteType`] - note that
/// `route_type` 6 is `aerial_lift_kmh` in this codebase, not `gondola_kmh`
/// as in the architecture spec.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SpeedLimits {
    pub tram_kmh: f64,
    pub subway_kmh: f64,
    pub rail_kmh: f64,
    pub bus_kmh: f64,
    pub ferry_kmh: f64,
    pub cable_tram_kmh: f64,
    pub aerial_lift_kmh: f64,
    pub funicular_kmh: f64,
    pub trolleybus_kmh: f64,
    pub monorail_kmh: f64,
    /// Fallback used for any route type without a dedicated entry
    /// (high-value transport types and unknowns).
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
            cable_tram_kmh: 30.0,
            aerial_lift_kmh: 50.0,
            funicular_kmh: 50.0,
            trolleybus_kmh: 150.0,
            monorail_kmh: 150.0,
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
    /// clock at validation time. Skipped by serde - kept programmatic so
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

/// Performance-tuning knobs. Forward-compat - none of these are wired yet.
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
/// flat `Config::quiet` field - the inversion is documented at every call site.
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

/// `.gl` batch-runner behaviour. Forward-compat - not wired.
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

/// Feature flags for post-MVP additions. Forward-compat - not wired.
#[allow(clippy::struct_excessive_bools)] // one bool per spec'd experimental flag
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ExperimentalSection {
    pub enabled: bool,
    pub validate_flex: bool,
    pub validate_fares_v2: bool,
    pub validate_geojson: bool,
}

// ============================================================================
// CLI overrides
// ============================================================================

/// Values supplied on the command line that take priority over any TOML
/// file. Each `Option`-typed field uses `None` to mean "not provided on
/// the CLI" - only `Some(_)` values overwrite the loaded config.
///
/// `disabled_rules` is **appended** to whatever the file already contained,
/// not replaced - additive rule blacklisting from the CLI is the more
/// useful semantics in practice.
#[derive(Debug, Default)]
pub struct CliOverrides {
    /// Optional `--config PATH` override. When set, this path replaces
    /// `./gapline.toml` in the lookup chain (the global config is still
    /// consulted as the lower-priority layer).
    pub config_path: Option<PathBuf>,
    pub feed: Option<PathBuf>,
    pub format: Option<String>,
    pub output: Option<PathBuf>,
    pub no_color: Option<bool>,
    pub force_color: Option<bool>,
    pub threads: Option<usize>,
    pub min_severity: Option<Severity>,
    pub disabled_rules: Vec<String>,
}

// ============================================================================
// Loader & errors
// ============================================================================

/// Errors produced by [`Config::load`] / [`Config::load_from`].
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Failed to read a config file (file existed but was unreadable).
    /// Missing files are not an error - they are silently skipped.
    #[error("Cannot read config file {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// TOML parse / deserialization failure. The message is pre-formatted
    /// to include the path and (when available) the line number.
    #[error("Config error in {path}: {message}")]
    Parse { path: PathBuf, message: String },
    /// Config deserialized cleanly but contains semantically invalid values
    /// (e.g. negative speed limit, incoherent thresholds). Caught after
    /// merge so the message can cite the specific field.
    #[error("Invalid config: {0}")]
    Invalid(String),
}

impl Config {
    /// Loads the config using the current process's working directory as
    /// the base for `./gapline.toml`. Convenience wrapper over
    /// [`Config::load_from`].
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] if a config file exists but cannot be read
    /// or contains invalid TOML.
    pub fn load(cli: CliOverrides) -> Result<Self, ConfigError> {
        let cwd = std::env::current_dir().ok();
        Self::load_from(cwd.as_deref(), cli)
    }

    /// Loads the config with an explicit base directory for the local
    /// `gapline.toml` lookup. Used by tests to avoid touching the global
    /// process cwd.
    ///
    /// Hierarchy (lowest to highest priority):
    /// 1. Built-in defaults
    /// 2. Global `~/.config/gapline/config.toml` (if it exists)
    /// 3. Local `<base>/gapline.toml`, or `cli.config_path` if set
    /// 4. CLI overrides applied last
    ///
    /// Missing files at any layer are silently skipped.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Io`] when a file exists but cannot be read,
    /// or [`ConfigError::Parse`] when the TOML is malformed.
    pub fn load_from(base_dir: Option<&Path>, cli: CliOverrides) -> Result<Self, ConfigError> {
        let mut merged = toml::Table::new();

        if let Some(global) = global_config_path()
            && let Some((table, _)) = read_table(&global)?
        {
            merge_tables(&mut merged, table);
        }

        let local_path = cli
            .config_path
            .clone()
            .or_else(|| base_dir.map(|d| d.join("gapline.toml")));
        let local_path_for_errors = local_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("./gapline.toml"));
        if let Some(path) = local_path.as_deref()
            && let Some((table, _)) = read_table(path)?
        {
            merge_tables(&mut merged, table);
        }

        let mut config: Config = Config::deserialize(merged).map_err(|e| ConfigError::Parse {
            path: local_path_for_errors,
            message: format_de_error(&e),
        })?;

        config.apply_cli_overrides(cli);
        config.validate()?;
        Ok(config)
    }

    /// Checks the merged config for semantically impossible values that the
    /// TOML deserializer cannot catch on its own (e.g. negative speed limits,
    /// 0-day calendar coverage, incoherent distance thresholds).
    ///
    /// Called automatically by [`Config::load_from`] after CLI overrides are
    /// applied. Exposed so tests and custom loaders can re-check.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Invalid`] on the first violated invariant.
    pub fn validate(&self) -> Result<(), ConfigError> {
        let t = &self.validation.thresholds;

        let speed_checks: [(&str, f64); 11] = [
            ("tram_kmh", t.speed_limits.tram_kmh),
            ("subway_kmh", t.speed_limits.subway_kmh),
            ("rail_kmh", t.speed_limits.rail_kmh),
            ("bus_kmh", t.speed_limits.bus_kmh),
            ("ferry_kmh", t.speed_limits.ferry_kmh),
            ("cable_tram_kmh", t.speed_limits.cable_tram_kmh),
            ("aerial_lift_kmh", t.speed_limits.aerial_lift_kmh),
            ("funicular_kmh", t.speed_limits.funicular_kmh),
            ("trolleybus_kmh", t.speed_limits.trolleybus_kmh),
            ("monorail_kmh", t.speed_limits.monorail_kmh),
            ("default_kmh", t.speed_limits.default_kmh),
        ];
        for (name, v) in speed_checks {
            if !v.is_finite() || v <= 0.0 {
                return Err(ConfigError::Invalid(format!(
                    "[validation.thresholds.speed_limits] {name} must be > 0 (got {v})"
                )));
            }
        }

        let dist_checks: [(&str, f64); 5] = [
            (
                "max_stop_to_shape_distance_m",
                t.distances.max_stop_to_shape_distance_m,
            ),
            (
                "min_shape_point_distance_m",
                t.distances.min_shape_point_distance_m,
            ),
            (
                "shape_dist_incoherence_ratio",
                t.distances.shape_dist_incoherence_ratio,
            ),
            (
                "max_transfer_distance_m",
                t.distances.max_transfer_distance_m,
            ),
            (
                "transfer_distance_warning_m",
                t.distances.transfer_distance_warning_m,
            ),
        ];
        for (name, v) in dist_checks {
            if !v.is_finite() || v < 0.0 {
                return Err(ConfigError::Invalid(format!(
                    "[validation.thresholds.distances] {name} must be ≥ 0 (got {v})"
                )));
            }
        }
        if t.distances.shape_dist_incoherence_ratio > 1.0 {
            return Err(ConfigError::Invalid(
                "[validation.thresholds.distances] shape_dist_incoherence_ratio must be ≤ 1.0"
                    .into(),
            ));
        }
        if t.distances.transfer_distance_warning_m > t.distances.max_transfer_distance_m {
            return Err(ConfigError::Invalid(
                "[validation.thresholds.distances] transfer_distance_warning_m must be ≤ \
                 max_transfer_distance_m"
                    .into(),
            ));
        }

        if let Some(h) = t.time.max_trip_duration_hours
            && h == 0
        {
            return Err(ConfigError::Invalid(
                "[validation.thresholds.time] max_trip_duration_hours must be > 0 when set".into(),
            ));
        }

        if t.coordinates.min_distance_from_origin_m < 0.0
            || t.coordinates.min_distance_from_poles_m < 0.0
        {
            return Err(ConfigError::Invalid(
                "[validation.thresholds.coordinates] distances must be ≥ 0".into(),
            ));
        }

        if t.calendar.min_feed_coverage_days == 0 {
            return Err(ConfigError::Invalid(
                "[validation.thresholds.calendar] min_feed_coverage_days must be > 0".into(),
            ));
        }

        if t.naming.max_route_short_name_length == 0 {
            return Err(ConfigError::Invalid(
                "[validation.thresholds.naming] max_route_short_name_length must be > 0".into(),
            ));
        }

        Ok(())
    }

    /// Applies CLI overrides on top of an already-loaded config.
    fn apply_cli_overrides(&mut self, cli: CliOverrides) {
        if let Some(v) = cli.feed {
            self.default.feed = Some(v);
        }
        if let Some(v) = cli.format {
            self.default.format = Some(v);
        }
        if let Some(v) = cli.output {
            self.default.output = Some(v);
        }
        if let Some(v) = cli.no_color {
            self.output.no_color = v;
        }
        if let Some(v) = cli.force_color {
            self.output.force_color = v;
        }
        if let Some(v) = cli.threads {
            self.performance.num_threads = Some(v);
        }
        if let Some(v) = cli.min_severity {
            self.validation.min_severity = Some(v);
        }
        // Append rather than replace: CLI extends the file blacklist.
        self.validation.disabled_rules.extend(cli.disabled_rules);
    }
}

/// Returns `~/.config/gapline/config.toml` (or the OS equivalent), if a
/// home/config directory can be determined for the current user.
fn global_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("gapline").join("config.toml"))
}

/// Reads a TOML file and returns its parsed root table along with the
/// raw text (kept for future error-context use). Returns `Ok(None)` when
/// the file does not exist - that is the silently-ignored case.
fn read_table(path: &Path) -> Result<Option<(toml::Table, String)>, ConfigError> {
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    let table = text
        .parse::<toml::Table>()
        .map_err(|e| ConfigError::Parse {
            path: path.to_path_buf(),
            message: format_parse_error(&e),
        })?;
    Ok(Some((table, text)))
}

/// Recursively merges `from` into `into`. When both sides hold a sub-table
/// at the same key, the merge descends; otherwise the value from `from`
/// (the higher-priority layer) wins.
fn merge_tables(into: &mut toml::Table, from: toml::Table) {
    for (key, value) in from {
        match (into.remove(&key), value) {
            (Some(toml::Value::Table(mut existing)), toml::Value::Table(incoming)) => {
                merge_tables(&mut existing, incoming);
                into.insert(key, toml::Value::Table(existing));
            }
            (_, value) => {
                into.insert(key, value);
            }
        }
    }
}

/// Pretty-prints a `toml::de::Error` from the parsing stage. Includes the
/// line/column when the error span is known.
fn format_parse_error(err: &toml::de::Error) -> String {
    if let Some(span) = err.span() {
        format!("{} at byte {}", err.message(), span.start)
    } else {
        err.message().to_string()
    }
}

/// Pretty-prints a `toml::de::Error` from the deserialization stage.
/// Includes the field path / line when serde and toml expose them.
fn format_de_error(err: &toml::de::Error) -> String {
    err.to_string()
}
