//! Section 5 — Foreign Key Validation.
//!
//! Detects orphan references between GTFS files. Each rule checks that
//! foreign-key values in a source file point to existing primary keys in the
//! target file. Violations produce `foreign_key_violation` errors with full
//! context (file, line, field, orphan value).

#[macro_use]
mod macros;

// Core FK rules (HW-017)
pub mod calendar_dates_service;
pub mod frequencies_trip;
pub mod routes_agency;
pub mod stop_times_stop;
pub mod stop_times_trip;
pub mod stops_level;
pub mod stops_parent_station;
pub mod trips_route;
pub mod trips_service;
pub mod trips_shape;

// Extended FK rules (HW-018)
pub mod attributions_refs;
pub mod fare_attributes_agency;
pub mod fare_rules_fare;
pub mod fare_rules_route;
pub mod fare_rules_zones;
pub mod pathways_stops;
pub mod transfers_from_route;
pub mod transfers_from_stop;
pub mod transfers_from_trip;
pub mod transfers_to_route;
pub mod transfers_to_stop;
pub mod transfers_to_trip;
pub mod translations_record;

// Fares v2 FK rules (HW-E26)
pub mod fare_leg_join_rules;
pub mod fare_leg_rules_areas;
pub mod fare_leg_rules_network;
pub mod fare_leg_rules_product;
pub mod fare_leg_rules_timeframes;
pub mod fare_products_media;
pub mod fare_products_rider;
pub mod fare_transfer_rules_legs;
pub mod fare_transfer_rules_product;
pub mod route_networks;
pub mod stop_areas;
pub mod timeframes_service;

use crate::validation::engine::ValidationEngine;

/// Section number shared by every foreign-key rule.
pub(super) const SECTION: &str = "5";
/// Default rule identifier for foreign-key violations. Rules with their own
/// identifier (e.g. `calendar_dates_service_not_in_calendar`) keep a local
/// `const RULE_ID`.
pub(super) const RULE_ID: &str = "foreign_key_violation";

/// Registers all foreign-key validation rules with the engine.
pub fn register_rules(engine: &mut ValidationEngine) {
    // Core (HW-017)
    engine.register_rule(Box::new(routes_agency::RoutesAgencyFkRule));
    engine.register_rule(Box::new(trips_route::TripsRouteFkRule));
    engine.register_rule(Box::new(trips_service::TripsServiceFkRule));
    engine.register_rule(Box::new(trips_shape::TripsShapeFkRule));
    engine.register_rule(Box::new(stop_times_trip::StopTimesTripFkRule));
    engine.register_rule(Box::new(stop_times_stop::StopTimesStopFkRule));
    engine.register_rule(Box::new(calendar_dates_service::CalendarDatesServiceFkRule));
    engine.register_rule(Box::new(frequencies_trip::FrequenciesTripFkRule));
    engine.register_rule(Box::new(stops_parent_station::StopsParentStationFkRule));
    engine.register_rule(Box::new(stops_level::StopsLevelFkRule));

    // Extended (HW-018)
    engine.register_rule(Box::new(transfers_from_stop::TransfersFromStopFkRule));
    engine.register_rule(Box::new(transfers_to_stop::TransfersToStopFkRule));
    engine.register_rule(Box::new(transfers_from_trip::TransfersFromTripFkRule));
    engine.register_rule(Box::new(transfers_to_trip::TransfersToTripFkRule));
    engine.register_rule(Box::new(transfers_from_route::TransfersFromRouteFkRule));
    engine.register_rule(Box::new(transfers_to_route::TransfersToRouteFkRule));
    engine.register_rule(Box::new(pathways_stops::PathwaysStopsFkRule));
    engine.register_rule(Box::new(fare_rules_fare::FareRulesFareFkRule));
    engine.register_rule(Box::new(fare_rules_route::FareRulesRouteFkRule));
    engine.register_rule(Box::new(fare_rules_zones::FareRulesZonesFkRule));
    engine.register_rule(Box::new(fare_attributes_agency::FareAttributesAgencyFkRule));
    engine.register_rule(Box::new(translations_record::TranslationsRecordFkRule));
    engine.register_rule(Box::new(attributions_refs::AttributionsRefsFkRule));

    // Fares v2 (HW-E26)
    engine.register_rule(Box::new(fare_products_media::FareProductsMediaFkRule));
    engine.register_rule(Box::new(fare_products_rider::FareProductsRiderFkRule));
    engine.register_rule(Box::new(fare_leg_rules_product::FareLegRulesProductFkRule));
    engine.register_rule(Box::new(fare_leg_rules_areas::FareLegRulesFromAreaFkRule));
    engine.register_rule(Box::new(fare_leg_rules_areas::FareLegRulesToAreaFkRule));
    engine.register_rule(Box::new(
        fare_leg_rules_timeframes::FareLegRulesFromTimeframeFkRule,
    ));
    engine.register_rule(Box::new(
        fare_leg_rules_timeframes::FareLegRulesToTimeframeFkRule,
    ));
    engine.register_rule(Box::new(fare_leg_rules_network::FareLegRulesNetworkFkRule));
    engine.register_rule(Box::new(
        fare_transfer_rules_legs::FareTransferRulesFromLegFkRule,
    ));
    engine.register_rule(Box::new(
        fare_transfer_rules_legs::FareTransferRulesToLegFkRule,
    ));
    engine.register_rule(Box::new(
        fare_transfer_rules_product::FareTransferRulesProductFkRule,
    ));
    engine.register_rule(Box::new(stop_areas::StopAreasAreaFkRule));
    engine.register_rule(Box::new(stop_areas::StopAreasStopFkRule));
    engine.register_rule(Box::new(timeframes_service::TimeframesServiceFkRule));
    engine.register_rule(Box::new(route_networks::RouteNetworksNetworkFkRule));
    engine.register_rule(Box::new(route_networks::RouteNetworksRouteFkRule));
    engine.register_rule(Box::new(
        fare_leg_join_rules::FareLegJoinRulesFromNetworkFkRule,
    ));
    engine.register_rule(Box::new(
        fare_leg_join_rules::FareLegJoinRulesToNetworkFkRule,
    ));
    engine.register_rule(Box::new(
        fare_leg_join_rules::FareLegJoinRulesFromStopFkRule,
    ));
    engine.register_rule(Box::new(fare_leg_join_rules::FareLegJoinRulesToStopFkRule));
}
