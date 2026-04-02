//! Section 4 — Field Definition Validation.
//!
//! Validates conditional field requirements in the parsed `GtfsFeed`:
//! fields that are required only when certain conditions are met
//! (e.g. `agency_id` required when multiple agencies exist), and
//! mutual-exclusion / at-least-one constraints.
//!
//! **Note:** Fields that are unconditionally required and already enforced by
//! the parser (via `required_id`, `required_parse`, etc.) are *not*
//! re-validated here — only rules requiring cross-record or conditional logic
//! live in this module. `calendar.txt` is entirely covered by the parser and
//! has no section 4 rule.

pub mod agency;
pub mod routes;
pub mod stop_times;
pub mod stops;
pub mod trips;

use crate::validation::engine::ValidationEngine;

/// Registers all section 4 (Field Definition Validation) rules with the engine.
pub fn register_rules(engine: &mut ValidationEngine) {
    engine.register_rule(Box::new(agency::AgencyFieldDefinitionRule));
    engine.register_rule(Box::new(stops::StopsFieldDefinitionRule));
    engine.register_rule(Box::new(routes::RoutesFieldDefinitionRule));
    engine.register_rule(Box::new(trips::TripsFieldDefinitionRule));
    engine.register_rule(Box::new(stop_times::StopTimesFieldDefinitionRule));
}
