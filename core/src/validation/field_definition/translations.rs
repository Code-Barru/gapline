//! Field definition validation for `translations.txt`.

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "translations.txt";
const SECTION: &str = "4";
const RULE_ID: &str = "field_definition_translations";

/// Validates conditional field constraints for `translations.txt`.
///
/// - `record_id` is required unless `table_name` is `feed_info`.
pub struct TranslationsFieldDefinitionRule;

impl ValidationRule for TranslationsFieldDefinitionRule {
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
        if !feed.has_file("translations.txt") {
            return Vec::new();
        }

        let mut errors = Vec::new();

        for (i, translation) in feed.translations.iter().enumerate() {
            let line = i + 2;

            if translation.table_name != "feed_info"
                && translation.record_id.as_ref().is_none_or(String::is_empty)
            {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message("record_id is required unless table_name is feed_info")
                        .file(FILE)
                        .line(line)
                        .field("record_id"),
                );
            }
        }

        errors
    }
}
