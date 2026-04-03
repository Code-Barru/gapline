//! FK rule: `trips.route_id` → `routes.route_id`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "trips.txt";
const SECTION: &str = "5";
const RULE_ID: &str = "foreign_key_violation";

/// Every `route_id` in trips.txt must exist in routes.txt.
pub struct TripsRouteFkRule;

impl ValidationRule for TripsRouteFkRule {
    fn rule_id(&self) -> &'static str {
        RULE_ID
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let valid_ids: HashSet<&str> = feed.routes.iter().map(|r| r.route_id.as_ref()).collect();

        feed.trips
            .iter()
            .enumerate()
            .filter(|(_, t)| !valid_ids.contains(t.route_id.as_ref()))
            .map(|(i, t)| {
                let line = i + 2;
                ValidationError::new(RULE_ID, SECTION, Severity::Error)
                    .message(format!(
                        "route_id '{}' in trips.txt line {} references non-existent route in routes.txt",
                        t.route_id, line
                    ))
                    .file(FILE)
                    .line(line)
                    .field("route_id")
                    .value(t.route_id.as_ref())
            })
            .collect()
    }
}
