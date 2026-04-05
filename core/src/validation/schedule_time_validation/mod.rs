//! Schedule time validation (section 7).
//!
//! Validates temporal consistency of `stop_times.txt` (7.1) and
//! `frequencies.txt` (7.2), shape geometry (7.3), stop-to-shape distances
//! (7.4), calendar date ranges and feed coverage (7.5), and `calendar_dates`
//! coherence plus trip activity (7.6).

pub mod calendar_dates_coherence;
pub mod calendar_ranges;
pub mod distances;
pub mod feed_coverage;
pub mod frequencies;
pub mod shapes;
pub mod stop_times;
pub mod trip_activity;

use crate::models::GtfsDate;
use crate::validation::engine::ValidationEngine;

/// Distance thresholds used by the section-7 geometric rules.
#[derive(Debug, Clone, Copy)]
pub struct DistanceThresholds {
    pub max_stop_to_shape_distance_m: f64,
    pub min_shape_point_distance_m: f64,
    pub shape_dist_incoherence_ratio: f64,
}

/// Calendar-coherence thresholds used by the section-7 calendar rules.
#[derive(Debug, Clone, Copy)]
pub struct CalendarThresholds {
    pub min_feed_coverage_days: u32,
    pub feed_expiration_warning_days: i64,
    pub min_trip_activity_days: u32,
    pub reference_date: Option<GtfsDate>,
}

pub fn register_rules(
    engine: &mut ValidationEngine,
    max_trip_duration_hours: Option<u32>,
    distances: DistanceThresholds,
    calendar: CalendarThresholds,
) {
    engine.register_rule(Box::new(stop_times::StopTimesTimeSequenceRule::new(
        max_trip_duration_hours,
    )));
    engine.register_rule(Box::new(frequencies::FrequenciesCoherenceRule));
    engine.register_rule(Box::new(shapes::ShapesGeometryRule::new(
        distances.min_shape_point_distance_m,
        distances.shape_dist_incoherence_ratio,
    )));
    engine.register_rule(Box::new(distances::StopToShapeDistanceRule::new(
        distances.max_stop_to_shape_distance_m,
    )));
    engine.register_rule(Box::new(calendar_ranges::CalendarRangesRule));
    engine.register_rule(Box::new(feed_coverage::FeedCoverageRule::new(
        calendar.min_feed_coverage_days,
        calendar.feed_expiration_warning_days,
        calendar.reference_date,
    )));
    engine.register_rule(Box::new(
        calendar_dates_coherence::CalendarDatesCoherenceRule,
    ));
    engine.register_rule(Box::new(trip_activity::TripActivityRule::new(
        calendar.min_trip_activity_days,
    )));
}
