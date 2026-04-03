//! FK rule: `translations.record_id` → target table PK (based on `table_name`),
//! and `translations.record_sub_id` → `stop_times` `stop_sequence` for `stop_times`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "translations.txt";
const SECTION: &str = "5";
const RULE_ID: &str = "foreign_key_violation";

/// If `record_id` is non-empty in translations.txt, it must reference an existing
/// primary key in the table indicated by `table_name`.
///
/// If `record_sub_id` is non-empty and `table_name` is `stop_times`, the combination
/// of `record_id` (trip_id) + `record_sub_id` (stop_sequence) must exist.
pub struct TranslationsRecordFkRule;

impl ValidationRule for TranslationsRecordFkRule {
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

        // Pre-build PK sets for each table that translations can reference.
        let agency_ids: HashSet<&str> = feed
            .agencies
            .iter()
            .filter_map(|a| a.agency_id.as_ref().map(AsRef::as_ref))
            .collect();
        let stop_ids: HashSet<&str> = feed.stops.iter().map(|s| s.stop_id.as_ref()).collect();
        let route_ids: HashSet<&str> = feed.routes.iter().map(|r| r.route_id.as_ref()).collect();
        let trip_ids: HashSet<&str> = feed.trips.iter().map(|t| t.trip_id.as_ref()).collect();
        let pathway_ids: HashSet<&str> = feed
            .pathways
            .iter()
            .map(|p| p.pathway_id.as_ref())
            .collect();
        let level_ids: HashSet<&str> = feed.levels.iter().map(|l| l.level_id.as_ref()).collect();
        let attribution_ids: HashSet<&str> = feed
            .attributions
            .iter()
            .filter_map(|a| a.attribution_id.as_deref()) // String field, as_deref works
            .collect();
        // stop_times keyed by (trip_id, stop_sequence) for record_sub_id validation.
        let stop_time_keys: HashSet<(&str, u32)> = feed
            .stop_times
            .iter()
            .map(|st| (st.trip_id.as_ref(), st.stop_sequence))
            .collect();

        for (i, tr) in feed.translations.iter().enumerate() {
            let Some(record_id) = tr.record_id.as_deref() else {
                continue;
            };
            let line = i + 2;

            let pk_set: Option<&HashSet<&str>> = match tr.table_name.as_str() {
                "agency" => Some(&agency_ids),
                "stops" => Some(&stop_ids),
                "routes" => Some(&route_ids),
                "trips" => Some(&trip_ids),
                "stop_times" => Some(&trip_ids), // record_id is trip_id for stop_times
                "pathways" => Some(&pathway_ids),
                "levels" => Some(&level_ids),
                "attributions" => Some(&attribution_ids),
                "feed_info" => None, // single-row table, no PK to check
                _ => None,
            };

            if let Some(set) = pk_set {
                if !set.contains(record_id) {
                    errors.push(
                        ValidationError::new(RULE_ID, SECTION, Severity::Error)
                            .message(format!(
                                "record_id '{record_id}' in translations.txt line {line} references non-existent record in {}.txt",
                                tr.table_name
                            ))
                            .file(FILE)
                            .line(line)
                            .field("record_id")
                            .value(record_id),
                    );
                    continue; // no point checking sub_id if main id is bad
                }
            }

            // Validate record_sub_id for stop_times.
            if tr.table_name == "stop_times" {
                if let Some(sub_id) = tr.record_sub_id.as_deref() {
                    if let Ok(seq) = sub_id.parse::<u32>() {
                        if !stop_time_keys.contains(&(record_id, seq)) {
                            errors.push(
                                ValidationError::new(RULE_ID, SECTION, Severity::Error)
                                    .message(format!(
                                        "record_sub_id '{sub_id}' in translations.txt line {line} references non-existent stop_sequence for trip '{record_id}' in stop_times.txt"
                                    ))
                                    .file(FILE)
                                    .line(line)
                                    .field("record_sub_id")
                                    .value(sub_id),
                            );
                        }
                    } else {
                        errors.push(
                            ValidationError::new(RULE_ID, SECTION, Severity::Error)
                                .message(format!(
                                    "record_sub_id '{sub_id}' in translations.txt line {line} is not a valid stop_sequence integer"
                                ))
                                .file(FILE)
                                .line(line)
                                .field("record_sub_id")
                                .value(sub_id),
                        );
                    }
                }
            }
        }

        errors
    }
}
