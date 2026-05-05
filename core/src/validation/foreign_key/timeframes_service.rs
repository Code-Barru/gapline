//! FK rule: `timeframes.service_id` → `calendar.service_id` OR
//! `calendar_dates.service_id`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "timeframes.txt";
use super::{RULE_ID, SECTION};

pub struct TimeframesServiceFkRule;

impl ValidationRule for TimeframesServiceFkRule {
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

        feed.timeframes
            .iter()
            .enumerate()
            .filter(|(_, tf)| !valid_ids.contains(tf.service_id.as_ref()))
            .map(|(i, tf)| {
                let line = i + 2;
                ValidationError::new(RULE_ID, SECTION, Severity::Error)
                    .message(format!(
                        "service_id '{}' in timeframes.txt line {} references non-existent service in calendar.txt or calendar_dates.txt",
                        tf.service_id, line
                    ))
                    .file(FILE)
                    .line(line)
                    .field("service_id")
                    .value(tf.service_id.as_ref())
            })
            .collect()
    }
}
