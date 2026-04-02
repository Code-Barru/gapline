//! Field definition validation for `feed_info.txt`.

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "feed_info.txt";
const SECTION: &str = "4";
const RULE_ID: &str = "field_definition_feed_info";

/// Validates recommended field presence for `feed_info.txt`.
///
/// - `feed_start_date` is recommended (WARNING if absent).
/// - `feed_end_date` is recommended (WARNING if absent).
pub struct FeedInfoFieldDefinitionRule;

impl ValidationRule for FeedInfoFieldDefinitionRule {
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
        if !feed.has_file("feed_info.txt") {
            return Vec::new();
        }

        let Some(info) = &feed.feed_info else {
            return Vec::new();
        };

        let mut errors = Vec::new();
        let line = 2;

        if info.feed_start_date.is_none() {
            errors.push(
                ValidationError::new(RULE_ID, SECTION, Severity::Warning)
                    .message("feed_start_date is recommended")
                    .file(FILE)
                    .line(line)
                    .field("feed_start_date"),
            );
        }

        if info.feed_end_date.is_none() {
            errors.push(
                ValidationError::new(RULE_ID, SECTION, Severity::Warning)
                    .message("feed_end_date is recommended")
                    .file(FILE)
                    .line(line)
                    .field("feed_end_date"),
            );
        }

        errors
    }
}
