//! Shared service-date cache (used by sections 7.6 and 7.12).
//!
//! Computes the set of active dates per `service_id` by expanding
//! `calendar.txt` weekday patterns within their date ranges and applying
//! `calendar_dates.txt` exceptions. The result is cached behind a
//! [`OnceLock`] so that multiple validation rules can share it without
//! redundant computation.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use chrono::{Datelike, Duration, Weekday};

use crate::models::{Calendar, ExceptionType, GtfsDate, GtfsFeed};

/// Thread-safe, lazily-computed cache of active dates per service.
///
/// Wrap in `Arc` and clone into every rule that needs service-date data.
/// The first call to [`get`](Self::get) computes the map; subsequent calls
/// return a reference to the cached result.
pub struct ServiceDateCache {
    inner: OnceLock<HashMap<String, HashSet<GtfsDate>>>,
}

impl Default for ServiceDateCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceDateCache {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: OnceLock::new(),
        }
    }

    /// Returns the map of `service_id → active dates`. Computes on first
    /// call, returns cached result afterwards.
    pub fn get(&self, feed: &GtfsFeed) -> &HashMap<String, HashSet<GtfsDate>> {
        self.inner.get_or_init(|| compute(feed))
    }
}

fn compute(feed: &GtfsFeed) -> HashMap<String, HashSet<GtfsDate>> {
    let mut per_service: HashMap<String, HashSet<GtfsDate>> = HashMap::new();

    for cal in &feed.calendars {
        let sid = cal.service_id.to_string();
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
        let sid = cd.service_id.to_string();
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
