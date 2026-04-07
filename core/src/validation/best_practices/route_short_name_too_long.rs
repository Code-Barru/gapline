use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "routes.txt";
const SECTION: &str = "8";
const RULE_ID: &str = "route_short_name_too_long";

#[derive(Clone, Copy)]
pub struct NamingThresholds {
    pub max_route_short_name_length: usize,
}

/// Flags routes whose `route_short_name` exceeds a configurable length.
pub struct RouteShortNameTooLongRule {
    max_len: usize,
}

impl RouteShortNameTooLongRule {
    #[must_use]
    pub fn new(thresholds: NamingThresholds) -> Self {
        Self {
            max_len: thresholds.max_route_short_name_length,
        }
    }
}

impl ValidationRule for RouteShortNameTooLongRule {
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
                let name = route.route_short_name.as_deref()?;
                if name.len() > self.max_len {
                    Some(
                        ValidationError::new(RULE_ID, SECTION, Severity::Warning)
                            .message(format!(
                                "route_short_name '{name}' exceeds {} characters",
                                self.max_len
                            ))
                            .file(FILE)
                            .line(i + 2)
                            .field("route_short_name")
                            .value(name),
                    )
                } else {
                    None
                }
            })
            .collect()
    }
}
