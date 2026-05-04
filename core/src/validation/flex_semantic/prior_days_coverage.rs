//! `booking_type=PriorDays` requires the referenced trip's service to span
//! more than one day, otherwise the rule cannot be honoured.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::models::{BookingType, GtfsFeed};
use crate::validation::schedule_time_validation::service_dates::ServiceDateCache;
use crate::validation::{Severity, ValidationError, ValidationRule};

const SECTION: &str = "9";
const RULE_ID: &str = "flex_insufficient_service_coverage";
const FILE: &str = "stop_times.txt";

pub struct PriorDaysServiceCoverageRule {
    cache: Arc<ServiceDateCache>,
}

impl PriorDaysServiceCoverageRule {
    #[must_use]
    pub fn new(cache: Arc<ServiceDateCache>) -> Self {
        Self { cache }
    }
}

impl ValidationRule for PriorDaysServiceCoverageRule {
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
        if !feed.has_flex() {
            return Vec::new();
        }

        let prior_days: HashSet<&str> = feed
            .booking_rules
            .iter()
            .filter(|br| matches!(br.booking_type, Some(BookingType::PriorDays)))
            .map(|br| br.booking_rule_id.as_ref())
            .collect();
        if prior_days.is_empty() {
            return Vec::new();
        }

        let trip_service: HashMap<&str, &str> = feed
            .trips
            .iter()
            .map(|t| (t.trip_id.as_ref(), t.service_id.as_ref()))
            .collect();

        let active_dates = self.cache.get(feed);

        let mut errors = Vec::new();
        let mut seen: HashSet<(&str, &str)> = HashSet::new();
        for (i, st) in feed.stop_times.iter().enumerate() {
            for br_id in [
                st.pickup_booking_rule_id.as_ref(),
                st.drop_off_booking_rule_id.as_ref(),
            ]
            .into_iter()
            .flatten()
            {
                let br_str = br_id.as_ref();
                if !prior_days.contains(br_str) {
                    continue;
                }
                let trip_str = st.trip_id.as_ref();
                if !seen.insert((trip_str, br_str)) {
                    continue;
                }
                let Some(service_id) = trip_service.get(trip_str) else {
                    continue;
                };
                let date_count = active_dates.get(*service_id).map_or(0, HashSet::len);
                if date_count <= 1 {
                    errors.push(
                        ValidationError::new(RULE_ID, SECTION, Severity::Warning)
                            .message(format!(
                                "booking_type=PriorDays requires multi-day service; '{service_id}' active on {date_count} day(s)"
                            ))
                            .file(FILE)
                            .line(i + 2)
                            .field("trip_id")
                            .value(trip_str.to_string()),
                    );
                }
            }
        }
        errors
    }
}
