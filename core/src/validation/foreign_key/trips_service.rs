//! FK rule: `trips.service_id` → `calendar.service_id` OR `calendar_dates.service_id`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "trips.txt";
use super::{RULE_ID, SECTION};

/// Every `service_id` in trips.txt must exist in calendar.txt **or**
/// `calendar_dates.txt`. `calendar_dates` alone is sufficient.
pub struct TripsServiceFkRule;

impl ValidationRule for TripsServiceFkRule {
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
        let mut valid_ids: HashSet<&str> = feed
            .calendars
            .iter()
            .map(|c| c.service_id.as_ref())
            .collect();

        for cd in &feed.calendar_dates {
            valid_ids.insert(cd.service_id.as_ref());
        }

        feed.trips
            .iter()
            .enumerate()
            .filter(|(_, t)| !valid_ids.contains(t.service_id.as_ref()))
            .map(|(i, t)| {
                let line = i + 2;
                ValidationError::new(RULE_ID, SECTION, Severity::Error)
                    .message(format!(
                        "service_id '{}' in trips.txt line {} references non-existent service in calendar.txt or calendar_dates.txt",
                        t.service_id, line
                    ))
                    .file(FILE)
                    .line(line)
                    .field("service_id")
                    .value(t.service_id.as_ref())
            })
            .collect()
    }
}
