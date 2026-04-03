//! FK rule: `transfers.to_stop_id` → `stops.stop_id`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "transfers.txt";
const SECTION: &str = "5";
const RULE_ID: &str = "foreign_key_violation";

/// If `to_stop_id` is non-empty in transfers.txt, it must exist in stops.txt.
pub struct TransfersToStopFkRule;

impl ValidationRule for TransfersToStopFkRule {
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
        let valid_ids: HashSet<&str> = feed.stops.iter().map(|s| s.stop_id.as_ref()).collect();

        feed.transfers
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                let id = t.to_stop_id.as_ref()?;
                if valid_ids.contains(id.as_ref()) {
                    return None;
                }
                let line = i + 2;
                Some(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message(format!(
                            "to_stop_id '{id}' in transfers.txt line {line} references non-existent stop in stops.txt"
                        ))
                        .file(FILE)
                        .line(line)
                        .field("to_stop_id")
                        .value(id.as_ref()),
                )
            })
            .collect()
    }
}
