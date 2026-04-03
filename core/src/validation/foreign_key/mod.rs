//! Section 5 — Foreign Key Validation.
//!
//! Detects orphan references between core GTFS files. Each rule checks that
//! foreign-key values in a source file point to existing primary keys in the
//! target file. Violations produce `foreign_key_violation` errors with full
//! context (file, line, field, orphan value).

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

use crate::validation::engine::ValidationEngine;

/// Registers all foreign-key validation rules with the engine.
pub fn register_rules(engine: &mut ValidationEngine) {
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
}
