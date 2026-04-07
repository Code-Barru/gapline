use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "routes.txt";
const SECTION: &str = "8";
const RULE_ID: &str = "missing_route_colors";

/// Flags routes missing `route_color` or `route_text_color`.
pub struct MissingRouteColorsRule;

impl ValidationRule for MissingRouteColorsRule {
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
        let mut errors = Vec::new();
        for (i, route) in feed.routes.iter().enumerate() {
            let line = i + 2;
            if route.route_color.is_none() {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Info)
                        .message("route_color is recommended for display purposes")
                        .file(FILE)
                        .line(line)
                        .field("route_color"),
                );
            }
            if route.route_text_color.is_none() {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Info)
                        .message("route_text_color is recommended for display purposes")
                        .file(FILE)
                        .line(line)
                        .field("route_text_color"),
                );
            }
        }
        errors
    }
}
