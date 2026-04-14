//! FK rule: `stop_times.stop_id` → `stops.stop_id`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "stop_times.txt";
use super::{RULE_ID, SECTION};

/// Every `stop_id` in `stop_times.txt` must exist in stops.txt.
pub struct StopTimesStopFkRule;

impl ValidationRule for StopTimesStopFkRule {
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

        feed.stop_times
            .iter()
            .enumerate()
            .filter(|(_, st)| !valid_ids.contains(st.stop_id.as_ref()))
            .map(|(i, st)| {
                let line = i + 2;
                ValidationError::new(RULE_ID, SECTION, Severity::Error)
                    .message(format!(
                        "stop_id '{}' in stop_times.txt line {} references non-existent stop in stops.txt",
                        st.stop_id, line
                    ))
                    .file(FILE)
                    .line(line)
                    .field("stop_id")
                    .value(st.stop_id.as_ref())
            })
            .collect()
    }
}
