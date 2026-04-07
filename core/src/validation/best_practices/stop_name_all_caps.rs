use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "stops.txt";
const SECTION: &str = "8";
const RULE_ID: &str = "stop_name_all_caps";

/// Flags stop names written entirely in uppercase.
pub struct StopNameAllCapsRule;

impl ValidationRule for StopNameAllCapsRule {
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
        feed.stops
            .iter()
            .enumerate()
            .filter_map(|(i, stop)| {
                let name = stop.stop_name.as_deref()?;
                let has_letter = name.chars().any(char::is_alphabetic);
                if has_letter && name == name.to_uppercase() {
                    Some(
                        ValidationError::new(RULE_ID, SECTION, Severity::Warning)
                            .message(format!(
                                "stop_name '{name}' is entirely uppercase; mixed case is recommended"
                            ))
                            .file(FILE)
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
