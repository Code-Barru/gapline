//! Section 8 — Best-practice checks (WARNING / INFO only).
//!
//! These rules flag missing optional fields, naming conventions, and
//! accessibility information that improve feed quality but are not
//! specification violations.

/// Generates a `ValidationRule` impl that flags every record in a feed
/// collection where an `Option`-typed field is `None`. Used by the handful of
/// missing-field best-practice rules that all share the same filter-map
/// shape.
macro_rules! missing_field_rule {
    (
        $struct_name:ident,
        rule_id = $rule_id:literal,
        file = $file:literal,
        collection = $coll:ident,
        field = $field:ident,
        severity = $severity:expr,
        message = $message:literal $(,)?
    ) => {
        pub struct $struct_name;

        impl $crate::validation::ValidationRule for $struct_name {
            fn rule_id(&self) -> &'static str {
                $rule_id
            }
            fn section(&self) -> &'static str {
                "8"
            }
            fn severity(&self) -> $crate::validation::Severity {
                $severity
            }
            fn validate(
                &self,
                feed: &$crate::models::GtfsFeed,
            ) -> Vec<$crate::validation::ValidationError> {
                feed.$coll
                    .iter()
                    .enumerate()
                    .filter(|(_, item)| item.$field.is_none())
                    .map(|(i, _)| {
                        $crate::validation::ValidationError::new($rule_id, "8", $severity)
                            .message($message)
                            .file($file)
                            .line(i + 2)
                            .field(stringify!($field))
                    })
                    .collect()
            }
        }
    };
}

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
