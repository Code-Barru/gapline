//! Advisory rule: `calendar_dates.service_id` not in `calendar.service_id`.
//!
//! A `service_id` defined only in `calendar_dates.txt` is valid GTFS (services
//! can be defined entirely through exceptions), so this is a WARNING, not an
//! ERROR.

use std::collections::HashSet;

use super::SECTION;
use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "calendar_dates.txt";
const RULE_ID: &str = "calendar_dates_service_not_in_calendar";

/// If `calendar.txt` exists, warns about `service_id` values in
/// `calendar_dates.txt` that have no base schedule in `calendar.txt`.
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
                        "service_id '{}' in calendar_dates.txt line {} has no base schedule in calendar.txt",
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
