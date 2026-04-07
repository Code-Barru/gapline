//! Section 8 — Best-practice checks (WARNING / INFO only).
//!
//! These rules flag missing optional fields, naming conventions, and
//! accessibility information that improve feed quality but are not
//! specification violations.

pub mod missing_agency_email;
pub mod missing_bikes_info;
pub mod missing_direction_id;
pub mod missing_route_colors;
pub mod missing_wheelchair_info;
pub mod redundant_route_name;
pub mod route_short_name_too_long;
pub mod stop_name_all_caps;

pub use self::route_short_name_too_long::NamingThresholds;

use crate::validation::engine::ValidationEngine;

pub fn register_rules(engine: &mut ValidationEngine, thresholds: NamingThresholds) {
    engine.register_rule(Box::new(missing_agency_email::MissingAgencyEmailRule));
    engine.register_rule(Box::new(missing_route_colors::MissingRouteColorsRule));
    engine.register_rule(Box::new(missing_direction_id::MissingDirectionIdRule));
    engine.register_rule(Box::new(
        route_short_name_too_long::RouteShortNameTooLongRule::new(thresholds),
    ));
    engine.register_rule(Box::new(stop_name_all_caps::StopNameAllCapsRule));
    engine.register_rule(Box::new(redundant_route_name::RedundantRouteNameRule));
    engine.register_rule(Box::new(
        missing_wheelchair_info::MissingWheelchairStopsRule,
    ));
    engine.register_rule(Box::new(
        missing_wheelchair_info::MissingWheelchairTripsRule,
    ));
    engine.register_rule(Box::new(missing_bikes_info::MissingBikesInfoRule));
}
