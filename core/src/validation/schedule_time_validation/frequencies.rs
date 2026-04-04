//! Frequency coherence validation for `frequencies.txt` (section 7.2).

use std::collections::HashMap;

use crate::models::{Frequency, GtfsFeed};
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "frequencies.txt";
const SECTION: &str = "7";

/// Validates coherence of frequency entries.
pub struct FrequenciesCoherenceRule;

impl ValidationRule for FrequenciesCoherenceRule {
    fn rule_id(&self) -> &'static str {
        "frequencies_coherence"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        let mut by_trip: HashMap<&str, Vec<(usize, &Frequency)>> = HashMap::new();

        for (i, freq) in feed.frequencies.iter().enumerate() {
            let line = i + 2;

            if freq.headway_secs == 0 {
                errors.push(
                    ValidationError::new("invalid_headway", SECTION, Severity::Error)
                        .message("headway_secs must be greater than 0")
                        .file(FILE)
                        .line(line)
                        .field("headway_secs")
                        .value("0"),
                );
            }

            if freq.start_time >= freq.end_time {
                errors.push(
                    ValidationError::new("invalid_time_range", SECTION, Severity::Error)
                        .message(format!(
                            "start_time {} is not before end_time {}",
                            freq.start_time, freq.end_time
                        ))
                        .file(FILE)
                        .line(line)
                        .field("start_time")
                        .value(freq.start_time.to_string()),
                );
            }

            by_trip
                .entry(freq.trip_id.as_ref())
                .or_default()
                .push((i, freq));
        }

        // Check for overlapping ranges within each trip.
        for (trip_id, freqs) in &by_trip {
            if freqs.len() < 2 {
                continue;
            }
            let mut sorted = freqs.clone();
            sorted.sort_by_key(|&(_, f)| f.start_time);

            for pair in sorted.windows(2) {
                let (_, prev) = pair[0];
                let (idx, curr) = pair[1];
                if curr.start_time < prev.end_time {
                    errors.push(
                        ValidationError::new("overlapping_frequencies", SECTION, Severity::Error)
                            .message(format!(
                                "frequency range {}-{} overlaps with {}-{} for trip {trip_id}",
                                curr.start_time, curr.end_time, prev.start_time, prev.end_time
                            ))
                            .file(FILE)
                            .line(idx + 2)
                            .field("start_time")
                            .value(curr.start_time.to_string()),
                    );
                }
            }
        }

        errors
    }
}
