//! GTFS-Realtime validation rule trait and shared context.
//!
//! Mirrors [`ValidationRule`](crate::validation::ValidationRule) but operates on
//! a [`GtfsRtFeed`] paired with an optional Schedule [`GtfsFeed`]. Cross-validation
//! rules use the prebuilt [`ScheduleIndex`] to test ID membership in O(1).

use std::collections::{HashMap, HashSet};

use crate::models::{GtfsFeed, rt::GtfsRtFeed};
use crate::validation::{Severity, ValidationError};

/// Indexed view of a GTFS Schedule feed used by RT cross-validation rules.
///
/// Built once per `validate_rt` call so every rule shares the same lookup
/// structures.
pub struct ScheduleIndex {
    pub trip_ids: HashSet<String>,
    pub stop_ids: HashSet<String>,
    pub route_ids: HashSet<String>,
    pub bbox: Option<BoundingBox>,
    /// `stop_id` → `location_type` (0 = `StopOrPlatform`). Stops with no
    /// `location_type` default to 0 per spec.
    pub stop_location_types: HashMap<String, u8>,
    /// `trip_id` → first `arrival_time` ordered by `stop_sequence`.
    pub trip_first_arrivals: HashMap<String, String>,
    /// Trips that visit the same `stop_id` at least twice in `stop_times.txt`.
    pub trip_repeated_stops: HashSet<String>,
    /// Trip ids covered by `frequencies.txt` — used to skip rules that only
    /// apply to scheduled (non-frequency) trips.
    pub trips_in_frequencies: HashSet<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

impl ScheduleIndex {
    #[must_use]
    pub fn from_feed(feed: &GtfsFeed) -> Self {
        let trip_ids = feed.trips.iter().map(|t| t.trip_id.to_string()).collect();
        let stop_ids = feed.stops.iter().map(|s| s.stop_id.to_string()).collect();
        let route_ids = feed.routes.iter().map(|r| r.route_id.to_string()).collect();

        let stop_location_types: HashMap<String, u8> = feed
            .stops
            .iter()
            .map(|s| {
                let lt = s.location_type.map_or(0, |l| l as u8);
                (s.stop_id.to_string(), lt)
            })
            .collect();

        let mut bbox: Option<BoundingBox> = None;
        for stop in &feed.stops {
            if let (Some(lat), Some(lon)) = (stop.stop_lat, stop.stop_lon) {
                let (lat, lon) = (lat.0, lon.0);
                bbox = Some(match bbox {
                    None => BoundingBox {
                        min_lat: lat,
                        max_lat: lat,
                        min_lon: lon,
                        max_lon: lon,
                    },
                    Some(b) => BoundingBox {
                        min_lat: b.min_lat.min(lat),
                        max_lat: b.max_lat.max(lat),
                        min_lon: b.min_lon.min(lon),
                        max_lon: b.max_lon.max(lon),
                    },
                });
            }
        }

        // Group stop_times by trip_id; within each group, find first arrival
        // by stop_sequence and detect repeated stop_ids.
        let mut by_trip: HashMap<String, Vec<&crate::models::StopTime>> = HashMap::new();
        for st in &feed.stop_times {
            by_trip.entry(st.trip_id.to_string()).or_default().push(st);
        }
        let mut trip_first_arrivals = HashMap::new();
        let mut trip_repeated_stops = HashSet::new();
        for (trip_id, mut sts) in by_trip {
            sts.sort_by_key(|s| s.stop_sequence);
            if let Some(first) = sts.iter().find(|s| s.arrival_time.is_some())
                && let Some(t) = first.arrival_time
            {
                trip_first_arrivals.insert(trip_id.clone(), t.to_string());
            }
            let mut seen: HashSet<String> = HashSet::new();
            for st in &sts {
                if !seen.insert(st.stop_id.to_string()) {
                    trip_repeated_stops.insert(trip_id.clone());
                    break;
                }
            }
        }

        let trips_in_frequencies = feed
            .frequencies
            .iter()
            .map(|f| f.trip_id.to_string())
            .collect();

        Self {
            trip_ids,
            stop_ids,
            route_ids,
            bbox,
            stop_location_types,
            trip_first_arrivals,
            trip_repeated_stops,
            trips_in_frequencies,
        }
    }
}

pub struct RtValidationContext<'a> {
    pub rt: &'a GtfsRtFeed,
    pub schedule: Option<&'a GtfsFeed>,
    pub schedule_index: Option<&'a ScheduleIndex>,
    pub now_unix: u64,
    pub max_delay_seconds: u32,
}

pub trait RtValidationRule: Send + Sync {
    fn rule_id(&self) -> &'static str;
    fn section(&self) -> &'static str;
    fn severity(&self) -> Severity;
    fn validate(&self, ctx: &RtValidationContext<'_>) -> Vec<ValidationError>;
}
