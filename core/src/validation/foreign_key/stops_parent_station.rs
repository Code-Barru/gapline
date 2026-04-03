//! FK rule: `stops.parent_station` → `stops.stop_id` (self-reference, `location_type=1`).

use std::collections::HashMap;

use crate::models::{GtfsFeed, LocationType};
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "stops.txt";
const SECTION: &str = "5";
const RULE_ID: &str = "foreign_key_violation";

/// If `parent_station` is non-empty, it must reference an existing `stop_id`
/// with `location_type = 1` (Station).
pub struct StopsParentStationFkRule;

impl ValidationRule for StopsParentStationFkRule {
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
        let stops_by_id: HashMap<&str, Option<LocationType>> = feed
            .stops
            .iter()
            .map(|s| (s.stop_id.as_ref(), s.location_type))
            .collect();

        let mut errors = Vec::new();

        for (i, stop) in feed.stops.iter().enumerate() {
            let Some(parent_id) = &stop.parent_station else {
                continue;
            };
            let line = i + 2;

            match stops_by_id.get(parent_id.as_ref()) {
                None => {
                    errors.push(
                        ValidationError::new(RULE_ID, SECTION, Severity::Error)
                            .message(format!(
                                "parent_station '{parent_id}' in stops.txt line {line} references non-existent stop in stops.txt"
                            ))
                            .file(FILE)
                            .line(line)
                            .field("parent_station")
                            .value(parent_id.as_ref()),
                    );
                }
                Some(loc_type) if *loc_type != Some(LocationType::Station) => {
                    errors.push(
                        ValidationError::new(RULE_ID, SECTION, Severity::Error)
                            .message(format!(
                                "parent_station '{parent_id}' in stops.txt line {line} must reference a stop with location_type=1 (Station)"
                            ))
                            .file(FILE)
                            .line(line)
                            .field("parent_station")
                            .value(parent_id.as_ref()),
                    );
                }
                _ => {}
            }
        }

        errors
    }
}
