//! FK rule: `calendar_dates.service_id` → `calendar.service_id` (WARNING).

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "calendar_dates.txt";
const SECTION: &str = "5";
const RULE_ID: &str = "foreign_key_violation";

/// If calendar.txt exists, a `service_id` in `calendar_dates.txt` that does
/// not appear in calendar.txt produces a **warning** (not an error) because
/// `calendar_dates.txt` can legitimately define services on its own.
pub struct CalendarDatesServiceFkRule;

impl ValidationRule for CalendarDatesServiceFkRule {
    fn rule_id(&self) -> &'static str {
        RULE_ID
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_file("calendar.txt") {
            return Vec::new();
        }

        let calendar_ids: HashSet<&str> = feed
            .calendars
            .iter()
            .map(|c| c.service_id.as_ref())
            .collect();

        feed.calendar_dates
            .iter()
            .enumerate()
            .filter(|(_, cd)| !calendar_ids.contains(cd.service_id.as_ref()))
            .map(|(i, cd)| {
                let line = i + 2;
                ValidationError::new(RULE_ID, SECTION, Severity::Warning)
                    .message(format!(
                        "service_id '{}' in calendar_dates.txt line {} is not defined in calendar.txt",
                        cd.service_id, line
                    ))
                    .file(FILE)
                    .line(line)
                    .field("service_id")
                    .value(cd.service_id.as_ref())
            })
            .collect()
    }
}
