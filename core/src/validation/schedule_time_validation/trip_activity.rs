//! Trip activity validation (section 7.6).
//!
//! Flags trips whose service yields fewer active days than the configured
//! threshold with `low_trip_activity`. Active-day computation is delegated
//! to the shared [`ServiceDateCache`].

use std::collections::HashSet;
use std::sync::Arc;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

use super::service_dates::ServiceDateCache;

const FILE: &str = "trips.txt";
const SECTION: &str = "7";

/// Validates that each trip's service has at least the configured number of
/// active days.
pub struct TripActivityRule {
    min_active_days: u32,
    cache: Arc<ServiceDateCache>,
}

impl TripActivityRule {
    #[must_use]
    pub fn new(min_active_days: u32, cache: Arc<ServiceDateCache>) -> Self {
        Self {
            min_active_days,
            cache,
        }
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

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let active_dates = self.cache.get(feed);
        let min_active = self.min_active_days as usize;
        let mut errors = Vec::new();

        for (i, trip) in feed.trips.iter().enumerate() {
            let line = i + 2;
            let sid = trip.service_id.to_string();
            let days = active_dates.get(&sid).map_or(0, HashSet::len);

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
