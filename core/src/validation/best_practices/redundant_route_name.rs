use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "routes.txt";
const SECTION: &str = "8";
const RULE_ID: &str = "redundant_route_name";

/// Flags routes where `route_long_name` is identical to `route_short_name`.
pub struct RedundantRouteNameRule;

impl ValidationRule for RedundantRouteNameRule {
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
        feed.routes
            .iter()
            .enumerate()
            .filter_map(|(i, route)| {
                let short = route.route_short_name.as_deref()?;
                let long = route.route_long_name.as_deref()?;
                if short == long {
                    Some(
                        ValidationError::new(RULE_ID, SECTION, Severity::Warning)
                            .message(format!(
                                "route_long_name '{long}' is identical to route_short_name"
                            ))
                            .file(FILE)
                            .line(i + 2)
                            .field("route_long_name")
                            .value(long),
                    )
                } else {
                    None
                }
            })
            .collect()
    }
}
