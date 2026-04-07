//! Unused entity detection (section 7).
//!
//! Warns about routes, shapes, services, agencies, and fares that are
//! defined but never referenced by any trip or fare rule.

use std::collections::{HashMap, HashSet};

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const SECTION: &str = "7";

/// Warns when a route is not referenced by any trip.
pub struct UnusedRouteRule;

impl ValidationRule for UnusedRouteRule {
    fn rule_id(&self) -> &'static str {
        "unused_route"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let referenced: HashSet<&str> = feed.trips.iter().map(|t| t.route_id.as_ref()).collect();
        let mut errors = Vec::new();
        for (i, route) in feed.routes.iter().enumerate() {
            if !referenced.contains(route.route_id.as_ref()) {
                errors.push(
                    ValidationError::new("unused_route", SECTION, Severity::Warning)
                        .message(format!(
                            "route '{}' is not referenced by any trip",
                            route.route_id,
                        ))
                        .file("routes.txt")
                        .line(i + 2)
                        .field("route_id")
                        .value(route.route_id.as_ref()),
                );
            }
        }
        errors
    }
}

/// Warns when a shape is not referenced by any trip.
pub struct UnusedShapeRule;

impl ValidationRule for UnusedShapeRule {
    fn rule_id(&self) -> &'static str {
        "unused_shape"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let referenced: HashSet<&str> = feed
            .trips
            .iter()
            .filter_map(|t| t.shape_id.as_ref().map(AsRef::as_ref))
            .collect();

        // Shapes are stored as individual points; deduplicate by shape_id.
        let mut seen: HashMap<&str, usize> = HashMap::new();
        for (i, shape) in feed.shapes.iter().enumerate() {
            seen.entry(shape.shape_id.as_ref()).or_insert(i);
        }

        let mut errors = Vec::new();
        for (&shape_id, &first_line_idx) in &seen {
            if !referenced.contains(shape_id) {
                errors.push(
                    ValidationError::new("unused_shape", SECTION, Severity::Warning)
                        .message(format!("shape '{shape_id}' is not referenced by any trip",))
                        .file("shapes.txt")
                        .line(first_line_idx + 2)
                        .field("shape_id")
                        .value(shape_id),
                );
            }
        }
        errors
    }
}

/// Warns when a service (from `calendar.txt` or `calendar_dates.txt`) is
/// not referenced by any trip.
pub struct UnusedServiceRule;

impl ValidationRule for UnusedServiceRule {
    fn rule_id(&self) -> &'static str {
        "unused_service"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let referenced: HashSet<&str> = feed.trips.iter().map(|t| t.service_id.as_ref()).collect();

        let mut defined: HashMap<&str, (&str, usize)> = HashMap::new();
        for (i, cal) in feed.calendars.iter().enumerate() {
            defined
                .entry(cal.service_id.as_ref())
                .or_insert(("calendar.txt", i + 2));
        }
        for (i, cd) in feed.calendar_dates.iter().enumerate() {
            defined
                .entry(cd.service_id.as_ref())
                .or_insert(("calendar_dates.txt", i + 2));
        }

        let mut errors = Vec::new();
        for (&sid, &(file, line)) in &defined {
            if !referenced.contains(sid) {
                errors.push(
                    ValidationError::new("unused_service", SECTION, Severity::Warning)
                        .message(format!("service '{sid}' is not referenced by any trip",))
                        .file(file)
                        .line(line)
                        .field("service_id")
                        .value(sid),
                );
            }
        }
        errors
    }
}

/// Warns when an agency is not referenced by any route. Only applies when
/// the feed contains more than one agency.
pub struct UnusedAgencyRule;

impl ValidationRule for UnusedAgencyRule {
    fn rule_id(&self) -> &'static str {
        "unused_agency"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        // Single agency is implicitly referenced by all routes (GTFS spec).
        if feed.agencies.len() <= 1 {
            return Vec::new();
        }

        let referenced: HashSet<&str> = feed
            .routes
            .iter()
            .filter_map(|r| r.agency_id.as_ref().map(AsRef::as_ref))
            .collect();

        let mut errors = Vec::new();
        for (i, agency) in feed.agencies.iter().enumerate() {
            let Some(ref aid) = agency.agency_id else {
                continue;
            };
            if !referenced.contains(aid.as_ref()) {
                errors.push(
                    ValidationError::new("unused_agency", SECTION, Severity::Warning)
                        .message(format!("agency '{aid}' is not referenced by any route",))
                        .file("agency.txt")
                        .line(i + 2)
                        .field("agency_id")
                        .value(aid.as_ref()),
                );
            }
        }
        errors
    }
}

/// Warns when a fare attribute is not referenced by any fare rule.
pub struct UnusedFareRule;

impl ValidationRule for UnusedFareRule {
    fn rule_id(&self) -> &'static str {
        "unused_fare"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let referenced: HashSet<&str> = feed
            .fare_rules
            .iter()
            .map(|fr| fr.fare_id.as_ref())
            .collect();

        let mut errors = Vec::new();
        for (i, fare) in feed.fare_attributes.iter().enumerate() {
            if !referenced.contains(fare.fare_id.as_ref()) {
                errors.push(
                    ValidationError::new("unused_fare", SECTION, Severity::Warning)
                        .message(format!(
                            "fare '{}' is not referenced by any fare_rule",
                            fare.fare_id,
                        ))
                        .file("fare_attributes.txt")
                        .line(i + 2)
                        .field("fare_id")
                        .value(fare.fare_id.as_ref()),
                );
            }
        }
        errors
    }
}
