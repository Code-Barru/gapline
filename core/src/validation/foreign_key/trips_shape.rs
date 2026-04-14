//! FK rule: `trips.shape_id` → `shapes.shape_id`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "trips.txt";
use super::{RULE_ID, SECTION};

/// If `shape_id` is non-empty in trips.txt, it must exist in shapes.txt.
/// Empty `shape_id` is allowed (optional field).
pub struct TripsShapeFkRule;

impl ValidationRule for TripsShapeFkRule {
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
        let valid_ids: HashSet<&str> = feed.shapes.iter().map(|s| s.shape_id.as_ref()).collect();

        feed.trips
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                let sid = t.shape_id.as_ref()?;
                if valid_ids.contains(sid.as_ref()) {
                    return None;
                }
                let line = i + 2;
                Some(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message(format!(
                            "shape_id '{sid}' in trips.txt line {line} references non-existent shape in shapes.txt"
                        ))
                        .file(FILE)
                        .line(line)
                        .field("shape_id")
                        .value(sid.as_ref()),
                )
            })
            .collect()
    }
}
