//! Stop-to-shape distance validation (section 7.4).
//!
//! For each trip that references a shape, checks that every stop of the trip
//! is within `max_stop_to_shape_distance_m` of the nearest shape point.

use std::collections::HashMap;

use crate::geo::haversine_meters;
use crate::models::{GtfsFeed, Shape, Stop};
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "stop_times.txt";
const SECTION: &str = "7";

/// Validates that every stop lies within a configurable distance of its
/// trip's shape. Trips without a shape reference are skipped.
pub struct StopToShapeDistanceRule {
    max_distance_m: f64,
}

impl StopToShapeDistanceRule {
    #[must_use]
    pub fn new(max_distance_m: f64) -> Self {
        Self { max_distance_m }
    }
}

impl ValidationRule for StopToShapeDistanceRule {
    fn rule_id(&self) -> &'static str {
        "stop_too_far_from_shape"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        // Group shape points by shape_id, sorted by sequence. Points with no
        // coords cannot occur (lat/lon are required in shapes.txt).
        let mut shapes_by_id: HashMap<&str, Vec<&Shape>> = HashMap::new();
        for shape in &feed.shapes {
            shapes_by_id
                .entry(shape.shape_id.as_ref())
                .or_default()
                .push(shape);
        }
        for pts in shapes_by_id.values_mut() {
            pts.sort_by_key(|s| s.shape_pt_sequence);
        }

        // Index stops by id for O(1) coord lookup.
        let stops_by_id: HashMap<&str, &Stop> =
            feed.stops.iter().map(|s| (s.stop_id.as_ref(), s)).collect();

        // Index trips that carry a non-empty shape_id.
        let trip_shape: HashMap<&str, &str> = feed
            .trips
            .iter()
            .filter_map(|t| {
                let sid = t.shape_id.as_ref()?.as_ref();
                if sid.is_empty() {
                    None
                } else {
                    Some((t.trip_id.as_ref(), sid))
                }
            })
            .collect();

        // Cache the min distance per (stop_id, shape_id) to avoid recomputing
        // across trips that share the same shape and stops.
        let mut cache: HashMap<(&str, &str), f64> = HashMap::new();
        let mut errors = Vec::new();

        for (i, st) in feed.stop_times.iter().enumerate() {
            let trip_id = st.trip_id.as_ref();
            let Some(&shape_id) = trip_shape.get(trip_id) else {
                continue;
            };
            let Some(points) = shapes_by_id.get(shape_id) else {
                continue; // covered by FK rule
            };
            if points.is_empty() {
                continue;
            }
            let stop_id = st.stop_id.as_ref();
            let Some(stop) = stops_by_id.get(stop_id) else {
                continue; // covered by FK rule
            };
            let (Some(lat), Some(lon)) = (stop.stop_lat, stop.stop_lon) else {
                continue;
            };

            let key = (stop_id, shape_id);
            let min_dist = *cache
                .entry(key)
                .or_insert_with(|| min_distance_to_shape(lat.0, lon.0, points));

            if min_dist > self.max_distance_m {
                let nearest = nearest_point(lat.0, lon.0, points);
                errors.push(
                    ValidationError::new("stop_too_far_from_shape", SECTION, Severity::Warning)
                        .message(format!(
                            "stop '{stop_id}' in trip '{trip_id}' is {min_dist:.1}m from its shape \
                             '{shape_id}' (threshold {:.1}m); stop=({}, {}) nearest_point=({}, {})",
                            self.max_distance_m,
                            lat.0,
                            lon.0,
                            nearest.shape_pt_lat.0,
                            nearest.shape_pt_lon.0,
                        ))
                        .file(FILE)
                        .line(i + 2)
                        .field("stop_id")
                        .value(stop_id),
                );
            }
        }

        errors
    }
}

fn min_distance_to_shape(lat: f64, lon: f64, points: &[&Shape]) -> f64 {
    points
        .iter()
        .map(|p| haversine_meters(lat, lon, p.shape_pt_lat.0, p.shape_pt_lon.0))
        .fold(f64::INFINITY, f64::min)
}

fn nearest_point<'a>(lat: f64, lon: f64, points: &'a [&'a Shape]) -> &'a Shape {
    points
        .iter()
        .min_by(|a, b| {
            let da = haversine_meters(lat, lon, a.shape_pt_lat.0, a.shape_pt_lon.0);
            let db = haversine_meters(lat, lon, b.shape_pt_lat.0, b.shape_pt_lon.0);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
        .expect("caller ensures points is non-empty")
}
