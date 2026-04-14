//! Coordinate sanity validation (section 7.13).
//!
//! Detects stops whose coordinates are suspiciously close to the geographic
//! origin (0, 0), near the poles, or identical across different stops.

use std::collections::HashMap;

use crate::geo::haversine_meters;
use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "stops.txt";
const SECTION: &str = "7";

/// Flags stops whose coordinates are suspiciously close to the geographic
/// origin (0, 0).
pub struct CoordinatesNearOriginRule {
    min_distance_m: f64,
}

impl CoordinatesNearOriginRule {
    #[must_use]
    pub fn new(min_distance_m: f64) -> Self {
        Self { min_distance_m }
    }
}

impl ValidationRule for CoordinatesNearOriginRule {
    fn rule_id(&self) -> &'static str {
        "coordinates_near_origin"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for (i, stop) in feed.stops.iter().enumerate() {
            let (Some(lat), Some(lon)) = (stop.stop_lat, stop.stop_lon) else {
                continue;
            };
            let dist = haversine_meters(lat.0, lon.0, 0.0, 0.0);
            if dist < self.min_distance_m {
                errors.push(
                    ValidationError::new("coordinates_near_origin", SECTION, Severity::Warning)
                        .message(format!(
                            "stop '{}' is {dist:.0}m from the origin (0, 0), \
                             below the {:.0}m threshold",
                            stop.stop_id, self.min_distance_m,
                        ))
                        .file(FILE)
                        .line(i + 2)
                        .field("stop_lat")
                        .value(format!("{},{}", lat.0, lon.0)),
                );
            }
        }
        errors
    }
}

/// Flags stops whose coordinates are suspiciously close to the North or
/// South Pole.
pub struct CoordinatesNearPoleRule {
    min_distance_m: f64,
}

impl CoordinatesNearPoleRule {
    #[must_use]
    pub fn new(min_distance_m: f64) -> Self {
        Self { min_distance_m }
    }
}

impl ValidationRule for CoordinatesNearPoleRule {
    fn rule_id(&self) -> &'static str {
        "coordinates_near_pole"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for (i, stop) in feed.stops.iter().enumerate() {
            let (Some(lat), Some(lon)) = (stop.stop_lat, stop.stop_lon) else {
                continue;
            };
            let to_north = haversine_meters(lat.0, lon.0, 90.0, 0.0);
            let to_south = haversine_meters(lat.0, lon.0, -90.0, 0.0);
            let min_dist = to_north.min(to_south);
            if min_dist < self.min_distance_m {
                let pole = if to_north < to_south {
                    "North"
                } else {
                    "South"
                };
                errors.push(
                    ValidationError::new("coordinates_near_pole", SECTION, Severity::Warning)
                        .message(format!(
                            "stop '{}' is {min_dist:.0}m from the {pole} Pole, \
                             below the {:.0}m threshold",
                            stop.stop_id, self.min_distance_m,
                        ))
                        .file(FILE)
                        .line(i + 2)
                        .field("stop_lat")
                        .value(format!("{},{}", lat.0, lon.0)),
                );
            }
        }
        errors
    }
}

/// Flags groups of stops that share identical coordinates.
pub struct DuplicateCoordinatesRule;

impl ValidationRule for DuplicateCoordinatesRule {
    fn rule_id(&self) -> &'static str {
        "duplicate_coordinates"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut by_coords: HashMap<(u64, u64), Vec<(usize, &str)>> = HashMap::new();
        for (i, stop) in feed.stops.iter().enumerate() {
            let (Some(lat), Some(lon)) = (stop.stop_lat, stop.stop_lon) else {
                continue;
            };
            let key = (lat.0.to_bits(), lon.0.to_bits());
            by_coords
                .entry(key)
                .or_default()
                .push((i, stop.stop_id.as_ref()));
        }

        let mut errors = Vec::new();
        for ((lat_bits, lon_bits), entries) in &by_coords {
            if entries.len() < 2 {
                continue;
            }
            let lat = f64::from_bits(*lat_bits);
            let lon = f64::from_bits(*lon_bits);
            let first_idx = entries[0].0;
            let ids: Vec<&str> = entries.iter().map(|(_, id)| *id).collect();
            errors.push(
                ValidationError::new("duplicate_coordinates", SECTION, Severity::Warning)
                    .message(format!(
                        "stops {} share identical coordinates ({}, {})",
                        ids.join(", "),
                        lat,
                        lon,
                    ))
                    .file(FILE)
                    .line(first_idx + 2)
                    .field("stop_lat")
                    .value(format!("{lat},{lon}")),
            );
        }
        errors
    }
}
