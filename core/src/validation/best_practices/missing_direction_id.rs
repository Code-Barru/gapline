use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "trips.txt";
const SECTION: &str = "8";
const RULE_ID: &str = "missing_direction_id";

/// Warns when no trip in the feed provides a `direction_id`.
pub struct MissingDirectionIdRule;

impl ValidationRule for MissingDirectionIdRule {
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
        if !feed.trips.is_empty() && feed.trips.iter().all(|t| t.direction_id.is_none()) {
            vec![
                ValidationError::new(RULE_ID, SECTION, Severity::Warning)
                    .message("direction_id is recommended for round-trip routes")
                    .file(FILE)
                    .field("direction_id"),
            ]
        } else {
            Vec::new()
        }
    }
}
