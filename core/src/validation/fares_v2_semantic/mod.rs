//! Section 10 - Fares v2 semantic rules.

use crate::validation::engine::ValidationEngine;

pub mod rules;

pub fn register_rules(engine: &mut ValidationEngine) {
    engine.register_rule(Box::new(rules::NegativeAmountRule));
    engine.register_rule(Box::new(rules::ZeroAmountRule));
    engine.register_rule(Box::new(rules::TimeframeOverlapRule));
    engine.register_rule(Box::new(rules::InvalidTransferCountRule));
    engine.register_rule(Box::new(rules::ZeroDurationLimitRule));
    engine.register_rule(Box::new(rules::CircularTransferRule));
    engine.register_rule(Box::new(rules::UnusedFareProductRule));
    engine.register_rule(Box::new(rules::EmptyAreaRule));
}
