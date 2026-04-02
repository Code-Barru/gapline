//! Field definition validation for `stop_times.txt`.

use std::collections::HashMap;

use crate::models::GtfsFeed;
use crate::models::Timepoint;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "stop_times.txt";
const SECTION: &str = "4";
const RULE_ID: &str = "field_definition_stop_times";

/// Validates conditional field constraints for `stop_times.txt`.
///
/// - `arrival_time` and `departure_time` are required for the first and last
///   stop of each trip.
/// - `arrival_time` and `departure_time` are required when `timepoint` is
///   `Exact` (1).
pub struct StopTimesFieldDefinitionRule;

impl ValidationRule for StopTimesFieldDefinitionRule {
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

        let mut trips_map: HashMap<&str, Vec<(usize, u32)>> = HashMap::new();
        for (i, st) in feed.stop_times.iter().enumerate() {
            trips_map
                .entry(st.trip_id.as_ref())
                .or_default()
                .push((i, st.stop_sequence));
        }

        for indices in trips_map.values() {
            let mut sorted = indices.clone();
            sorted.sort_by_key(|&(_, seq)| seq);

            let first_idx = sorted.first().map(|&(i, _)| i);
            let last_idx = if sorted.len() > 1 {
                sorted.last().map(|&(i, _)| i)
            } else {
                None
            };

            for &(i, _) in &sorted {
                let st = &feed.stop_times[i];
                let line = i + 2;
                let is_first_or_last = Some(i) == first_idx || Some(i) == last_idx;
                let is_exact_timepoint = st.timepoint == Some(Timepoint::Exact);

                if is_first_or_last || is_exact_timepoint {
                    if st.arrival_time.is_none() {
                        let reason = if is_first_or_last {
                            "arrival_time is required for the first and last stop of a trip"
                        } else {
                            "arrival_time is required when timepoint=1"
                        };
                        errors.push(
                            ValidationError::new(RULE_ID, SECTION, Severity::Error)
                                .message(reason)
                                .file(FILE)
                                .line(line)
                                .field("arrival_time"),
                        );
                    }
                    if st.departure_time.is_none() {
                        let reason = if is_first_or_last {
                            "departure_time is required for the first and last stop of a trip"
                        } else {
                            "departure_time is required when timepoint=1"
                        };
                        errors.push(
                            ValidationError::new(RULE_ID, SECTION, Severity::Error)
                                .message(reason)
                                .file(FILE)
                                .line(line)
                                .field("departure_time"),
                        );
                    }
                }
            }
        }

        errors
    }
}
