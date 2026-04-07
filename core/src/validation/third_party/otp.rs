use std::collections::HashMap;

use crate::models::{GtfsFeed, TripId};
use crate::validation::{Severity, ValidationError, ValidationRule};

const SECTION: &str = "13";

// ---- otp_trip_too_few_stops ------------------------------------------------

const FEW_STOPS_RULE_ID: &str = "otp_trip_too_few_stops";

/// Flags trips with fewer than 2 `stop_times`.
pub struct OtpTripTooFewStopsRule;

impl ValidationRule for OtpTripTooFewStopsRule {
    fn rule_id(&self) -> &'static str {
        FEW_STOPS_RULE_ID
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut counts: HashMap<&TripId, usize> = HashMap::new();
        for st in &feed.stop_times {
            *counts.entry(&st.trip_id).or_default() += 1;
        }

        feed.trips
            .iter()
            .enumerate()
            .filter(|(_, trip)| counts.get(&trip.trip_id).copied().unwrap_or(0) < 2)
            .map(|(i, trip)| {
                ValidationError::new(FEW_STOPS_RULE_ID, SECTION, Severity::Error)
                    .message(format!(
                        "trip '{}' has fewer than 2 stop_times",
                        trip.trip_id
                    ))
                    .file("trips.txt")
                    .line(i + 2)
                    .field("trip_id")
                    .value(trip.trip_id.as_ref())
            })
            .collect()
    }
}

// ---- otp_missing_feed_version ----------------------------------------------

const VERSION_RULE_ID: &str = "otp_missing_feed_version";

/// Flags feeds missing `feed_version` in `feed_info.txt`.
pub struct OtpMissingFeedVersionRule;

impl ValidationRule for OtpMissingFeedVersionRule {
    fn rule_id(&self) -> &'static str {
        VERSION_RULE_ID
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let Some(info) = &feed.feed_info else {
            return Vec::new();
        };

        if info.feed_version.is_none() {
            vec![
                ValidationError::new(VERSION_RULE_ID, SECTION, Severity::Warning)
                    .message("feed_version is recommended for feed versioning")
                    .file("feed_info.txt")
                    .line(2)
                    .field("feed_version"),
            ]
        } else {
            Vec::new()
        }
    }
}
