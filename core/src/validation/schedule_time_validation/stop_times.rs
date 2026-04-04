//! Time sequence validation for `stop_times.txt` (section 7.1).

use std::collections::HashMap;

use crate::models::{GtfsFeed, StopTime};
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "stop_times.txt";
const SECTION: &str = "7";

/// Validates temporal ordering and consistency of stop times within each trip.
pub struct StopTimesTimeSequenceRule {
    max_trip_duration_secs: Option<u32>,
}

impl StopTimesTimeSequenceRule {
    #[must_use]
    pub fn new(max_trip_duration_hours: Option<u32>) -> Self {
        Self {
            max_trip_duration_secs: max_trip_duration_hours.map(|h| h * 3600),
        }
    }
}

impl ValidationRule for StopTimesTimeSequenceRule {
    fn rule_id(&self) -> &'static str {
        "time_sequence"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    #[allow(clippy::too_many_lines)]
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        let mut trips: HashMap<&str, Vec<(usize, &StopTime)>> = HashMap::new();
        for (i, st) in feed.stop_times.iter().enumerate() {
            trips.entry(st.trip_id.as_ref()).or_default().push((i, st));
        }

        for (trip_id, stops) in &trips {
            let mut sorted = stops.clone();
            sorted.sort_by_key(|&(_, st)| st.stop_sequence);

            for (pos, &(idx, st)) in sorted.iter().enumerate() {
                let line = idx + 2;

                if pos == 0 {
                    check_first_stop(&mut errors, st, trip_id, line);
                } else {
                    let (prev_idx, prev) = sorted[pos - 1];
                    check_consecutive(&mut errors, st, prev, trip_id, line, prev_idx + 2);
                }

                // departure_time must not be before arrival_time (same stop).
                if let (Some(arr), Some(dep)) = (st.arrival_time, st.departure_time)
                    && dep < arr
                {
                    errors.push(
                        ValidationError::new("departure_before_arrival", SECTION, Severity::Error)
                            .message(format!(
                                "departure_time {dep} is before arrival_time {arr} \
                             at stop_sequence {} in trip {trip_id}",
                                st.stop_sequence
                            ))
                            .file(FILE)
                            .line(line)
                            .field("departure_time")
                            .value(dep.to_string()),
                    );
                }
            }

            // Trip total duration must not exceed threshold.
            if let Some(max_secs) = self.max_trip_duration_secs {
                check_trip_duration(&mut errors, &sorted, trip_id, max_secs);
            }
        }

        errors
    }
}

/// First stop: `arrival_time` must equal `departure_time`.
fn check_first_stop(errors: &mut Vec<ValidationError>, st: &StopTime, trip_id: &str, line: usize) {
    if let (Some(arr), Some(dep)) = (st.arrival_time, st.departure_time)
        && arr != dep
    {
        errors.push(
            ValidationError::new("first_stop_times_differ", SECTION, Severity::Warning)
                .message(format!(
                    "first stop of trip {trip_id} has arrival_time {arr} \
                     different from departure_time {dep}"
                ))
                .file(FILE)
                .line(line)
                .field("arrival_time")
                .value(arr.to_string()),
        );
    }
}

/// Checks between consecutive stops: sequence, time ordering, distance.
fn check_consecutive(
    errors: &mut Vec<ValidationError>,
    st: &StopTime,
    prev: &StopTime,
    trip_id: &str,
    line: usize,
    prev_line: usize,
) {
    if st.stop_sequence <= prev.stop_sequence {
        errors.push(
            ValidationError::new("non_increasing_stop_sequence", SECTION, Severity::Error)
                .message(format!(
                    "stop_sequence {} at line {line} is not greater than \
                     stop_sequence {} at line {prev_line} in trip {trip_id}",
                    st.stop_sequence, prev.stop_sequence
                ))
                .file(FILE)
                .line(line)
                .field("stop_sequence")
                .value(st.stop_sequence.to_string()),
        );
    }

    // arrival_time must not decrease vs previous departure_time.
    if let (Some(arr), Some(prev_dep)) = (st.arrival_time, prev.departure_time)
        && arr < prev_dep
    {
        errors.push(
            ValidationError::new("decreasing_time", SECTION, Severity::Error)
                .message(format!(
                    "arrival_time {arr} at stop_sequence {} is before \
                     departure_time {prev_dep} at stop_sequence {}",
                    st.stop_sequence, prev.stop_sequence
                ))
                .file(FILE)
                .line(line)
                .field("arrival_time")
                .value(arr.to_string()),
        );
    }

    // shape_dist_traveled must not decrease.
    if let (Some(dist), Some(prev_dist)) = (st.shape_dist_traveled, prev.shape_dist_traveled)
        && dist < prev_dist
    {
        errors.push(
            ValidationError::new("decreasing_shape_dist", SECTION, Severity::Error)
                .message(format!(
                    "shape_dist_traveled {dist} at stop_sequence {} is less \
                     than {prev_dist} at stop_sequence {} in trip {trip_id}",
                    st.stop_sequence, prev.stop_sequence
                ))
                .file(FILE)
                .line(line)
                .field("shape_dist_traveled")
                .value(dist.to_string()),
        );
    }
}

/// Trip total duration must not exceed the configured threshold.
fn check_trip_duration(
    errors: &mut Vec<ValidationError>,
    sorted: &[(usize, &StopTime)],
    trip_id: &str,
    max_secs: u32,
) {
    let first_dep = sorted.first().and_then(|&(_, st)| st.departure_time);
    let last_arr = sorted.last().and_then(|&(_, st)| st.arrival_time);

    if let (Some(dep), Some(arr)) = (first_dep, last_arr) {
        let duration_secs = arr.total_seconds.saturating_sub(dep.total_seconds);
        if duration_secs > max_secs {
            let duration_h = duration_secs / 3600;
            let duration_m = (duration_secs % 3600) / 60;
            let threshold_h = max_secs / 3600;
            let last_idx = sorted.last().map_or(0, |&(i, _)| i);
            errors.push(
                ValidationError::new("trip_too_long", SECTION, Severity::Warning)
                    .message(format!(
                        "trip {trip_id} duration is {duration_h}h{duration_m:02} \
                         which exceeds the {threshold_h}h threshold"
                    ))
                    .file(FILE)
                    .line(last_idx + 2)
                    .field("arrival_time")
                    .value(arr.to_string()),
            );
        }
    }
}
