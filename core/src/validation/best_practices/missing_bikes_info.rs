use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "trips.txt";
const SECTION: &str = "8";
const RULE_ID: &str = "missing_bikes_info";

/// Flags trips missing `bikes_allowed`.
pub struct MissingBikesInfoRule;

impl ValidationRule for MissingBikesInfoRule {
    fn rule_id(&self) -> &'static str {
        RULE_ID
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        feed.trips
            .iter()
            .enumerate()
            .filter(|(_, trip)| trip.bikes_allowed.is_none())
            .map(|(i, _)| {
                ValidationError::new(RULE_ID, SECTION, Severity::Info)
                    .message("bikes_allowed is recommended for accessibility")
                    .file(FILE)
                    .line(i + 2)
                    .field("bikes_allowed")
            })
            .collect()
    }
}
