//! FK rule: `fare_rules.origin_id` / `destination_id` / `contains_id` → `stops.zone_id`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "fare_rules.txt";
const SECTION: &str = "5";
const RULE_ID: &str = "foreign_key_violation";

/// If `origin_id`, `destination_id`, or `contains_id` is non-empty in `fare_rules.txt`,
/// each must match an existing `zone_id` in `stops.txt`.
pub struct FareRulesZonesFkRule;

impl ValidationRule for FareRulesZonesFkRule {
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
        let valid_zones: HashSet<&str> = feed
            .stops
            .iter()
            .filter_map(|s| s.zone_id.as_deref())
            .collect();

        let mut errors = Vec::new();

        for (i, fr) in feed.fare_rules.iter().enumerate() {
            let line = i + 2;

            for (field, value) in [
                ("origin_id", &fr.origin_id),
                ("destination_id", &fr.destination_id),
                ("contains_id", &fr.contains_id),
            ] {
                if let Some(zone) = value.as_deref()
                    && !valid_zones.contains(zone)
                {
                    errors.push(
                        ValidationError::new(RULE_ID, SECTION, Severity::Error)
                            .message(format!(
                                "{field} '{zone}' in fare_rules.txt line {line} references non-existent zone_id in stops.txt"
                            ))
                            .file(FILE)
                            .line(line)
                            .field(field)
                            .value(zone),
                    );
                }
            }
        }

        errors
    }
}
