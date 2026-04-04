//! FK rule: `fare_rules.fare_id` → `fare_attributes.fare_id`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "fare_rules.txt";
const SECTION: &str = "5";
const RULE_ID: &str = "foreign_key_violation";

/// `fare_id` in `fare_rules.txt` must exist in `fare_attributes.txt`.
pub struct FareRulesFareFkRule;

impl ValidationRule for FareRulesFareFkRule {
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
        let valid_ids: HashSet<&str> = feed
            .fare_attributes
            .iter()
            .map(|fa| fa.fare_id.as_ref())
            .collect();

        feed.fare_rules
            .iter()
            .enumerate()
            .filter(|(_, fr)| !valid_ids.contains(fr.fare_id.as_ref()))
            .map(|(i, fr)| {
                let line = i + 2;
                ValidationError::new(RULE_ID, SECTION, Severity::Error)
                    .message(format!(
                        "fare_id '{}' in fare_rules.txt line {line} references non-existent fare in fare_attributes.txt",
                        fr.fare_id
                    ))
                    .file(FILE)
                    .line(line)
                    .field("fare_id")
                    .value(fr.fare_id.as_ref())
            })
            .collect()
    }
}
