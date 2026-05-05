//! Sections 4 and 9 — Field Definition Validation.
//!
//! Validates conditional field requirements in the parsed `GtfsFeed`:
//! fields that are required only when certain conditions are met
//! (e.g. `agency_id` required when multiple agencies exist), and
//! mutual-exclusion / at-least-one constraints.
//!
//! **Note:** Fields that are unconditionally required and already enforced by
//! the parser (via `required_id`, `required_parse`, etc.) are *not*
//! re-validated here — only rules requiring cross-record or conditional logic
//! live in this module. The following files are entirely covered by the parser
//! and have no rule in this module: `calendar.txt`, `calendar_dates.txt`,
//! `shapes.txt`, `frequencies.txt`, `levels.txt`, `fare_attributes.txt`,
//! `fare_rules.txt`, `location_groups.txt`, `location_group_stops.txt`.

pub mod agency;
pub mod attributions;
pub mod booking_rules;
pub mod fares_v2;
pub mod feed_info;
pub mod pathways;
pub mod routes;
pub mod stop_times;
pub mod stop_times_flex;
pub mod stops;
pub mod transfers;
pub mod translations;
pub mod trips;

use crate::validation::engine::ValidationEngine;

/// Registers all field-definition rules (sections 4 and 9) with the engine.
pub fn register_rules(engine: &mut ValidationEngine) {
    engine.register_rule(Box::new(agency::AgencyFieldDefinitionRule));
    engine.register_rule(Box::new(stops::StopsFieldDefinitionRule));
    engine.register_rule(Box::new(routes::RoutesFieldDefinitionRule));
    engine.register_rule(Box::new(trips::TripsFieldDefinitionRule));
    engine.register_rule(Box::new(stop_times::StopTimesFieldDefinitionRule));
    engine.register_rule(Box::new(transfers::TransfersFieldDefinitionRule));
    engine.register_rule(Box::new(pathways::PathwaysFieldDefinitionRule));
    engine.register_rule(Box::new(feed_info::FeedInfoFieldDefinitionRule));
    engine.register_rule(Box::new(translations::TranslationsFieldDefinitionRule));
    engine.register_rule(Box::new(attributions::AttributionsFieldDefinitionRule));
    engine.register_rule(Box::new(booking_rules::BookingRulesFieldDefinitionRule));
    engine.register_rule(Box::new(stop_times_flex::StopTimesFlexFieldDefinitionRule));
    engine.register_rule(Box::new(fares_v2::FaresV2FieldDefinitionRule));
}
