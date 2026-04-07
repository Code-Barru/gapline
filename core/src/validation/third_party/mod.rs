//! Section 13 — Third-party validator compatibility checks.
//!
//! These rules replicate checks enforced by widely-used GTFS consumers
//! (Conveyal, Etalab, `OpenTripPlanner`, Google feedvalidator). A feed that
//! passes the GTFS specification but fails these tools is unusable in
//! practice.

pub mod conveyal;
pub mod etalab;
pub mod google;
pub mod otp;

use crate::validation::engine::ValidationEngine;

pub fn register_rules(engine: &mut ValidationEngine) {
    engine.register_rule(Box::new(conveyal::ConveyalTripWithoutShapeRule));
    engine.register_rule(Box::new(etalab::EtalabMissingContactRule));
    engine.register_rule(Box::new(otp::OtpTripTooFewStopsRule));
    engine.register_rule(Box::new(otp::OtpMissingFeedVersionRule));
    engine.register_rule(Box::new(google::GoogleCoordinatesInStopNameRule));
    engine.register_rule(Box::new(google::GoogleIdenticalRouteColorsRule));
}
