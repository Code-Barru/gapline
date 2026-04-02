//! Field definition validation for `attributions.txt`.

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "attributions.txt";
const SECTION: &str = "4";
const RULE_ID: &str = "field_definition_attributions";

/// Validates conditional field constraints for `attributions.txt`.
///
/// - At least one of `is_producer`, `is_operator`, or `is_authority` must be `1`.
pub struct AttributionsFieldDefinitionRule;

impl ValidationRule for AttributionsFieldDefinitionRule {
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
        if !feed.has_file("attributions.txt") {
            return Vec::new();
        }

        let mut errors = Vec::new();

        for (i, attribution) in feed.attributions.iter().enumerate() {
            let line = i + 2;

            let has_role = attribution.is_producer == Some(1)
                || attribution.is_operator == Some(1)
                || attribution.is_authority == Some(1);

            if !has_role {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message("at least one of is_producer, is_operator, is_authority must be 1")
                        .file(FILE)
                        .line(line)
                        .field("is_producer"),
                );
            }
        }

        errors
    }
}
