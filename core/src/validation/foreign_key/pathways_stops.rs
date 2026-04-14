//! FK rule: `pathways.from_stop_id` / `to_stop_id` → `stops.stop_id` with `location_type ∈ {2, 3, 4}`.

use std::collections::HashMap;

use crate::models::{GtfsFeed, LocationType};
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "pathways.txt";
use super::{RULE_ID, SECTION};

/// Both `from_stop_id` and `to_stop_id` in pathways.txt must reference existing
/// stops with `location_type` ∈ {2 (Entrance/Exit), 3 (Generic Node), 4 (Boarding Area)}.
pub struct PathwaysStopsFkRule;

impl ValidationRule for PathwaysStopsFkRule {
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

        for (i, pw) in feed.pathways.iter().enumerate() {
            let line = i + 2;

            for (field, stop_id) in [
                ("from_stop_id", &pw.from_stop_id),
                ("to_stop_id", &pw.to_stop_id),
            ] {
                match stops_by_id.get(stop_id.as_ref()) {
                    None => {
                        errors.push(
                            ValidationError::new(RULE_ID, SECTION, Severity::Error)
                                .message(format!(
                                    "{field} '{stop_id}' in pathways.txt line {line} references non-existent stop in stops.txt"
                                ))
                                .file(FILE)
                                .line(line)
                                .field(field)
                                .value(stop_id.as_ref()),
                        );
                    }
                    Some(loc_type)
                        if !matches!(
                            loc_type,
                            Some(
                                LocationType::EntranceExit
                                    | LocationType::GenericNode
                                    | LocationType::BoardingArea
                            )
                        ) =>
                    {
                        errors.push(
                            ValidationError::new(RULE_ID, SECTION, Severity::Error)
                                .message(format!(
                                    "{field} '{stop_id}' in pathways.txt line {line} must reference a stop with location_type 2, 3, or 4"
                                ))
                                .file(FILE)
                                .line(line)
                                .field(field)
                                .value(stop_id.as_ref()),
                        );
                    }
                    _ => {}
                }
            }
        }

        errors
    }
}
