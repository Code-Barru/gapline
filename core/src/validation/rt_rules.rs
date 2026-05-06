//! GTFS-Realtime validation rule trait and shared context.
//!
//! Mirrors [`ValidationRule`](crate::validation::ValidationRule) but operates on
//! a [`GtfsRtFeed`] paired with an optional Schedule [`GtfsFeed`]. Cross-validation
//! rules use the prebuilt [`ScheduleIndex`] to test ID membership in O(1).

use std::collections::HashSet;

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

        Self {
            trip_ids,
            stop_ids,
            route_ids,
            bbox,
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
