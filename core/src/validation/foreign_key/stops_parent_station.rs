//! FK rule: `stops.parent_station` → `stops.stop_id` (self-reference).

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "stops.txt";
use super::{RULE_ID, SECTION};

/// If `parent_station` is non-empty, it must reference an existing `stop_id`.
/// Parent type correctness is validated separately in section 7.
pub struct StopsParentStationFkRule;

impl ValidationRule for StopsParentStationFkRule {
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
        let stop_ids: HashSet<&str> = feed.stops.iter().map(|s| s.stop_id.as_ref()).collect();

        let mut errors = Vec::new();

        for (i, stop) in feed.stops.iter().enumerate() {
            let Some(parent_id) = &stop.parent_station else {
                continue;
            };
            let line = i + 2;

            if !stop_ids.contains(parent_id.as_ref()) {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message(format!(
                            "parent_station '{parent_id}' in stops.txt line {line} references non-existent stop in stops.txt"
                        ))
                        .file(FILE)
                        .line(line)
                        .field("parent_station")
                        .value(parent_id.as_ref()),
                );
            }
        }

        errors
    }
}
