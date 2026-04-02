//! Field definition validation for `stops.txt`.

use crate::models::GtfsFeed;
use crate::models::LocationType;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "stops.txt";
const SECTION: &str = "4";
const RULE_ID: &str = "field_definition_stops";

/// Validates conditional field constraints for `stops.txt`.
///
/// - `stop_name`, `stop_lat`, `stop_lon` required for `location_type` 0 (Stop) and 1 (Station).
/// - `parent_station` required for `location_type` 2, 3, 4.
/// - `parent_station` forbidden (WARNING) for `location_type` 1.
pub struct StopsFieldDefinitionRule;

impl ValidationRule for StopsFieldDefinitionRule {
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

        for (i, stop) in feed.stops.iter().enumerate() {
            let line = i + 2;
            // None means location_type 0 (StopOrPlatform) per GTFS spec default.
            let loc_type = stop.location_type.unwrap_or(LocationType::StopOrPlatform);

            match loc_type {
                LocationType::StopOrPlatform | LocationType::Station => {
                    if stop.stop_name.as_ref().is_none_or(String::is_empty) {
                        errors.push(
                            ValidationError::new(RULE_ID, SECTION, Severity::Error)
                                .message("stop_name is required for location_type 0 and 1")
                                .file(FILE)
                                .line(line)
                                .field("stop_name"),
                        );
                    }
                    if stop.stop_lat.is_none() {
                        errors.push(
                            ValidationError::new(RULE_ID, SECTION, Severity::Error)
                                .message("stop_lat is required for location_type 0 and 1")
                                .file(FILE)
                                .line(line)
                                .field("stop_lat"),
                        );
                    }
                    if stop.stop_lon.is_none() {
                        errors.push(
                            ValidationError::new(RULE_ID, SECTION, Severity::Error)
                                .message("stop_lon is required for location_type 0 and 1")
                                .file(FILE)
                                .line(line)
                                .field("stop_lon"),
                        );
                    }
                }
                LocationType::EntranceExit
                | LocationType::GenericNode
                | LocationType::BoardingArea => {}
            }

            match loc_type {
                LocationType::EntranceExit
                | LocationType::GenericNode
                | LocationType::BoardingArea => {
                    let missing = stop
                        .parent_station
                        .as_ref()
                        .is_none_or(|id| id.as_ref().is_empty());
                    if missing {
                        errors.push(
                            ValidationError::new(RULE_ID, SECTION, Severity::Error)
                                .message("parent_station is required for location_type 2, 3, and 4")
                                .file(FILE)
                                .line(line)
                                .field("parent_station"),
                        );
                    }
                }
                LocationType::Station => {
                    if stop.parent_station.is_some() {
                        errors.push(
                            ValidationError::new(RULE_ID, SECTION, Severity::Warning)
                                .message("parent_station should not be set for location_type 1 (Station)")
                                .file(FILE)
                                .line(line)
                                .field("parent_station"),
                        );
                    }
                }
                LocationType::StopOrPlatform => {}
            }
        }

        errors
    }
}
