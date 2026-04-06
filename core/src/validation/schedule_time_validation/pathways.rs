//! Pathway validation (section 7.10).
//!
//! Checks traversal time, one-way pathways without return, and stations
//! missing an entrance connected by pathway.

use std::collections::{HashMap, HashSet};

use crate::models::GtfsFeed;
use crate::models::{IsBidirectional, LocationType};
use crate::validation::{Severity, ValidationError, ValidationRule};

const PATHWAYS_FILE: &str = "pathways.txt";
const STOPS_FILE: &str = "stops.txt";
const SECTION: &str = "7";

/// Validates pathways for traversal time, directionality, and station
/// entrance connectivity.
pub struct PathwayValidationRule;

impl ValidationRule for PathwayValidationRule {
    fn rule_id(&self) -> &'static str {
        "pathway_validation"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (i, pathway) in feed.pathways.iter().enumerate() {
            if pathway.traversal_time == Some(0) {
                errors.push(
                    ValidationError::new("invalid_traversal_time", SECTION, Severity::Error)
                        .message(format!(
                            "Pathway '{}' has traversal_time=0",
                            pathway.pathway_id,
                        ))
                        .file(PATHWAYS_FILE)
                        .line(i + 2)
                        .field("traversal_time")
                        .value("0"),
                );
            }
        }

        // Bidirectional pathways implicitly provide a return path.
        let mut directed_edges: HashSet<(&str, &str)> = HashSet::new();
        for pathway in &feed.pathways {
            let from = pathway.from_stop_id.as_ref();
            let to = pathway.to_stop_id.as_ref();
            directed_edges.insert((from, to));
            if pathway.is_bidirectional == IsBidirectional::Bidirectional {
                directed_edges.insert((to, from));
            }
        }

        for (i, pathway) in feed.pathways.iter().enumerate() {
            if pathway.is_bidirectional == IsBidirectional::Unidirectional {
                let reverse = (pathway.to_stop_id.as_ref(), pathway.from_stop_id.as_ref());
                if !directed_edges.contains(&reverse) {
                    errors.push(
                        ValidationError::new(
                            "one_way_pathway_without_return",
                            SECTION,
                            Severity::Warning,
                        )
                        .message(format!(
                            "Pathway '{}' from '{}' to '{}' is unidirectional with no \
                             reverse pathway",
                            pathway.pathway_id, pathway.from_stop_id, pathway.to_stop_id,
                        ))
                        .file(PATHWAYS_FILE)
                        .line(i + 2)
                        .field("is_bidirectional")
                        .value("0"),
                    );
                }
            }
        }

        // Skip if the feed has no pathways to avoid noisy warnings for
        // feeds that don't model station internals.
        if !feed.pathways.is_empty() {
            let entrance_to_station: HashMap<&str, &str> = feed
                .stops
                .iter()
                .filter(|s| s.location_type == Some(LocationType::EntranceExit))
                .filter_map(|s| {
                    s.parent_station
                        .as_ref()
                        .map(|ps| (s.stop_id.as_ref(), ps.as_ref()))
                })
                .collect();

            let mut stations_with_entrance_pathway: HashSet<&str> = HashSet::new();
            for pathway in &feed.pathways {
                if let Some(&station) = entrance_to_station.get(pathway.from_stop_id.as_ref()) {
                    stations_with_entrance_pathway.insert(station);
                }
                if let Some(&station) = entrance_to_station.get(pathway.to_stop_id.as_ref()) {
                    stations_with_entrance_pathway.insert(station);
                }
            }

            for (i, stop) in feed.stops.iter().enumerate() {
                if stop.location_type == Some(LocationType::Station)
                    && !stations_with_entrance_pathway.contains(stop.stop_id.as_ref())
                {
                    errors.push(
                        ValidationError::new(
                            "station_without_entrance_pathway",
                            SECTION,
                            Severity::Warning,
                        )
                        .message(format!(
                            "Station '{}' has no entrance (location_type=2) connected \
                             by a pathway",
                            stop.stop_id,
                        ))
                        .file(STOPS_FILE)
                        .line(i + 2)
                        .field("stop_id")
                        .value(stop.stop_id.as_ref()),
                    );
                }
            }
        }

        errors
    }
}
