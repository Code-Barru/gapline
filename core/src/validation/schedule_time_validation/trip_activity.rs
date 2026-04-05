//! Trip activity validation (section 7.6).
//!
//! Computes the number of active days for each service (combining
//! `calendar.txt` weekday patterns within the date range with
//! `calendar_dates.txt` exceptions) and flags trips whose service yields
//! fewer active days than the configured threshold with `low_trip_activity`.

use std::collections::{HashMap, HashSet};

use chrono::{Datelike, Duration, Weekday};

use crate::models::{Calendar, ExceptionType, GtfsDate, GtfsFeed};
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "trips.txt";
const SECTION: &str = "7";

/// Validates that each trip's service has at least the configured number of
/// active days.
pub struct TripActivityRule {
    min_active_days: u32,
}

impl TripActivityRule {
    #[must_use]
    pub const fn new(min_active_days: u32) -> Self {
        Self { min_active_days }
    }
}

impl ValidationRule for TripActivityRule {
    fn rule_id(&self) -> &'static str {
        "low_trip_activity"
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
        let active_days = compute_active_days(feed);
        let min_active = self.min_active_days as usize;
        let mut errors = Vec::new();

        for (i, trip) in feed.trips.iter().enumerate() {
            let line = i + 2;
            let sid = trip.service_id.as_ref();
            let days = active_days.get(sid).copied().unwrap_or(0);

            if days < min_active {
                errors.push(
                    ValidationError::new("low_trip_activity", SECTION, Severity::Warning)
                        .message(format!(
                            "trip '{}' uses service '{sid}' which is active on \
                             {days} day(s), below the {}-day threshold",
                            trip.trip_id, self.min_active_days
                        ))
                        .file(FILE)
                        .line(line)
                        .field("service_id")
                        .value(trip.service_id.to_string()),
                );
            }
        }

        errors
    }
}

/// Computes the number of active days per `service_id`, combining
/// `calendar.txt` weekday patterns with `calendar_dates.txt` exceptions.
fn compute_active_days(feed: &GtfsFeed) -> HashMap<&str, usize> {
    let mut per_service: HashMap<&str, HashSet<GtfsDate>> = HashMap::new();

    for cal in &feed.calendars {
        let sid = cal.service_id.as_ref();
        let set = per_service.entry(sid).or_default();
        // Guard against inverted ranges (already flagged by CalendarRangesRule).
        if cal.start_date > cal.end_date {
            continue;
        }
        let mut current = cal.start_date.0;
        while current <= cal.end_date.0 {
            if weekday_active(cal, current.weekday()) {
                set.insert(GtfsDate(current));
            }
            current += Duration::days(1);
        }
    }

    for cd in &feed.calendar_dates {
        let sid = cd.service_id.as_ref();
        let set = per_service.entry(sid).or_default();
        match cd.exception_type {
            ExceptionType::Added => {
                set.insert(cd.date);
            }
            ExceptionType::Removed => {
                set.remove(&cd.date);
            }
        }
    }

    per_service
        .into_iter()
        .map(|(sid, set)| (sid, set.len()))
        .collect()
}

fn weekday_active(cal: &Calendar, wd: Weekday) -> bool {
    match wd {
        Weekday::Mon => cal.monday,
        Weekday::Tue => cal.tuesday,
        Weekday::Wed => cal.wednesday,
        Weekday::Thu => cal.thursday,
        Weekday::Fri => cal.friday,
        Weekday::Sat => cal.saturday,
        Weekday::Sun => cal.sunday,
    }
}
