//! Application configuration.
//!
//! Provides a minimal configuration struct that will be extended as features
//! are added (TOML loading, per-rule overrides, etc.).

use crate::models::GtfsDate;

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
    /// Minimum expected feed coverage in days (max date − min date + 1).
    /// Feeds with shorter ranges produce a `short_feed_coverage` warning.
    /// Defaults to `30`.
    pub min_feed_coverage_days: u32,
    /// Number of days before `feed_end_date` to warn about imminent
    /// expiration. A feed expiring within this window produces a
    /// `feed_expiring_soon` warning. Defaults to `7`.
    pub feed_expiration_warning_days: i64,
    /// Minimum active days a trip's service must have. Trips whose service
    /// yields fewer active days produce a `low_trip_activity` warning.
    /// Defaults to `7`.
    pub min_trip_activity_days: u32,
    /// Date used as "today" for expiration checks. `None` uses the system
    /// clock at validation time. Set this for deterministic tests.
    pub reference_date: Option<GtfsDate>,
    /// Maximum allowed distance in meters between `from_stop_id` and
    /// `to_stop_id` in a transfer. Transfers exceeding this produce a
    /// `transfer_distance_too_large` error. Defaults to `10_000.0`.
    pub max_transfer_distance_m: f64,
    /// Distance threshold in meters for suspicious transfers. Transfers
    /// between this value and `max_transfer_distance_m` produce a
    /// `transfer_distance_suspicious` warning. Defaults to `2_000.0`.
    pub transfer_distance_warning_m: f64,
    /// Maximum speed in km/h for Tram (`route_type=0`). Defaults to `150.0`.
    pub speed_limit_tram_kmh: f64,
    /// Maximum speed in km/h for Subway/Metro (`route_type=1`). Defaults to `150.0`.
    pub speed_limit_subway_kmh: f64,
    /// Maximum speed in km/h for Rail/Train (`route_type=2`). Defaults to `500.0`.
    pub speed_limit_rail_kmh: f64,
    /// Maximum speed in km/h for Bus (`route_type=3`). Defaults to `150.0`.
    pub speed_limit_bus_kmh: f64,
    /// Maximum speed in km/h for Ferry (`route_type=4`). Defaults to `150.0`.
    pub speed_limit_ferry_kmh: f64,
    /// Default maximum speed in km/h for route types without a specific
    /// limit. Defaults to `150.0`.
    pub speed_limit_default_kmh: f64,
    /// Minimum distance in meters from the geographic origin (0, 0) for a
    /// stop to avoid a `coordinates_near_origin` warning. Defaults to `1000.0`.
    pub min_distance_from_origin_m: f64,
    /// Minimum distance in meters from the geographic poles (latitude ±90°)
    /// for a stop to avoid a `coordinates_near_pole` warning. Defaults to
    /// `1000.0`.
    pub min_distance_from_poles_m: f64,
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
            min_feed_coverage_days: 30,
            feed_expiration_warning_days: 7,
            min_trip_activity_days: 7,
            reference_date: None,
            max_transfer_distance_m: 10_000.0,
            transfer_distance_warning_m: 2_000.0,
            speed_limit_tram_kmh: 150.0,
            speed_limit_subway_kmh: 150.0,
            speed_limit_rail_kmh: 500.0,
            speed_limit_bus_kmh: 150.0,
            speed_limit_ferry_kmh: 150.0,
            speed_limit_default_kmh: 150.0,
            min_distance_from_origin_m: 1000.0,
            min_distance_from_poles_m: 1000.0,
        }
    }
}
