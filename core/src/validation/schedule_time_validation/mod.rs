//! Schedule time validation (section 7).
//!
//! Validates temporal consistency of `stop_times.txt` (7.1) and
//! `frequencies.txt` (7.2), plus shape geometry (7.3) and stop-to-shape
//! distances (7.4).

pub mod distances;
pub mod frequencies;
pub mod shapes;
pub mod stop_times;

use crate::validation::engine::ValidationEngine;

/// Distance thresholds used by the section-7 geometric rules.
#[derive(Debug, Clone, Copy)]
pub struct DistanceThresholds {
    pub max_stop_to_shape_distance_m: f64,
    pub min_shape_point_distance_m: f64,
    pub shape_dist_incoherence_ratio: f64,
}

pub fn register_rules(
    engine: &mut ValidationEngine,
    max_trip_duration_hours: Option<u32>,
    distances: DistanceThresholds,
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
}
