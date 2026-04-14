//! FK rule: `stops.level_id` → `levels.level_id`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "stops.txt";
use super::{RULE_ID, SECTION};

/// If `level_id` is non-empty and levels.txt is present, the value must
/// reference an existing `level_id` in levels.txt.
pub struct StopsLevelFkRule;

impl ValidationRule for StopsLevelFkRule {
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
        if !feed.has_file("levels.txt") {
            return Vec::new();
        }

        let valid_ids: HashSet<&str> = feed.levels.iter().map(|l| l.level_id.as_ref()).collect();

        feed.stops
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                let lid = s.level_id.as_ref()?;
                if valid_ids.contains(lid.as_ref()) {
                    return None;
                }
                let line = i + 2;
                Some(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message(format!(
                            "level_id '{lid}' in stops.txt line {line} references non-existent level in levels.txt"
                        ))
                        .file(FILE)
                        .line(line)
                        .field("level_id")
                        .value(lid.as_ref()),
                )
            })
            .collect()
    }
}
