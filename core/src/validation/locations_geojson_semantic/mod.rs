//! Section 11 - locations.geojson semantic rules.
//!
//! Validates structure, properties and geometry of GTFS-Locations Features
//! consumed by GTFS-Flex (`stop_times.stop_id` may reference a Feature `id`).

use crate::validation::engine::ValidationEngine;

pub mod rules;

pub fn register_rules(engine: &mut ValidationEngine) {
    engine.register_rule(Box::new(rules::MissingFeatureIdRule));
    engine.register_rule(Box::new(rules::DuplicateFeatureIdRule));
    engine.register_rule(Box::new(rules::UnsupportedGeometryTypeRule));
    engine.register_rule(Box::new(rules::PolygonTooFewPointsRule));
    engine.register_rule(Box::new(rules::UnclosedPolygonRule));
    engine.register_rule(Box::new(rules::CoordinateOutOfRangeRule));
    engine.register_rule(Box::new(rules::SelfIntersectingPolygonRule));
    engine.register_rule(Box::new(rules::ZeroAreaPolygonRule));
    engine.register_rule(Box::new(rules::MissingRecommendedPropertyRule));
}
