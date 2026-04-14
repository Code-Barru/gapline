//! FK rule: `frequencies.trip_id` → `trips.trip_id`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "frequencies.txt";
use super::{RULE_ID, SECTION};

/// Every `trip_id` in frequencies.txt must exist in trips.txt.
pub struct FrequenciesTripFkRule;

impl ValidationRule for FrequenciesTripFkRule {
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

        feed.frequencies
            .iter()
            .enumerate()
            .filter(|(_, f)| !valid_ids.contains(f.trip_id.as_ref()))
            .map(|(i, f)| {
                let line = i + 2;
                ValidationError::new(RULE_ID, SECTION, Severity::Error)
                    .message(format!(
                        "trip_id '{}' in frequencies.txt line {} references non-existent trip in trips.txt",
                        f.trip_id, line
                    ))
                    .file(FILE)
                    .line(line)
                    .field("trip_id")
                    .value(f.trip_id.as_ref())
            })
            .collect()
    }
}
