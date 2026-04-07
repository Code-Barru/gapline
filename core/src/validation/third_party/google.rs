use std::sync::LazyLock;

use regex::Regex;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const SECTION: &str = "13";

// ---- google_coordinates_in_stop_name ---------------------------------------

const COORDS_RULE_ID: &str = "google_coordinates_in_stop_name";

static COORDS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-?\d+\.?\d*\s*,\s*-?\d+\.?\d*").expect("hard-coded regex is valid")
});

/// Flags stop names that contain raw coordinates.
pub struct GoogleCoordinatesInStopNameRule;

impl ValidationRule for GoogleCoordinatesInStopNameRule {
    fn rule_id(&self) -> &'static str {
        COORDS_RULE_ID
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        feed.stops
            .iter()
            .enumerate()
            .filter_map(|(i, stop)| {
                let name = stop.stop_name.as_deref()?;
                if COORDS_RE.is_match(name) {
                    Some(
                        ValidationError::new(COORDS_RULE_ID, SECTION, Severity::Warning)
                            .message(format!(
                                "stop_name '{name}' appears to contain raw coordinates"
                            ))
                            .file("stops.txt")
                            .line(i + 2)
                            .field("stop_name")
                            .value(name),
                    )
                } else {
                    None
                }
            })
            .collect()
    }
}

// ---- google_identical_route_colors -----------------------------------------

const COLORS_RULE_ID: &str = "google_identical_route_colors";

/// Flags routes where `route_color` and `route_text_color` are identical.
pub struct GoogleIdenticalRouteColorsRule;

impl ValidationRule for GoogleIdenticalRouteColorsRule {
    fn rule_id(&self) -> &'static str {
        COLORS_RULE_ID
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
                let color = route.route_color.as_ref()?;
                let text_color = route.route_text_color.as_ref()?;
                if color.0.eq_ignore_ascii_case(&text_color.0) {
                    Some(
                        ValidationError::new(COLORS_RULE_ID, SECTION, Severity::Warning)
                            .message(format!(
                                "route_color and route_text_color are both '{color}'"
                            ))
                            .file("routes.txt")
                            .line(i + 2)
                            .field("route_color")
                            .value(color.as_ref()),
                    )
                } else {
                    None
                }
            })
            .collect()
    }
}
