//! Tests for section 11 - locations.geojson semantic validation.

use std::collections::HashMap;

use gapline_core::models::{GeoJsonGeometry, GeoJsonLocation, GtfsFeed, Position};
use gapline_core::validation::ValidationRule;
use gapline_core::validation::locations_geojson_semantic::rules::{
    CoordinateOutOfRangeRule, DuplicateFeatureIdRule, MissingFeatureIdRule,
    MissingRecommendedPropertyRule, PolygonTooFewPointsRule, SelfIntersectingPolygonRule,
    UnclosedPolygonRule, UnsupportedGeometryTypeRule, ZeroAreaPolygonRule,
};

fn pos(lon: f64, lat: f64) -> Position {
    Position { lon, lat }
}

fn ring(points: &[(f64, f64)]) -> Vec<Position> {
    points.iter().map(|&(x, y)| pos(x, y)).collect()
}

fn loc(id: &str, geom: GeoJsonGeometry) -> GeoJsonLocation {
    let mut props = HashMap::new();
    props.insert(
        "stop_name".to_string(),
        serde_json::Value::String(format!("{id} name")),
    );
    GeoJsonLocation {
        id: id.to_string(),
        id_was_generated: false,
        geometry: geom,
        properties: props,
    }
}

fn polygon(rings: Vec<Vec<Position>>) -> GeoJsonGeometry {
    GeoJsonGeometry::Polygon { coordinates: rings }
}

fn closed_square(corner: f64) -> Vec<Position> {
    ring(&[
        (0.0, 0.0),
        (corner, 0.0),
        (corner, corner),
        (0.0, corner),
        (0.0, 0.0),
    ])
}

fn feed_with(locs: Vec<GeoJsonLocation>) -> GtfsFeed {
    GtfsFeed {
        geojson_locations: locs,
        ..GtfsFeed::default()
    }
}

// All rules instantiated and run against a feed; returns total errors.
fn run_all(feed: &GtfsFeed) -> Vec<gapline_core::validation::ValidationError> {
    let rules: Vec<Box<dyn ValidationRule>> = vec![
        Box::new(MissingFeatureIdRule),
        Box::new(DuplicateFeatureIdRule),
        Box::new(UnsupportedGeometryTypeRule),
        Box::new(PolygonTooFewPointsRule),
        Box::new(UnclosedPolygonRule),
        Box::new(CoordinateOutOfRangeRule),
        Box::new(SelfIntersectingPolygonRule),
        Box::new(ZeroAreaPolygonRule),
        Box::new(MissingRecommendedPropertyRule),
    ];
    rules.iter().flat_map(|r| r.validate(feed)).collect()
}

#[test]
fn valid_geojson_no_errors() {
    let feed = feed_with(vec![loc("zone-1", polygon(vec![closed_square(1.0)]))]);
    let errors = run_all(&feed);
    assert!(errors.is_empty(), "expected 0 errors, got {errors:#?}");
}

#[test]
fn feature_without_id() {
    let mut l = loc("ignored", polygon(vec![closed_square(1.0)]));
    l.id = "__autogen_0".to_string();
    l.id_was_generated = true;
    let feed = feed_with(vec![l]);
    let errors = MissingFeatureIdRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "missing_feature_id");
}

#[test]
fn duplicate_feature_id() {
    let feed = feed_with(vec![
        loc("zone-1", polygon(vec![closed_square(1.0)])),
        loc("zone-1", polygon(vec![closed_square(2.0)])),
    ]);
    let errors = DuplicateFeatureIdRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "duplicate_feature_id");
    assert_eq!(errors[0].value.as_deref(), Some("zone-1"));
}

#[test]
fn unsupported_geometry_point() {
    let feed = feed_with(vec![loc(
        "p1",
        GeoJsonGeometry::Unsupported {
            type_: "Point".to_string(),
        },
    )]);
    let errors = UnsupportedGeometryTypeRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "unsupported_geometry_type");
    assert!(errors[0].message.contains("Point"));
}

#[test]
fn unclosed_polygon() {
    let r = ring(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
    let feed = feed_with(vec![loc("z", polygon(vec![r]))]);
    let errors = UnclosedPolygonRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "unclosed_polygon");
}

#[test]
fn latitude_out_of_range() {
    let r = ring(&[(2.0, 91.0), (3.0, 0.0), (4.0, 0.0), (2.0, 91.0)]);
    let feed = feed_with(vec![loc("z", polygon(vec![r]))]);
    let errors = CoordinateOutOfRangeRule.validate(&feed);
    assert!(!errors.is_empty());
    assert!(
        errors
            .iter()
            .all(|e| e.rule_id == "coordinate_out_of_range")
    );
    assert!(errors[0].message.contains("91"));
}

#[test]
fn longitude_out_of_range() {
    let r = ring(&[(181.0, 48.0), (1.0, 0.0), (2.0, 0.0), (181.0, 48.0)]);
    let feed = feed_with(vec![loc("z", polygon(vec![r]))]);
    let errors = CoordinateOutOfRangeRule.validate(&feed);
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("181"));
}

#[test]
fn three_point_ring_reports_too_few_points_not_unclosed() {
    let r = ring(&[(0.0, 0.0), (1.0, 0.0), (0.5, 1.0)]);
    let feed = feed_with(vec![loc("z", polygon(vec![r]))]);
    let too_few = PolygonTooFewPointsRule.validate(&feed);
    let unclosed = UnclosedPolygonRule.validate(&feed);
    assert_eq!(too_few.len(), 1);
    assert_eq!(too_few[0].rule_id, "polygon_too_few_points");
    assert_eq!(unclosed.len(), 0);
}

#[test]
fn self_intersecting_polygon() {
    // Figure-8: segments (0,0)-(2,2) and (2,0)-(0,2) cross at (1,1).
    let r = ring(&[(0.0, 0.0), (2.0, 2.0), (2.0, 0.0), (0.0, 2.0), (0.0, 0.0)]);
    let feed = feed_with(vec![loc("z", polygon(vec![r]))]);
    let errors = SelfIntersectingPolygonRule.validate(&feed);
    assert_eq!(errors.len(), 1, "got {errors:#?}");
    assert_eq!(errors[0].rule_id, "self_intersecting_polygon");
}

#[test]
fn missing_stop_name() {
    let mut l = loc("z", polygon(vec![closed_square(1.0)]));
    l.properties.remove("stop_name");
    let feed = feed_with(vec![l]);
    let errors = MissingRecommendedPropertyRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "missing_recommended_property");
}

#[test]
fn zero_area_polygon() {
    let r = ring(&[(5.0, 5.0), (5.0, 5.0), (5.0, 5.0), (5.0, 5.0)]);
    let feed = feed_with(vec![loc("z", polygon(vec![r]))]);
    let errors = ZeroAreaPolygonRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "zero_area_polygon");
}

#[test]
fn feed_without_geojson_no_errors() {
    let feed = GtfsFeed::default();
    let errors = run_all(&feed);
    assert!(errors.is_empty(), "expected 0 errors, got {errors:#?}");
}

// Bonus: MultiPolygon iteration covers both polygons.
#[test]
fn multipolygon_unclosed_inner_polygon_reported() {
    let good = closed_square(1.0);
    let bad = ring(&[(5.0, 5.0), (6.0, 5.0), (6.0, 6.0), (5.0, 6.0)]); // unclosed
    let geom = GeoJsonGeometry::MultiPolygon {
        coordinates: vec![vec![good], vec![bad]],
    };
    let feed = feed_with(vec![loc("mp", geom)]);
    let errors = UnclosedPolygonRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("polygon[1]"));
}
