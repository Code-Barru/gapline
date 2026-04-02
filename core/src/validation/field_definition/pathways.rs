//! Field definition validation for `pathways.txt`.

use crate::models::GtfsFeed;
use crate::models::PathwayMode;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "pathways.txt";
const SECTION: &str = "4";
const RULE_ID: &str = "field_definition_pathways";

/// Validates conditional field constraints for `pathways.txt`.
///
/// - `length` is required when `pathway_mode` is `FareGate` (6) or `ExitGate` (7).
pub struct PathwaysFieldDefinitionRule;

impl ValidationRule for PathwaysFieldDefinitionRule {
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
        if !feed.has_file("pathways.txt") {
            return Vec::new();
        }

        let mut errors = Vec::new();

        for (i, pathway) in feed.pathways.iter().enumerate() {
            let line = i + 2;

            if matches!(
                pathway.pathway_mode,
                PathwayMode::FareGate | PathwayMode::ExitGate
            ) && pathway.length.is_none()
            {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message(
                            "length is required when pathway_mode is 6 (FareGate) or 7 (ExitGate)",
                        )
                        .file(FILE)
                        .line(line)
                        .field("length"),
                );
            }
        }

        errors
    }
}
