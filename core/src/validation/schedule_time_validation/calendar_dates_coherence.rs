//! Coherence of `calendar_dates.txt` vs `calendar.txt` (section 7.6).
//!
//! Emits:
//! - `service_never_active` (WARNING): a service lives only in
//!   `calendar_dates.txt` and every entry is `exception_type=2` (Removed),
//!   so no day is ever added.
//! - `exception_date_out_of_range` (WARNING): an exception date falls
//!   outside the `[start_date, end_date]` range of the matching calendar.

use std::collections::HashMap;

use crate::models::{ExceptionType, GtfsDate, GtfsFeed};
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "calendar_dates.txt";
const SECTION: &str = "7";

/// Validates the coherence of `calendar_dates` exceptions with `calendar.txt`.
pub struct CalendarDatesCoherenceRule;

impl ValidationRule for CalendarDatesCoherenceRule {
    fn rule_id(&self) -> &'static str {
        "calendar_dates_coherence"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn progress_group(&self) -> &'static str {
        "7-cal"
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Build lookup: service_id → (start_date, end_date) from calendar.txt.
        let calendar_ranges: HashMap<&str, (GtfsDate, GtfsDate)> = feed
            .calendars
            .iter()
            .map(|c| (c.service_id.as_ref(), (c.start_date, c.end_date)))
            .collect();

        // Track per-service aggregates to detect services that never activate.
        let mut agg: HashMap<&str, ServiceAgg> = HashMap::new();

        for (i, cd) in feed.calendar_dates.iter().enumerate() {
            let line = i + 2;
            let sid = cd.service_id.as_ref();

            let entry = agg.entry(sid).or_insert(ServiceAgg {
                first_line: line,
                has_addition: false,
                has_removal: false,
            });
            match cd.exception_type {
                ExceptionType::Added => entry.has_addition = true,
                ExceptionType::Removed => entry.has_removal = true,
            }

            // Exception date must lie within the matching calendar's range.
            if let Some(&(start, end)) = calendar_ranges.get(sid)
                && start <= end
                && (cd.date < start || cd.date > end)
            {
                errors.push(
                    ValidationError::new("exception_date_out_of_range", SECTION, Severity::Warning)
                        .message(format!(
                            "calendar_dates entry for service '{sid}' has date {} \
                         outside the calendar range [{start}, {end}]",
                            cd.date
                        ))
                        .file(FILE)
                        .line(line)
                        .field("date")
                        .value(cd.date.to_string()),
                );
            }
        }

        // A service that only appears in calendar_dates with removals and
        // no additions can never be active.
        for (sid, s) in &agg {
            if calendar_ranges.contains_key(*sid) {
                continue;
            }
            if s.has_removal && !s.has_addition {
                errors.push(
                    ValidationError::new("service_never_active", SECTION, Severity::Warning)
                        .message(format!(
                            "service '{sid}' is defined only in calendar_dates.txt \
                             with removal entries and no additions; it will never be active"
                        ))
                        .file(FILE)
                        .line(s.first_line)
                        .field("service_id")
                        .value((*sid).to_string()),
                );
            }
        }

        errors
    }
}

struct ServiceAgg {
    first_line: usize,
    has_addition: bool,
    has_removal: bool,
}
