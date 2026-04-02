use std::collections::{HashMap, HashSet, VecDeque};

use crate::models::{GtfsFeed, ZoneId};

use super::types::{EntityRef, RelationType};

type RelMap = HashMap<EntityRef, Vec<(EntityRef, RelationType)>>;

/// Bidirectional referential integrity index for a GTFS feed.
///
/// Built once after feed loading, then shared (via `Arc`) between the
/// validation engine and the CRUD engine.
///
/// Every known entity is registered as a key in `forward`, even those
/// with no outgoing FK (empty `Vec`), so `forward` doubles as the
/// entity registry.
pub struct IntegrityIndex {
    /// FK source -> referenced targets. Also the entity registry.
    pub forward: RelMap,
    /// PK target -> dependents referencing it.
    pub reverse: RelMap,
}

const EMPTY: &[(EntityRef, RelationType); 0] = &[];

impl IntegrityIndex {
    #[must_use]
    pub fn build_from_feed(feed: &GtfsFeed) -> Self {
        let capacity = estimate_capacity(feed);
        let mut forward: RelMap = HashMap::with_capacity(capacity);
        let mut reverse: RelMap = HashMap::with_capacity(capacity);

        register_entities(feed, &mut forward);
        build_relations(feed, &mut forward, &mut reverse);

        Self { forward, reverse }
    }

    #[must_use]
    pub fn entity_exists(&self, entity: &EntityRef) -> bool {
        self.forward.contains_key(entity)
    }

    #[must_use]
    pub fn get_references(&self, from: &EntityRef) -> &[(EntityRef, RelationType)] {
        self.forward.get(from).map_or(EMPTY, Vec::as_slice)
    }

    #[must_use]
    pub fn find_dependents(&self, entity: &EntityRef) -> &[(EntityRef, RelationType)] {
        self.reverse.get(entity).map_or(EMPTY, Vec::as_slice)
    }

    /// BFS over the reverse index to collect all transitive dependents.
    #[must_use]
    pub fn find_dependents_recursive(&self, entity: &EntityRef) -> Vec<(EntityRef, RelationType)> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        for (dependent, relation) in self.find_dependents(entity) {
            queue.push_back((dependent.clone(), *relation));
        }

        while let Some((dependent, relation)) = queue.pop_front() {
            if visited.insert(dependent.clone()) {
                result.push((dependent.clone(), relation));
                for (child, child_relation) in self.find_dependents(&dependent) {
                    if !visited.contains(child) {
                        queue.push_back((child.clone(), *child_relation));
                    }
                }
            }
        }

        result
    }
}

fn estimate_capacity(feed: &GtfsFeed) -> usize {
    feed.agencies.len()
        + feed.stops.len()
        + feed.routes.len()
        + feed.trips.len()
        + feed.calendars.len()
        + feed.calendar_dates.len()
        + feed.shapes.len()
        + feed.frequencies.len()
        + feed.transfers.len()
        + feed.pathways.len()
        + feed.levels.len()
        + feed.fare_attributes.len()
        + feed.fare_rules.len()
        + feed.stop_times.len()
        + feed.attributions.len()
}

fn register_entities(feed: &GtfsFeed, forward: &mut RelMap) {
    for agency in &feed.agencies {
        if let Some(ref agency_id) = agency.agency_id {
            forward
                .entry(EntityRef::Agency(agency_id.clone()))
                .or_default();
        }
    }

    for stop in &feed.stops {
        forward
            .entry(EntityRef::Stop(stop.stop_id.clone()))
            .or_default();
        if let Some(ref zone_id) = stop.zone_id {
            forward
                .entry(EntityRef::Zone(ZoneId::from(zone_id.as_str())))
                .or_default();
        }
    }

    for route in &feed.routes {
        forward
            .entry(EntityRef::Route(route.route_id.clone()))
            .or_default();
    }

    for trip in &feed.trips {
        forward
            .entry(EntityRef::Trip(trip.trip_id.clone()))
            .or_default();
    }

    for calendar in &feed.calendars {
        forward
            .entry(EntityRef::Service(calendar.service_id.clone()))
            .or_default();
    }

    for calendar_date in &feed.calendar_dates {
        forward
            .entry(EntityRef::Service(calendar_date.service_id.clone()))
            .or_default();
        forward
            .entry(EntityRef::CalendarDate(
                calendar_date.service_id.clone(),
                calendar_date.date,
            ))
            .or_default();
    }

    for shape in &feed.shapes {
        forward
            .entry(EntityRef::Shape(shape.shape_id.clone()))
            .or_default();
        forward
            .entry(EntityRef::ShapePoint(
                shape.shape_id.clone(),
                shape.shape_pt_sequence,
            ))
            .or_default();
    }

    for stop_time in &feed.stop_times {
        forward
            .entry(EntityRef::StopTime(
                stop_time.trip_id.clone(),
                stop_time.stop_sequence,
            ))
            .or_default();
    }

    for frequency in &feed.frequencies {
        forward
            .entry(EntityRef::Frequency(
                frequency.trip_id.clone(),
                frequency.start_time.total_seconds,
            ))
            .or_default();
    }

    for (index, _) in feed.transfers.iter().enumerate() {
        forward.entry(EntityRef::Transfer(index)).or_default();
    }

    for pathway in &feed.pathways {
        forward
            .entry(EntityRef::Pathway(pathway.pathway_id.clone()))
            .or_default();
    }

    for level in &feed.levels {
        forward
            .entry(EntityRef::Level(level.level_id.clone()))
            .or_default();
    }

    for fare_attribute in &feed.fare_attributes {
        forward
            .entry(EntityRef::Fare(fare_attribute.fare_id.clone()))
            .or_default();
    }

    for (index, _) in feed.fare_rules.iter().enumerate() {
        forward.entry(EntityRef::FareRule(index)).or_default();
    }

    for (index, _) in feed.attributions.iter().enumerate() {
        forward.entry(EntityRef::Attribution(index)).or_default();
    }
}

fn build_relations(feed: &GtfsFeed, forward: &mut RelMap, reverse: &mut RelMap) {
    build_route_relations(feed, forward, reverse);
    build_trip_relations(feed, forward, reverse);
    build_stop_time_relations(feed, forward, reverse);
    build_calendar_date_relations(feed, forward, reverse);
    build_frequency_relations(feed, forward, reverse);
    build_stop_relations(feed, forward, reverse);
    build_transfer_relations(feed, forward, reverse);
    build_pathway_relations(feed, forward, reverse);
    build_fare_relations(feed, forward, reverse);
    build_attribution_relations(feed, forward, reverse);
}

fn build_route_relations(feed: &GtfsFeed, forward: &mut RelMap, reverse: &mut RelMap) {
    for route in &feed.routes {
        if let Some(ref agency_id) = route.agency_id {
            add_relation(
                forward,
                reverse,
                EntityRef::Route(route.route_id.clone()),
                EntityRef::Agency(agency_id.clone()),
                RelationType::AgencyOfRoute,
            );
        }
    }
}

fn build_trip_relations(feed: &GtfsFeed, forward: &mut RelMap, reverse: &mut RelMap) {
    for trip in &feed.trips {
        let source = EntityRef::Trip(trip.trip_id.clone());
        add_relation(
            forward,
            reverse,
            source.clone(),
            EntityRef::Route(trip.route_id.clone()),
            RelationType::RouteOfTrip,
        );
        add_relation(
            forward,
            reverse,
            source.clone(),
            EntityRef::Service(trip.service_id.clone()),
            RelationType::ServiceOfTrip,
        );
        if let Some(ref shape_id) = trip.shape_id {
            add_relation(
                forward,
                reverse,
                source,
                EntityRef::Shape(shape_id.clone()),
                RelationType::ShapeOfTrip,
            );
        }
    }
}

fn build_stop_time_relations(feed: &GtfsFeed, forward: &mut RelMap, reverse: &mut RelMap) {
    for stop_time in &feed.stop_times {
        let source = EntityRef::StopTime(stop_time.trip_id.clone(), stop_time.stop_sequence);
        add_relation(
            forward,
            reverse,
            source.clone(),
            EntityRef::Trip(stop_time.trip_id.clone()),
            RelationType::TripOfStopTime,
        );
        add_relation(
            forward,
            reverse,
            source,
            EntityRef::Stop(stop_time.stop_id.clone()),
            RelationType::StopOfStopTime,
        );
    }
}

fn build_calendar_date_relations(feed: &GtfsFeed, forward: &mut RelMap, reverse: &mut RelMap) {
    for calendar_date in &feed.calendar_dates {
        let source = EntityRef::CalendarDate(calendar_date.service_id.clone(), calendar_date.date);
        add_relation(
            forward,
            reverse,
            source,
            EntityRef::Service(calendar_date.service_id.clone()),
            RelationType::ServiceOfCalendarDate,
        );
    }
}

fn build_frequency_relations(feed: &GtfsFeed, forward: &mut RelMap, reverse: &mut RelMap) {
    for frequency in &feed.frequencies {
        let source = EntityRef::Frequency(
            frequency.trip_id.clone(),
            frequency.start_time.total_seconds,
        );
        add_relation(
            forward,
            reverse,
            source,
            EntityRef::Trip(frequency.trip_id.clone()),
            RelationType::TripOfFrequency,
        );
    }
}

fn build_stop_relations(feed: &GtfsFeed, forward: &mut RelMap, reverse: &mut RelMap) {
    for stop in &feed.stops {
        let source = EntityRef::Stop(stop.stop_id.clone());
        if let Some(ref parent_station_id) = stop.parent_station {
            add_relation(
                forward,
                reverse,
                source.clone(),
                EntityRef::Stop(parent_station_id.clone()),
                RelationType::ParentStation,
            );
        }
        if let Some(ref level_id) = stop.level_id {
            add_relation(
                forward,
                reverse,
                source,
                EntityRef::Level(level_id.clone()),
                RelationType::LevelOfStop,
            );
        }
    }
}

fn build_transfer_relations(feed: &GtfsFeed, forward: &mut RelMap, reverse: &mut RelMap) {
    for (index, transfer) in feed.transfers.iter().enumerate() {
        let source = EntityRef::Transfer(index);
        if let Some(ref from_stop_id) = transfer.from_stop_id {
            add_relation(
                forward,
                reverse,
                source.clone(),
                EntityRef::Stop(from_stop_id.clone()),
                RelationType::TransferFromStop,
            );
        }
        if let Some(ref to_stop_id) = transfer.to_stop_id {
            add_relation(
                forward,
                reverse,
                source.clone(),
                EntityRef::Stop(to_stop_id.clone()),
                RelationType::TransferToStop,
            );
        }
        if let Some(ref from_route_id) = transfer.from_route_id {
            add_relation(
                forward,
                reverse,
                source.clone(),
                EntityRef::Route(from_route_id.clone()),
                RelationType::TransferFromRoute,
            );
        }
        if let Some(ref to_route_id) = transfer.to_route_id {
            add_relation(
                forward,
                reverse,
                source.clone(),
                EntityRef::Route(to_route_id.clone()),
                RelationType::TransferToRoute,
            );
        }
        if let Some(ref from_trip_id) = transfer.from_trip_id {
            add_relation(
                forward,
                reverse,
                source.clone(),
                EntityRef::Trip(from_trip_id.clone()),
                RelationType::TransferFromTrip,
            );
        }
        if let Some(ref to_trip_id) = transfer.to_trip_id {
            add_relation(
                forward,
                reverse,
                source,
                EntityRef::Trip(to_trip_id.clone()),
                RelationType::TransferToTrip,
            );
        }
    }
}

fn build_pathway_relations(feed: &GtfsFeed, forward: &mut RelMap, reverse: &mut RelMap) {
    for pathway in &feed.pathways {
        let source = EntityRef::Pathway(pathway.pathway_id.clone());
        add_relation(
            forward,
            reverse,
            source.clone(),
            EntityRef::Stop(pathway.from_stop_id.clone()),
            RelationType::PathwayFromStop,
        );
        add_relation(
            forward,
            reverse,
            source,
            EntityRef::Stop(pathway.to_stop_id.clone()),
            RelationType::PathwayToStop,
        );
    }
}

fn build_fare_relations(feed: &GtfsFeed, forward: &mut RelMap, reverse: &mut RelMap) {
    for fare_attribute in &feed.fare_attributes {
        if let Some(ref agency_id) = fare_attribute.agency_id {
            add_relation(
                forward,
                reverse,
                EntityRef::Fare(fare_attribute.fare_id.clone()),
                EntityRef::Agency(agency_id.clone()),
                RelationType::AgencyOfFareAttribute,
            );
        }
    }

    for (index, fare_rule) in feed.fare_rules.iter().enumerate() {
        let source = EntityRef::FareRule(index);
        add_relation(
            forward,
            reverse,
            source.clone(),
            EntityRef::Fare(fare_rule.fare_id.clone()),
            RelationType::FareOfFareRule,
        );
        if let Some(ref route_id) = fare_rule.route_id {
            add_relation(
                forward,
                reverse,
                source.clone(),
                EntityRef::Route(route_id.clone()),
                RelationType::RouteOfFareRule,
            );
        }
        if let Some(ref origin_id) = fare_rule.origin_id {
            add_relation(
                forward,
                reverse,
                source.clone(),
                EntityRef::Zone(ZoneId::from(origin_id.as_str())),
                RelationType::OriginZoneOfFareRule,
            );
        }
        if let Some(ref destination_id) = fare_rule.destination_id {
            add_relation(
                forward,
                reverse,
                source.clone(),
                EntityRef::Zone(ZoneId::from(destination_id.as_str())),
                RelationType::DestinationZoneOfFareRule,
            );
        }
        if let Some(ref contains_id) = fare_rule.contains_id {
            add_relation(
                forward,
                reverse,
                source,
                EntityRef::Zone(ZoneId::from(contains_id.as_str())),
                RelationType::ContainsZoneOfFareRule,
            );
        }
    }
}

fn build_attribution_relations(feed: &GtfsFeed, forward: &mut RelMap, reverse: &mut RelMap) {
    for (index, attribution) in feed.attributions.iter().enumerate() {
        let source = EntityRef::Attribution(index);
        if let Some(ref agency_id) = attribution.agency_id {
            add_relation(
                forward,
                reverse,
                source.clone(),
                EntityRef::Agency(agency_id.clone()),
                RelationType::AgencyOfAttribution,
            );
        }
        if let Some(ref route_id) = attribution.route_id {
            add_relation(
                forward,
                reverse,
                source.clone(),
                EntityRef::Route(route_id.clone()),
                RelationType::RouteOfAttribution,
            );
        }
        if let Some(ref trip_id) = attribution.trip_id {
            add_relation(
                forward,
                reverse,
                source,
                EntityRef::Trip(trip_id.clone()),
                RelationType::TripOfAttribution,
            );
        }
    }
}

fn add_relation(
    forward: &mut RelMap,
    reverse: &mut RelMap,
    source: EntityRef,
    target: EntityRef,
    relation: RelationType,
) {
    forward
        .entry(source.clone())
        .or_default()
        .push((target.clone(), relation));
    reverse.entry(target).or_default().push((source, relation));
}
