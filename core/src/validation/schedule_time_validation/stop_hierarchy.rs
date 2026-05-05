//! Stop hierarchy validation (section 7.7).
//!
//! Validates the semantic correctness of parent–child relationships between
//! stops based on their `location_type`, and detects unused stations/stops.

use std::collections::{HashMap, HashSet};

use crate::models::{GtfsFeed, LocationType};
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "stops.txt";
const SECTION: &str = "7";

/// Validates that each stop's parent has the correct `location_type`:
/// - Boarding Area (4) → parent must be Stop/Platform (0)
/// - Entrance/Exit (2) → parent must be Station (1)
/// - Generic Node (3) → parent must be Station (1)
pub struct InvalidParentTypeRule;

impl ValidationRule for InvalidParentTypeRule {
    fn rule_id(&self) -> &'static str {
        "invalid_parent_type"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let stops_by_id: HashMap<&str, LocationType> = feed
            .stops
            .iter()
            .map(|s| {
                (
                    s.stop_id.as_ref(),
                    s.location_type.unwrap_or(LocationType::StopOrPlatform),
                )
            })
            .collect();

        let mut errors = Vec::new();

        for (i, stop) in feed.stops.iter().enumerate() {
            let Some(parent_id) = &stop.parent_station else {
                continue;
            };
            let loc_type = stop.location_type.unwrap_or(LocationType::StopOrPlatform);

            // Parent doesn't exist → FK rule already catches this.
            let Some(&parent_loc_type) = stops_by_id.get(parent_id.as_ref()) else {
                continue;
            };

            let expected = match loc_type {
                LocationType::BoardingArea => Some(LocationType::StopOrPlatform),
                LocationType::EntranceExit | LocationType::GenericNode => {
                    Some(LocationType::Station)
                }
                // StopOrPlatform: parent is optional, no type constraint.
                // Station: parent_station forbidden (handled by section 4).
                _ => None,
            };

            if let Some(expected_type) = expected
                && parent_loc_type != expected_type
            {
                let line = i + 2;
                let msg = match loc_type {
                    LocationType::BoardingArea => {
                        "Boarding area must have parent with location_type=0 (Stop/Platform)"
                    }
                    LocationType::EntranceExit => {
                        "Entrance/exit must have parent with location_type=1 (Station)"
                    }
                    LocationType::GenericNode => {
                        "Generic node must have parent with location_type=1 (Station)"
                    }
                    _ => unreachable!(),
                };
                errors.push(
                    ValidationError::new("invalid_parent_type", SECTION, Severity::Error)
                        .message(msg)
                        .file(FILE)
                        .line(line)
                        .field("parent_station")
                        .value(parent_id.as_ref()),
                );
            }
        }

        errors
    }
}

/// Warns when a Station (`location_type=1`) has no children - no stop references
/// it as `parent_station`.
pub struct UnusedStationRule;

impl ValidationRule for UnusedStationRule {
    fn rule_id(&self) -> &'static str {
        "unused_station"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let referenced_parents: HashSet<&str> = feed
            .stops
            .iter()
            .filter_map(|s| s.parent_station.as_ref().map(AsRef::as_ref))
            .collect();

        let mut errors = Vec::new();

        for (i, stop) in feed.stops.iter().enumerate() {
            let loc_type = stop.location_type.unwrap_or(LocationType::StopOrPlatform);
            if loc_type != LocationType::Station {
                continue;
            }
            if !referenced_parents.contains(stop.stop_id.as_ref()) {
                errors.push(
                    ValidationError::new("unused_station", SECTION, Severity::Warning)
                        .message(format!(
                            "Station '{}' has no child stops",
                            stop.stop_id.as_ref()
                        ))
                        .file(FILE)
                        .line(i + 2)
                        .field("stop_id")
                        .value(stop.stop_id.as_ref()),
                );
            }
        }

        errors
    }
}

/// Warns when a Stop/Platform (`location_type=0`) is not referenced by any
/// `stop_time`.
pub struct UnusedStopRule;

impl ValidationRule for UnusedStopRule {
    fn rule_id(&self) -> &'static str {
        "unused_stop"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let referenced_stops: HashSet<&str> = feed
            .stop_times
            .iter()
            .map(|st| st.stop_id.as_ref())
            .collect();

        let mut errors = Vec::new();

        for (i, stop) in feed.stops.iter().enumerate() {
            let loc_type = stop.location_type.unwrap_or(LocationType::StopOrPlatform);
            if loc_type != LocationType::StopOrPlatform {
                continue;
            }
            if !referenced_stops.contains(stop.stop_id.as_ref()) {
                errors.push(
                    ValidationError::new("unused_stop", SECTION, Severity::Warning)
                        .message(format!(
                            "Stop '{}' is not referenced by any stop_time",
                            stop.stop_id.as_ref()
                        ))
                        .file(FILE)
                        .line(i + 2)
                        .field("stop_id")
                        .value(stop.stop_id.as_ref()),
                );
            }
        }

        errors
    }
}
