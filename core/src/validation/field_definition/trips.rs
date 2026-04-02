//! Field definition validation for `trips.txt`.

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "trips.txt";
const SECTION: &str = "4";
const RULE_ID: &str = "field_definition_trips";

/// Validates conditional field constraints for `trips.txt`.
///
/// - `shape_id` is required when `shapes.txt` is present in the feed.
pub struct TripsFieldDefinitionRule;

impl ValidationRule for TripsFieldDefinitionRule {
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
        if !feed.has_file("shapes.txt") {
            return Vec::new();
        }

        let mut errors = Vec::new();

        for (i, trip) in feed.trips.iter().enumerate() {
            let line = i + 2;

            let missing = trip
                .shape_id
                .as_ref()
                .is_none_or(|id| id.as_ref().is_empty());
            if missing {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message("shape_id is required when shapes.txt is present in the feed")
                        .file(FILE)
                        .line(line)
                        .field("shape_id"),
                );
            }
        }

        errors
    }
}
