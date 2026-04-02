//! Field definition validation for `agency.txt`.

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "agency.txt";
const SECTION: &str = "4";
const RULE_ID: &str = "field_definition_agency";

/// Validates conditional and required field constraints for `agency.txt`.
///
/// - `agency_name`, `agency_url`, `agency_timezone` must not be empty.
/// - `agency_id` is required when the feed contains more than one agency.
pub struct AgencyFieldDefinitionRule;

impl ValidationRule for AgencyFieldDefinitionRule {
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
        let mut errors = Vec::new();
        let multiple_agencies = feed.agencies.len() > 1;

        for (i, agency) in feed.agencies.iter().enumerate() {
            let line = i + 2;

            if agency.agency_name.is_empty() {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message("Required field agency_name is missing or empty")
                        .file(FILE)
                        .line(line)
                        .field("agency_name"),
                );
            }

            if agency.agency_url.as_ref().is_empty() {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message("Required field agency_url is missing or empty")
                        .file(FILE)
                        .line(line)
                        .field("agency_url"),
                );
            }

            if agency.agency_timezone.as_ref().is_empty() {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message("Required field agency_timezone is missing or empty")
                        .file(FILE)
                        .line(line)
                        .field("agency_timezone"),
                );
            }

            if multiple_agencies {
                let id_missing = agency
                    .agency_id
                    .as_ref()
                    .is_none_or(|id| id.as_ref().is_empty());
                if id_missing {
                    errors.push(
                        ValidationError::new(RULE_ID, SECTION, Severity::Error)
                            .message(
                                "agency_id is required when the feed contains multiple agencies",
                            )
                            .file(FILE)
                            .line(line)
                            .field("agency_id"),
                    );
                }
            }
        }

        errors
    }
}
