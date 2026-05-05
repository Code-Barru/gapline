//! Section 9 - GTFS-Flex semantic rules.

use std::sync::Arc;

use crate::validation::engine::ValidationEngine;
use crate::validation::schedule_time_validation::service_dates::ServiceDateCache;

pub mod prior_days_coverage;
pub mod rules;

pub fn register_rules(engine: &mut ValidationEngine, service_cache: Arc<ServiceDateCache>) {
    engine.register_rule(Box::new(rules::WindowOrderRule));
    engine.register_rule(Box::new(rules::PriorNoticeMinPositiveRule));
    engine.register_rule(Box::new(rules::PriorNoticeMinMaxRule));
    engine.register_rule(Box::new(rules::PriorNoticeLastDayTimeRule));
    engine.register_rule(Box::new(rules::EmptyLocationGroupRule));
    engine.register_rule(Box::new(rules::MeanDurationFactorPositiveRule));
    engine.register_rule(Box::new(rules::SafeDurationFactorRule));
    engine.register_rule(Box::new(rules::ScheduledWithBookingRuleRule));
    engine.register_rule(Box::new(
        prior_days_coverage::PriorDaysServiceCoverageRule::new(service_cache),
    ));
}
