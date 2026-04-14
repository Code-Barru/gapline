//! FK rule: `stop_times.trip_id` → `trips.trip_id`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "stop_times.txt";
use super::{RULE_ID, SECTION};

/// Every `trip_id` in `stop_times.txt` must exist in trips.txt.
pub struct StopTimesTripFkRule;

impl ValidationRule for StopTimesTripFkRule {
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

        feed.stop_times
            .iter()
            .enumerate()
            .filter(|(_, st)| !valid_ids.contains(st.trip_id.as_ref()))
            .map(|(i, st)| {
                let line = i + 2;
                ValidationError::new(RULE_ID, SECTION, Severity::Error)
                    .message(format!(
                        "trip_id '{}' in stop_times.txt line {} references non-existent trip in trips.txt",
                        st.trip_id, line
                    ))
                    .file(FILE)
                    .line(line)
                    .field("trip_id")
                    .value(st.trip_id.as_ref())
            })
            .collect()
    }
}
