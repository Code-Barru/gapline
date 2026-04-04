//! Schedule time validation (section 7).
//!
//! Validates temporal consistency of `stop_times.txt` (7.1) and
//! `frequencies.txt` (7.2).

pub mod frequencies;
pub mod stop_times;

use crate::validation::engine::ValidationEngine;

pub fn register_rules(engine: &mut ValidationEngine, max_trip_duration_hours: Option<u32>) {
    engine.register_rule(Box::new(stop_times::StopTimesTimeSequenceRule::new(
        max_trip_duration_hours,
    )));
    engine.register_rule(Box::new(frequencies::FrequenciesCoherenceRule));
}
