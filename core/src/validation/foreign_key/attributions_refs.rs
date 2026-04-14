//! FK rules: `attributions.agency_id` → `agency`, `attributions.route_id` → `routes`,
//! `attributions.trip_id` → `trips`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "attributions.txt";
use super::{RULE_ID, SECTION};

/// If `agency_id`, `route_id`, or `trip_id` is non-empty in attributions.txt,
/// each must exist in its respective table.
pub struct AttributionsRefsFkRule;

impl ValidationRule for AttributionsRefsFkRule {
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
        let agency_ids: HashSet<&str> = feed
            .agencies
            .iter()
            .filter_map(|a| a.agency_id.as_ref().map(AsRef::as_ref))
            .collect();
        let route_ids: HashSet<&str> = feed.routes.iter().map(|r| r.route_id.as_ref()).collect();
        let trip_ids: HashSet<&str> = feed.trips.iter().map(|t| t.trip_id.as_ref()).collect();

        let mut errors = Vec::new();

        for (i, attr) in feed.attributions.iter().enumerate() {
            let line = i + 2;

            if let Some(id) = attr.agency_id.as_ref()
                && !agency_ids.contains(id.as_ref())
            {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message(format!(
                            "agency_id '{id}' in attributions.txt line {line} references non-existent agency in agency.txt"
                        ))
                        .file(FILE)
                        .line(line)
                        .field("agency_id")
                        .value(id.as_ref()),
                );
            }

            if let Some(id) = attr.route_id.as_ref()
                && !route_ids.contains(id.as_ref())
            {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message(format!(
                            "route_id '{id}' in attributions.txt line {line} references non-existent route in routes.txt"
                        ))
                        .file(FILE)
                        .line(line)
                        .field("route_id")
                        .value(id.as_ref()),
                );
            }

            if let Some(id) = attr.trip_id.as_ref()
                && !trip_ids.contains(id.as_ref())
            {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message(format!(
                            "trip_id '{id}' in attributions.txt line {line} references non-existent trip in trips.txt"
                        ))
                        .file(FILE)
                        .line(line)
                        .field("trip_id")
                        .value(id.as_ref()),
                );
            }
        }

        errors
    }
}
