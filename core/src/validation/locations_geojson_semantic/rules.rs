use std::collections::HashMap;

use geo::algorithm::area::Area;
use geo::algorithm::intersects::Intersects;
use geo::{Coord, Line, LineString, Polygon};

use crate::models::{GeoJsonGeometry, GtfsFeed, LinearRing, Position};
use crate::validation::{Severity, ValidationError, ValidationRule};

const SECTION: &str = "11";
const FILE: &str = "locations.geojson";
const ZERO_AREA_EPSILON: f64 = 1e-12;

fn err(
    rule_id: &'static str,
    sev: Severity,
    feature_id: &str,
    msg: impl Into<String>,
) -> ValidationError {
    ValidationError::new(rule_id, SECTION, sev)
        .message(msg.into())
        .file(FILE)
        .field("feature_id")
        .value(feature_id)
}

/// Yields `(polygon_idx, ring_idx, ring)` for every ring in a Feature's
/// geometry. `polygon_idx` is `None` for `Polygon`, `Some(i)` for
/// `MultiPolygon`. `ring_idx == 0` is the outer ring.
fn for_each_ring<F: FnMut(Option<usize>, usize, &LinearRing)>(geom: &GeoJsonGeometry, mut f: F) {
    match geom {
        GeoJsonGeometry::Polygon { coordinates } => {
            for (i, ring) in coordinates.iter().enumerate() {
                f(None, i, ring);
            }
        }
        GeoJsonGeometry::MultiPolygon { coordinates } => {
            for (pi, polygon) in coordinates.iter().enumerate() {
                for (ri, ring) in polygon.iter().enumerate() {
                    f(Some(pi), ri, ring);
                }
            }
        }
        GeoJsonGeometry::Unsupported { .. } => {}
    }
}

fn ring_label(polygon_idx: Option<usize>, ring_idx: usize) -> String {
    match polygon_idx {
        Some(p) => format!("polygon[{p}].ring[{ring_idx}]"),
        None => format!("ring[{ring_idx}]"),
    }
}

fn to_coord(p: Position) -> Coord<f64> {
    Coord { x: p.lon, y: p.lat }
}

pub struct MissingFeatureIdRule;

impl ValidationRule for MissingFeatureIdRule {
    fn rule_id(&self) -> &'static str {
        "missing_feature_id"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for (idx, loc) in feed.geojson_locations.iter().enumerate() {
            if loc.id_was_generated {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Error)
                        .message(format!("Feature at index {idx} has no `id`"))
                        .file(FILE)
                        .field("id")
                        .value(idx.to_string()),
                );
            }
        }
        errors
    }
}

pub struct DuplicateFeatureIdRule;

impl ValidationRule for DuplicateFeatureIdRule {
    fn rule_id(&self) -> &'static str {
        "duplicate_feature_id"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for loc in &feed.geojson_locations {
            if loc.id_was_generated {
                continue;
            }
            *counts.entry(loc.id.as_str()).or_insert(0) += 1;
        }
        counts
            .into_iter()
            .filter(|&(_, c)| c > 1)
            .map(|(id, c)| {
                err(
                    self.rule_id(),
                    Severity::Error,
                    id,
                    format!("duplicate Feature id `{id}` ({c} occurrences)"),
                )
            })
            .collect()
    }
}

pub struct UnsupportedGeometryTypeRule;

impl ValidationRule for UnsupportedGeometryTypeRule {
    fn rule_id(&self) -> &'static str {
        "unsupported_geometry_type"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        feed.geojson_locations
            .iter()
            .filter_map(|loc| match &loc.geometry {
                GeoJsonGeometry::Unsupported { type_ } => Some(err(
                    self.rule_id(),
                    Severity::Error,
                    &loc.id,
                    format!("geometry type `{type_}` not allowed (only Polygon and MultiPolygon)"),
                )),
                _ => None,
            })
            .collect()
    }
}

pub struct PolygonTooFewPointsRule;

impl ValidationRule for PolygonTooFewPointsRule {
    fn rule_id(&self) -> &'static str {
        "polygon_too_few_points"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for loc in &feed.geojson_locations {
            for_each_ring(&loc.geometry, |pi, ri, ring| {
                if ring.len() < 4 {
                    errors.push(err(
                        self.rule_id(),
                        Severity::Error,
                        &loc.id,
                        format!(
                            "{} has {} points; a closed polygon ring requires at least 4",
                            ring_label(pi, ri),
                            ring.len()
                        ),
                    ));
                }
            });
        }
        errors
    }
}

pub struct UnclosedPolygonRule;

impl ValidationRule for UnclosedPolygonRule {
    fn rule_id(&self) -> &'static str {
        "unclosed_polygon"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for loc in &feed.geojson_locations {
            for_each_ring(&loc.geometry, |pi, ri, ring| {
                if ring.len() < 4 {
                    return;
                }
                let first = ring.first().copied();
                let last = ring.last().copied();
                if first != last {
                    errors.push(err(
                        self.rule_id(),
                        Severity::Error,
                        &loc.id,
                        format!(
                            "{} is not closed (first point {:?} ≠ last point {:?})",
                            ring_label(pi, ri),
                            first.map(|p| (p.lon, p.lat)),
                            last.map(|p| (p.lon, p.lat)),
                        ),
                    ));
                }
            });
        }
        errors
    }
}

pub struct CoordinateOutOfRangeRule;

impl ValidationRule for CoordinateOutOfRangeRule {
    fn rule_id(&self) -> &'static str {
        "coordinate_out_of_range"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for loc in &feed.geojson_locations {
            for_each_ring(&loc.geometry, |pi, ri, ring| {
                for (ci, pos) in ring.iter().enumerate() {
                    if !(-90.0..=90.0).contains(&pos.lat) || !(-180.0..=180.0).contains(&pos.lon) {
                        errors.push(err(
                            self.rule_id(),
                            Severity::Error,
                            &loc.id,
                            format!(
                                "{}.coord[{ci}] = [{}, {}] out of range",
                                ring_label(pi, ri),
                                pos.lon,
                                pos.lat,
                            ),
                        ));
                    }
                }
            });
        }
        errors
    }
}

pub struct SelfIntersectingPolygonRule;

impl SelfIntersectingPolygonRule {
    fn ring_self_intersects(ring: &[Position]) -> bool {
        if ring.len() < 4 {
            return false;
        }
        let lines: Vec<Line<f64>> = ring
            .windows(2)
            .map(|w| Line::new(to_coord(w[0]), to_coord(w[1])))
            .collect();
        let n = lines.len();
        for i in 0..n {
            // j starts at i+2 to skip the adjacent segment that legitimately
            // shares an endpoint. The closing pair (first, last segment of a
            // closed ring) also shares an endpoint, so skip it too.
            for j in (i + 2)..n {
                if i == 0 && j == n - 1 {
                    continue;
                }
                if lines[i].intersects(&lines[j]) {
                    return true;
                }
            }
        }
        false
    }
}

impl ValidationRule for SelfIntersectingPolygonRule {
    fn rule_id(&self) -> &'static str {
        "self_intersecting_polygon"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for loc in &feed.geojson_locations {
            for_each_ring(&loc.geometry, |pi, ri, ring| {
                if Self::ring_self_intersects(ring) {
                    errors.push(err(
                        self.rule_id(),
                        Severity::Warning,
                        &loc.id,
                        format!("{} self-intersects", ring_label(pi, ri)),
                    ));
                }
            });
        }
        errors
    }
}

pub struct ZeroAreaPolygonRule;

impl ValidationRule for ZeroAreaPolygonRule {
    fn rule_id(&self) -> &'static str {
        "zero_area_polygon"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for loc in &feed.geojson_locations {
            for_each_ring(&loc.geometry, |pi, ri, ring| {
                if ring.len() < 4 {
                    return;
                }
                let coords: Vec<Coord<f64>> = ring.iter().copied().map(to_coord).collect();
                let polygon = Polygon::new(LineString::new(coords), vec![]);
                if polygon.signed_area().abs() < ZERO_AREA_EPSILON {
                    errors.push(err(
                        self.rule_id(),
                        Severity::Warning,
                        &loc.id,
                        format!("{} has near-zero area", ring_label(pi, ri)),
                    ));
                }
            });
        }
        errors
    }
}

pub struct MissingRecommendedPropertyRule;

impl ValidationRule for MissingRecommendedPropertyRule {
    fn rule_id(&self) -> &'static str {
        "missing_recommended_property"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        feed.geojson_locations
            .iter()
            .filter(|loc| loc.stop_name().is_none())
            .map(|loc| {
                err(
                    self.rule_id(),
                    Severity::Warning,
                    &loc.id,
                    "recommended property `stop_name` missing",
                )
            })
            .collect()
    }
}
