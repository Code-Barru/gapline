//! FK rule: `transfers.from_trip_id` → `trips.trip_id`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "transfers.txt";
use super::{RULE_ID, SECTION};

/// If `from_trip_id` is non-empty in transfers.txt, it must exist in trips.txt.
pub struct TransfersFromTripFkRule;

impl ValidationRule for TransfersFromTripFkRule {
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
        let valid_ids: HashSet<&str> = feed.trips.iter().map(|t| t.trip_id.as_ref()).collect();

        feed.transfers
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                let id = t.from_trip_id.as_ref()?;
                if valid_ids.contains(id.as_ref()) {
                    return None;
                }
                let line = i + 2;
                Some(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message(format!(
                            "from_trip_id '{id}' in transfers.txt line {line} references non-existent trip in trips.txt"
                        ))
                        .file(FILE)
                        .line(line)
                        .field("from_trip_id")
                        .value(id.as_ref()),
                )
            })
            .collect()
    }
}
