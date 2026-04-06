//! Calendar date-range coherence validation for `calendar.txt` (section 7.5).
//!
//! Emits:
//! - `inverted_date_range` (ERROR): `start_date > end_date`.
//! - `inactive_service` (WARNING): all seven weekday flags are 0 and the
//!   service has no `exception_type=1` entry in `calendar_dates.txt`.

use std::collections::HashSet;

use crate::models::{ExceptionType, GtfsFeed};
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "calendar.txt";
const SECTION: &str = "7";

/// Validates per-row date ranges and detects fully inactive services.
pub struct CalendarRangesRule;

impl ValidationRule for CalendarRangesRule {
    fn rule_id(&self) -> &'static str {
        "calendar_ranges"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if feed.calendars.is_empty() {
            return errors;
        }

        // Pre-compute the set of service_ids that have at least one
        // `exception_type=1` (Added) entry — these services remain active
        // even when all weekday flags are 0.
        let services_with_additions: HashSet<&str> = feed
            .calendar_dates
            .iter()
            .filter(|cd| cd.exception_type == ExceptionType::Added)
            .map(|cd| cd.service_id.as_ref())
            .collect();

        for (i, cal) in feed.calendars.iter().enumerate() {
            let line = i + 2;

            if cal.start_date > cal.end_date {
                errors.push(
                    ValidationError::new("inverted_date_range", SECTION, Severity::Error)
                        .message(format!(
                            "service '{}' has start_date {} after end_date {}",
                            cal.service_id, cal.start_date, cal.end_date
                        ))
                        .file(FILE)
                        .line(line)
                        .field("start_date")
                        .value(cal.start_date.to_string()),
                );
            }

            let all_days_off = !cal.monday
                && !cal.tuesday
                && !cal.wednesday
                && !cal.thursday
                && !cal.friday
                && !cal.saturday
                && !cal.sunday;

            if all_days_off && !services_with_additions.contains(cal.service_id.as_ref()) {
                errors.push(
                    ValidationError::new("inactive_service", SECTION, Severity::Warning)
                        .message(format!(
                            "service '{}' has all weekdays set to 0 and no \
                             exception_type=1 entry in calendar_dates.txt; \
                             the service will never be active",
                            cal.service_id
                        ))
                        .file(FILE)
                        .line(line)
                        .field("service_id")
                        .value(cal.service_id.to_string()),
                );
            }
        }

        errors
    }
}
