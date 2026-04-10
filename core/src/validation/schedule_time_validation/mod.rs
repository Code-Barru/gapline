//! Schedule time validation (section 7).
//!
//! Validates temporal consistency of `stop_times.txt` (7.1) and
//! `frequencies.txt` (7.2), shape geometry (7.3), stop-to-shape distances
//! (7.4), calendar date ranges and feed coverage (7.5), `calendar_dates`
//! coherence plus trip activity (7.6), stop hierarchy (7.7), route type
//! semantics (7.8), transfer validation (7.9), pathway validation (7.10),
//! speed validation (7.11), `block_id` overlap (7.12), coordinate sanity
//! (7.13), and unused entity detection.

pub mod block_id;
pub mod calendar_dates_coherence;
pub mod calendar_ranges;
pub mod coordinates;
pub mod distances;
pub mod feed_coverage;
pub mod frequencies;
pub mod pathways;
pub mod route_type_semantics;
pub mod service_dates;
pub mod shapes;
pub mod speed;
pub mod stop_hierarchy;
pub mod stop_times;
pub mod transfers;
pub mod trip_activity;
pub mod unused_entities;

use std::sync::Arc;

use crate::models::GtfsDate;
use crate::models::RouteType;
use crate::validation::engine::ValidationEngine;

use self::service_dates::ServiceDateCache;

/// Distance and coordinate thresholds used by the section-7 geometric rules.
#[derive(Debug, Clone, Copy)]
pub struct DistanceThresholds {
    pub max_stop_to_shape_distance_m: f64,
    pub min_shape_point_distance_m: f64,
    pub shape_dist_incoherence_ratio: f64,
    pub min_distance_from_origin_m: f64,
    pub min_distance_from_poles_m: f64,
}

/// Calendar-coherence thresholds used by the section-7 calendar rules.
#[derive(Debug, Clone, Copy)]
pub struct CalendarThresholds {
    pub min_feed_coverage_days: u32,
    pub feed_expiration_warning_days: i64,
    pub min_trip_activity_days: u32,
    pub reference_date: Option<GtfsDate>,
}

/// Transfer distance thresholds used by the section-7 transfer rules.
#[derive(Debug, Clone, Copy)]
pub struct TransferThresholds {
    pub max_transfer_distance_m: f64,
    pub transfer_distance_warning_m: f64,
}

/// Speed thresholds used by the section-7 speed validation rule.
/// Each field is a maximum speed in km/h for the corresponding route type.
/// Extended types ([`RouteType::Hvt`], [`RouteType::Unknown`]) fall back
/// to `default_kmh`.
#[derive(Debug, Clone, Copy)]
pub struct SpeedThresholds {
    pub tram_kmh: f64,
    pub subway_kmh: f64,
    pub rail_kmh: f64,
    pub bus_kmh: f64,
    pub ferry_kmh: f64,
    pub cable_tram_kmh: f64,
    pub aerial_lift_kmh: f64,
    pub funicular_kmh: f64,
    pub trolleybus_kmh: f64,
    pub monorail_kmh: f64,
    pub default_kmh: f64,
}

impl SpeedThresholds {
    /// Returns the speed limit in km/h for the given route type.
    #[must_use]
    pub fn limit_for(&self, route_type: &RouteType) -> f64 {
        match route_type {
            RouteType::Tram => self.tram_kmh,
            RouteType::Subway => self.subway_kmh,
            RouteType::Rail => self.rail_kmh,
            RouteType::Bus => self.bus_kmh,
            RouteType::Ferry => self.ferry_kmh,
            RouteType::CableTram => self.cable_tram_kmh,
            RouteType::AerialLift => self.aerial_lift_kmh,
            RouteType::Funicular => self.funicular_kmh,
            RouteType::Trolleybus => self.trolleybus_kmh,
            RouteType::Monorail => self.monorail_kmh,
            RouteType::Hvt(_) | RouteType::Unknown(_) => self.default_kmh,
        }
    }
}

pub fn register_rules(
    engine: &mut ValidationEngine,
    max_trip_duration_hours: Option<u32>,
    distances: DistanceThresholds,
    calendar: CalendarThresholds,
    transfer: TransferThresholds,
    speed: SpeedThresholds,
    service_cache: Arc<ServiceDateCache>,
) {
    engine.register_rule(Box::new(stop_times::StopTimesTimeSequenceRule::new(
        max_trip_duration_hours,
    )));
    engine.register_rule(Box::new(frequencies::FrequenciesCoherenceRule));
    engine.register_rule(Box::new(shapes::ShapesGeometryRule::new(
        distances.min_shape_point_distance_m,
        distances.shape_dist_incoherence_ratio,
    )));
    engine.register_rule(Box::new(distances::StopToShapeDistanceRule::new(
        distances.max_stop_to_shape_distance_m,
    )));
    engine.register_rule(Box::new(calendar_ranges::CalendarRangesRule));
    engine.register_rule(Box::new(feed_coverage::FeedCoverageRule::new(
        calendar.min_feed_coverage_days,
        calendar.feed_expiration_warning_days,
        calendar.reference_date,
    )));
    engine.register_rule(Box::new(
        calendar_dates_coherence::CalendarDatesCoherenceRule,
    ));
    engine.register_rule(Box::new(trip_activity::TripActivityRule::new(
        calendar.min_trip_activity_days,
        service_cache.clone(),
    )));
    engine.register_rule(Box::new(stop_hierarchy::InvalidParentTypeRule));
    engine.register_rule(Box::new(stop_hierarchy::UnusedStationRule));
    engine.register_rule(Box::new(stop_hierarchy::UnusedStopRule));
    engine.register_rule(Box::new(route_type_semantics::RouteTypeSemanticsRule));
    engine.register_rule(Box::new(transfers::TransferValidationRule::new(transfer)));
    engine.register_rule(Box::new(pathways::PathwayValidationRule));
    engine.register_rule(Box::new(speed::SpeedValidationRule::new(speed)));
    // 7.12
    engine.register_rule(Box::new(block_id::BlockIdTripOverlapRule::new(
        service_cache,
    )));
    // 7.13
    engine.register_rule(Box::new(coordinates::CoordinatesNearOriginRule::new(
        distances.min_distance_from_origin_m,
    )));
    engine.register_rule(Box::new(coordinates::CoordinatesNearPoleRule::new(
        distances.min_distance_from_poles_m,
    )));
    engine.register_rule(Box::new(coordinates::DuplicateCoordinatesRule));
    // Unused entities
    engine.register_rule(Box::new(unused_entities::UnusedRouteRule));
    engine.register_rule(Box::new(unused_entities::UnusedShapeRule));
    engine.register_rule(Box::new(unused_entities::UnusedServiceRule));
    engine.register_rule(Box::new(unused_entities::UnusedAgencyRule));
    engine.register_rule(Box::new(unused_entities::UnusedFareRule));
}
