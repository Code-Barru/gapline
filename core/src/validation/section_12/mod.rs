//! Section 12 — GTFS-Realtime semantic validation.
//!
//! Header/version sanity, cross-validation against a Schedule feed, and
//! RT-only semantic checks (timestamps order, delays, alerts).

use crate::validation::engine::ValidationEngine;

pub mod rules;

pub fn register_rules(engine: &mut ValidationEngine) {
    engine.register_rt_rule(Box::new(rules::MissingHeaderRule));
    engine.register_rt_rule(Box::new(rules::UnsupportedVersionRule));
    engine.register_rt_rule(Box::new(rules::MissingTimestampRule));
    engine.register_rt_rule(Box::new(rules::FutureTimestampRule));
    engine.register_rt_rule(Box::new(rules::RtTripNotInScheduleRule));
    engine.register_rt_rule(Box::new(rules::RtRouteNotInScheduleRule));
    engine.register_rt_rule(Box::new(rules::RtStopNotInScheduleRule));
    engine.register_rt_rule(Box::new(rules::PositionOutsideFeedBoundsRule));
    engine.register_rt_rule(Box::new(rules::UnorderedStopTimesRule));
    engine.register_rt_rule(Box::new(rules::ExcessiveDelayRule));
    engine.register_rt_rule(Box::new(rules::AlertWithoutTargetRule));
    engine.register_rt_rule(Box::new(rules::AlertTargetNotInScheduleRule));
    engine.register_rt_rule(Box::new(rules::DuplicateEntityIdRule));
}
