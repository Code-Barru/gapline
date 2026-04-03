//! FK rule: `fare_rules.route_id` → `routes.route_id`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "fare_rules.txt";
const SECTION: &str = "5";
const RULE_ID: &str = "foreign_key_violation";

/// If `route_id` is non-empty in fare_rules.txt, it must exist in routes.txt.
pub struct FareRulesRouteFkRule;

impl ValidationRule for FareRulesRouteFkRule {
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

        feed.fare_rules
            .iter()
            .enumerate()
            .filter_map(|(i, fr)| {
                let id = fr.route_id.as_ref()?;
                if valid_ids.contains(id.as_ref()) {
                    return None;
                }
                let line = i + 2;
                Some(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message(format!(
                            "route_id '{id}' in fare_rules.txt line {line} references non-existent route in routes.txt"
                        ))
                        .file(FILE)
                        .line(line)
                        .field("route_id")
                        .value(id.as_ref()),
                )
            })
            .collect()
    }
}
