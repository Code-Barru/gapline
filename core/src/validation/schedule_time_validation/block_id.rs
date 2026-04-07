//! Block ID overlap validation (section 7.12).
//!
//! Detects trips sharing the same `block_id` whose time ranges overlap on
//! common service days. A single vehicle (block) cannot operate two trips
//! simultaneously.

use std::collections::HashMap;
use std::sync::Arc;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

use super::service_dates::ServiceDateCache;

const FILE: &str = "trips.txt";
const SECTION: &str = "7";

pub struct BlockIdTripOverlapRule {
    cache: Arc<ServiceDateCache>,
}

impl BlockIdTripOverlapRule {
    #[must_use]
    pub fn new(cache: Arc<ServiceDateCache>) -> Self {
        Self { cache }
    }
}

struct TripTimeRange<'a> {
    trip_id: &'a str,
    service_id: &'a str,
    start: u32,
    end: u32,
    line: usize,
}

impl ValidationRule for BlockIdTripOverlapRule {
    fn rule_id(&self) -> &'static str {
        "block_id_trip_overlap"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let active_dates = self.cache.get(feed);

        let mut trip_times: HashMap<&str, (u32, u32)> = HashMap::new();
        {
            let mut trip_stop_times: HashMap<&str, Vec<&crate::models::StopTime>> = HashMap::new();
            for st in &feed.stop_times {
                trip_stop_times
                    .entry(st.trip_id.as_ref())
                    .or_default()
                    .push(st);
            }
            for (tid, mut stops) in trip_stop_times {
                stops.sort_by_key(|st| st.stop_sequence);
                let first_dep = stops.first().and_then(|s| s.departure_time);
                let last_arr = stops.last().and_then(|s| s.arrival_time);
                if let (Some(dep), Some(arr)) = (first_dep, last_arr) {
                    trip_times.insert(tid, (dep.total_seconds, arr.total_seconds));
                }
            }
        }

        let mut blocks: HashMap<&str, Vec<TripTimeRange>> = HashMap::new();
        for (i, trip) in feed.trips.iter().enumerate() {
            let Some(ref block_id) = trip.block_id else {
                continue;
            };
            let Some(&(start, end)) = trip_times.get(trip.trip_id.as_ref()) else {
                continue;
            };
            blocks
                .entry(block_id.as_str())
                .or_default()
                .push(TripTimeRange {
                    trip_id: trip.trip_id.as_ref(),
                    service_id: trip.service_id.as_ref(),
                    start,
                    end,
                    line: i + 2,
                });
        }

        let mut errors = Vec::new();
        for (block_id, trips) in &blocks {
            for (i, a) in trips.iter().enumerate() {
                for b in &trips[i + 1..] {
                    // >= (not >) so adjacent boundaries don't count as overlap.
                    if a.start >= b.end || b.start >= a.end {
                        continue;
                    }
                    let dates_a = active_dates.get(a.service_id);
                    let dates_b = active_dates.get(b.service_id);
                    let has_common_day = match (dates_a, dates_b) {
                        (Some(sa), Some(sb)) => sa.intersection(sb).next().is_some(),
                        _ => false,
                    };
                    if !has_common_day {
                        continue;
                    }
                    errors.push(
                        ValidationError::new("block_id_trip_overlap", SECTION, Severity::Error)
                            .message(format!(
                                "trips '{}' [{}-{}] and '{}' [{}-{}] share block_id '{}' \
                                 and overlap on common service days",
                                a.trip_id,
                                format_seconds(a.start),
                                format_seconds(a.end),
                                b.trip_id,
                                format_seconds(b.start),
                                format_seconds(b.end),
                                block_id,
                            ))
                            .file(FILE)
                            .line(a.line)
                            .field("block_id")
                            .value(block_id.to_string()),
                    );
                }
            }
        }

        errors
    }
}

fn format_seconds(secs: u32) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    format!("{h:02}:{m:02}")
}
