//! Speed validation between consecutive stops (section 7.11).
//!
//! For each trip, computes the implied travel speed between consecutive stops
//! using Haversine distance and elapsed time, then flags speeds that exceed
//! the configurable route-type threshold or are zero despite different coords.

use std::collections::HashMap;

use crate::geo::haversine_meters;
use crate::models::GtfsFeed;
use crate::validation::schedule_time_validation::SpeedThresholds;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "stop_times.txt";
const SECTION: &str = "7";

/// Threshold to avoid floating-point false positives on `zero_speed`.
const MIN_DISTANCE_FOR_ZERO_SPEED_M: f64 = 1.0;

/// Validates inter-stop travel speeds against configurable route-type
/// thresholds.
pub struct SpeedValidationRule {
    thresholds: SpeedThresholds,
}

impl SpeedValidationRule {
    #[must_use]
    pub fn new(thresholds: SpeedThresholds) -> Self {
        Self { thresholds }
    }
}

impl ValidationRule for SpeedValidationRule {
    fn rule_id(&self) -> &'static str {
        "speed_validation"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let stops_by_id: HashMap<&str, _> =
            feed.stops.iter().map(|s| (s.stop_id.as_ref(), s)).collect();

        let routes_by_id: HashMap<&str, _> = feed
            .routes
            .iter()
            .map(|r| (r.route_id.as_ref(), r))
            .collect();

        let trips_by_id: HashMap<&str, _> =
            feed.trips.iter().map(|t| (t.trip_id.as_ref(), t)).collect();

        let mut trip_stop_times: HashMap<&str, Vec<(usize, _)>> = HashMap::new();
        for (i, st) in feed.stop_times.iter().enumerate() {
            trip_stop_times
                .entry(st.trip_id.as_ref())
                .or_default()
                .push((i, st));
        }

        let mut errors = Vec::new();

        for (trip_id, stop_times) in &trip_stop_times {
            let Some(trip) = trips_by_id.get(trip_id) else {
                continue;
            };
            let Some(route) = routes_by_id.get(trip.route_id.as_ref()) else {
                continue;
            };
            let max_speed_kmh = self.thresholds.limit_for(&route.route_type);

            let mut sorted: Vec<_> = stop_times.clone();
            sorted.sort_by_key(|&(_, st)| st.stop_sequence);

            for pair in sorted.windows(2) {
                let (_, prev_st) = pair[0];
                let (curr_idx, curr_st) = pair[1];
                let curr_line = curr_idx + 2;

                let (Some(prev_dep), Some(curr_arr)) =
                    (prev_st.departure_time, curr_st.arrival_time)
                else {
                    continue;
                };

                let Some(prev_stop) = stops_by_id.get(prev_st.stop_id.as_ref()) else {
                    continue;
                };
                let Some(curr_stop) = stops_by_id.get(curr_st.stop_id.as_ref()) else {
                    continue;
                };
                let (Some(lat1), Some(lon1)) = (prev_stop.stop_lat, prev_stop.stop_lon) else {
                    continue;
                };
                let (Some(lat2), Some(lon2)) = (curr_stop.stop_lat, curr_stop.stop_lon) else {
                    continue;
                };

                let dist_m = haversine_meters(lat1.0, lon1.0, lat2.0, lon2.0);
                let time_secs = curr_arr
                    .total_seconds
                    .saturating_sub(prev_dep.total_seconds);

                if time_secs == 0 {
                    if dist_m > MIN_DISTANCE_FOR_ZERO_SPEED_M {
                        errors.push(
                            ValidationError::new("zero_speed", SECTION, Severity::Warning)
                                .message(format!(
                                    "Speed is 0 between stops '{}' and '{}' in trip \
                                     '{trip_id}' (distance {dist_m:.1}m, time difference 0s)",
                                    prev_st.stop_id, curr_st.stop_id,
                                ))
                                .file(FILE)
                                .line(curr_line)
                                .field("arrival_time")
                                .value(curr_arr.to_string()),
                        );
                    }
                    continue;
                }

                let speed_kmh = (dist_m / 1000.0) / (f64::from(time_secs) / 3600.0);

                if speed_kmh > max_speed_kmh {
                    errors.push(
                        ValidationError::new("unrealistic_speed", SECTION, Severity::Warning)
                            .message(format!(
                                "Speed {speed_kmh:.1} km/h between stops '{}' and '{}' in \
                             trip '{trip_id}' exceeds {max_speed_kmh} km/h limit for \
                             route_type {} (distance {dist_m:.1}m, time {time_secs}s)",
                                prev_st.stop_id,
                                curr_st.stop_id,
                                route.route_type.to_i32(),
                            ))
                            .file(FILE)
                            .line(curr_line)
                            .field("arrival_time")
                            .value(curr_arr.to_string()),
                    );
                }
            }
        }

        errors
    }
}
