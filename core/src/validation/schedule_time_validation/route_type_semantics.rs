//! Route type semantics validation (section 7.8).
//!
//! Checks that `route_type` values are either standard GTFS (0–12) or
//! officially recognised Extended Route Types (HVT 100–1702). Unknown values
//! produce a WARNING; recognised extended values produce an INFO.

use std::collections::HashSet;
use std::sync::LazyLock;

use crate::models::{GtfsFeed, RouteType};
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "routes.txt";
const SECTION: &str = "7";

/// Official GTFS Extended Route Types (HVT codes).
/// Source: <https://gtfs.org/documentation/schedule/reference/#routestxt>
static EXTENDED_ROUTE_TYPES: LazyLock<HashSet<u16>> = LazyLock::new(|| {
    [
        // Railway Service (100–117)
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117,
        // Coach Service (200–209)
        200, 201, 202, 203, 204, 205, 206, 207, 208, 209,
        // Suburban Railway Service (300)
        300, // Urban Railway Service (400–405)
        400, 401, 402, 403, 404, 405, // Metro Service (500)
        500, // Underground Service (600)
        600, // Tram Service (700–717)
        700, 701, 702, 703, 704, 710, 711, 712, 713, 714, 715, 716, 717,
        // Bus Service (800)
        800, // Trolleybus Service (900–907)
        900, 901, 902, 903, 904, 905, 906, 907, // Water Transport Service (1000–1002)
        1000, 1001, 1002, // Air Service (1100–1114)
        1100, 1101, 1102, 1103, 1104, 1105, 1106, 1107, 1108, 1109, 1110, 1111, 1112, 1113, 1114,
        // Ferry Service (1200)
        1200, // Aerial Lift Service (1300)
        1300, // Funicular Service (1400)
        1400, // Taxi Service (1500–1507)
        1500, 1501, 1502, 1503, 1504, 1505, 1506, 1507,
        // Miscellaneous Service (1600–1602)
        1600, 1601, 1602, // Horse-drawn Carriage (1700–1702)
        1700, 1701, 1702,
    ]
    .into_iter()
    .collect()
});

/// Validates route type semantics: INFO for official extended types, WARNING
/// for unrecognised values.
pub struct RouteTypeSemanticsRule;

impl ValidationRule for RouteTypeSemanticsRule {
    fn rule_id(&self) -> &'static str {
        "route_type_semantics"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (i, route) in feed.routes.iter().enumerate() {
            let line = i + 2;

            match route.route_type {
                // Standard types: no finding.
                RouteType::Tram
                | RouteType::Subway
                | RouteType::Rail
                | RouteType::Bus
                | RouteType::Ferry
                | RouteType::CableTram
                | RouteType::AerialLift
                | RouteType::Funicular
                | RouteType::Trolleybus
                | RouteType::Monorail => {}

                // HVT range: check if the value is an official Extended Route Type.
                RouteType::Hvt(v) => {
                    if EXTENDED_ROUTE_TYPES.contains(&v) {
                        errors.push(
                            ValidationError::new("extended_route_type", SECTION, Severity::Info)
                                .message(format!(
                                    "Route '{}' uses extended route_type {v}",
                                    route.route_id.as_ref()
                                ))
                                .file(FILE)
                                .line(line)
                                .field("route_type")
                                .value(v.to_string()),
                        );
                    } else {
                        errors.push(
                            ValidationError::new("unknown_route_type", SECTION, Severity::Warning)
                                .message(format!(
                                    "Route '{}' has unrecognized route_type {v}",
                                    route.route_id.as_ref()
                                ))
                                .file(FILE)
                                .line(line)
                                .field("route_type")
                                .value(v.to_string()),
                        );
                    }
                }

                // Anything outside the known ranges.
                RouteType::Unknown(v) => {
                    errors.push(
                        ValidationError::new("unknown_route_type", SECTION, Severity::Warning)
                            .message(format!(
                                "Route '{}' has unrecognized route_type {v}",
                                route.route_id.as_ref()
                            ))
                            .file(FILE)
                            .line(line)
                            .field("route_type")
                            .value(v.to_string()),
                    );
                }
            }
        }

        errors
    }
}
