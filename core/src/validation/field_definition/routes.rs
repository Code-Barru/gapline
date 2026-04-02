//! Field definition validation for `routes.txt`.

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "routes.txt";
const SECTION: &str = "4";
const RULE_ID: &str = "field_definition_routes";

/// Validates conditional field constraints for `routes.txt`.
///
/// - At least one of `route_short_name` or `route_long_name` must be present.
/// - `agency_id` is required when the feed contains more than one agency.
pub struct RoutesFieldDefinitionRule;

impl ValidationRule for RoutesFieldDefinitionRule {
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

        for (i, route) in feed.routes.iter().enumerate() {
            let line = i + 2;

            let short_empty = route.route_short_name.as_ref().is_none_or(String::is_empty);
            let long_empty = route.route_long_name.as_ref().is_none_or(String::is_empty);

            if short_empty && long_empty {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message("At least one of route_short_name or route_long_name is required")
                        .file(FILE)
                        .line(line)
                        .field("route_short_name"),
                );
            }

            if multiple_agencies {
                let id_missing = route
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
