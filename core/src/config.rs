//! Application configuration.
//!
//! Provides a minimal configuration struct that will be extended as features
//! are added (TOML loading, per-rule overrides, etc.).

/// Global configuration for headway.
///
/// Currently holds only the optional row-count limit used by
/// [`TooManyRowsRule`](crate::validation::file_structure::TooManyRowsRule).
/// Future tickets will add TOML deserialization and a three-tier config system.
#[derive(Debug, Clone)]
pub struct Config {
    /// Maximum allowed data rows per file. `None` disables the check.
    pub max_rows: Option<usize>,
    /// Suppress all progress bars and terminal output. Defaults to `false`.
    ///
    /// Set to `true` for benchmarks, tests, or non-interactive contexts.
    pub quiet: bool,
    /// Maximum allowed trip duration in hours. Trips exceeding this threshold
    /// produce a `trip_too_long` warning. Defaults to `Some(24)`.
    pub max_trip_duration_hours: Option<u32>,
    /// Maximum allowed distance in meters between a stop and the nearest point
    /// of its shape. Trips exceeding this produce a `stop_too_far_from_shape`
    /// warning. Defaults to `100.0`.
    pub max_stop_to_shape_distance_m: f64,
    /// Minimum expected distance in meters between two consecutive shape
    /// points. Points closer than this produce a `shape_points_too_close`
    /// warning. Defaults to `1.11`.
    pub min_shape_point_distance_m: f64,
    /// Tolerance ratio for `shape_dist_traveled` coherence. A point-to-point
    /// declared increment is flagged when it diverges from the expected
    /// Haversine increment (scaled by the shape's global declared/Haversine
    /// ratio) by more than this relative amount. Defaults to `0.5` (50 %).
    pub shape_dist_incoherence_ratio: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_rows: None,
            quiet: false,
            max_trip_duration_hours: Some(24),
            max_stop_to_shape_distance_m: 100.0,
            min_shape_point_distance_m: 1.11,
            shape_dist_incoherence_ratio: 0.5,
        }
    }
}
