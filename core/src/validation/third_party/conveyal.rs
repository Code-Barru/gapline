use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "trips.txt";
const SECTION: &str = "13";
const RULE_ID: &str = "conveyal_trip_without_shape";

/// Flags trips missing a `shape_id` when other trips reference shapes.
pub struct ConveyalTripWithoutShapeRule;

impl ValidationRule for ConveyalTripWithoutShapeRule {
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
        let shape_ids: HashSet<&str> = feed.shapes.iter().map(|s| s.shape_id.as_ref()).collect();

        if shape_ids.is_empty() {
            return Vec::new();
        }

        feed.trips
            .iter()
            .enumerate()
            .filter(|(_, trip)| trip.shape_id.is_none())
            .map(|(i, _)| {
                ValidationError::new(RULE_ID, SECTION, Severity::Warning)
                    .message("trip has no shape_id but other trips reference shapes")
                    .file(FILE)
                    .line(i + 2)
                    .field("shape_id")
            })
            .collect()
    }
}
