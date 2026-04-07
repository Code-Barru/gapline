use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const STOPS_FILE: &str = "stops.txt";
const TRIPS_FILE: &str = "trips.txt";
const SECTION: &str = "8";
const RULE_ID: &str = "missing_wheelchair_info";

/// Flags stops missing `wheelchair_boarding`.
pub struct MissingWheelchairStopsRule;

impl ValidationRule for MissingWheelchairStopsRule {
    fn rule_id(&self) -> &'static str {
        RULE_ID
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        feed.stops
            .iter()
            .enumerate()
            .filter(|(_, stop)| stop.wheelchair_boarding.is_none())
            .map(|(i, _)| {
                ValidationError::new(RULE_ID, SECTION, Severity::Warning)
                    .message("wheelchair_boarding is recommended for accessibility")
                    .file(STOPS_FILE)
                    .line(i + 2)
                    .field("wheelchair_boarding")
            })
            .collect()
    }
}

/// Flags trips missing `wheelchair_accessible`.
pub struct MissingWheelchairTripsRule;

impl ValidationRule for MissingWheelchairTripsRule {
    fn rule_id(&self) -> &'static str {
        RULE_ID
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        feed.trips
            .iter()
            .enumerate()
            .filter(|(_, trip)| trip.wheelchair_accessible.is_none())
            .map(|(i, _)| {
                ValidationError::new(RULE_ID, SECTION, Severity::Warning)
                    .message("wheelchair_accessible is recommended for accessibility")
                    .file(TRIPS_FILE)
                    .line(i + 2)
                    .field("wheelchair_accessible")
            })
            .collect()
    }
}
