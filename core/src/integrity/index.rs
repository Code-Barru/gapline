use std::collections::{HashMap, HashSet, VecDeque};

use crate::models::{BookingRuleId, GtfsFeed, LocationGroupId, StopId, TripId, ZoneId};

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
    /// `booking_rule_id` -> `(trip_id, stop_sequence)` referencing it.
    pub booking_rule_to_stop_times: HashMap<BookingRuleId, Vec<(TripId, u32)>>,
    /// `location_group_id` -> member stops.
    pub location_group_to_stops: HashMap<LocationGroupId, Vec<StopId>>,
    /// `location_id` -> index in `feed.geojson_locations`.
    pub geojson_location_index: HashMap<String, usize>,
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

        let booking_rule_to_stop_times = build_booking_rule_reverse(feed);
        let location_group_to_stops = build_location_group_reverse(feed);
        let geojson_location_index = build_geojson_location_index(feed);

        Self {
            forward,
            reverse,
            booking_rule_to_stop_times,
            location_group_to_stops,
            geojson_location_index,
        }
    }

    #[must_use]
    pub fn geojson_location<'f>(
        &self,
        feed: &'f GtfsFeed,
        location_id: &str,
    ) -> Option<&'f crate::models::GeoJsonLocation> {
        self.geojson_location_index
            .get(location_id)
            .and_then(|&i| feed.geojson_locations.get(i))
    }

    #[must_use]
    pub fn stop_times_for_booking_rule(&self, id: &BookingRuleId) -> &[(TripId, u32)] {
        self.booking_rule_to_stop_times
            .get(id)
            .map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn stops_for_location_group(&self, id: &LocationGroupId) -> &[StopId] {
        self.location_group_to_stops
            .get(id)
            .map_or(&[], Vec::as_slice)
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
        + feed.fare_media.len()
        + feed.fare_products.len()
        + feed.fare_leg_rules.len()
        + feed.fare_transfer_rules.len()
        + feed.rider_categories.len()
        + feed.timeframes.len()
        + feed.areas.len()
        + feed.stop_areas.len()
        + feed.networks.len()
        + feed.route_networks.len()
        + feed.fare_leg_join_rules.len()
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

    register_fares_v2_entities(feed, forward);
}

fn register_fares_v2_entities(feed: &GtfsFeed, forward: &mut RelMap) {
    for fm in &feed.fare_media {
        forward
            .entry(EntityRef::FareMedia(fm.fare_media_id.clone()))
            .or_default();
    }
    for fp in &feed.fare_products {
        forward
            .entry(EntityRef::FareProduct(fp.fare_product_id.clone()))
            .or_default();
    }
    for rc in &feed.rider_categories {
        forward
            .entry(EntityRef::RiderCategory(rc.rider_category_id.clone()))
            .or_default();
    }
    for tf in &feed.timeframes {
        forward
            .entry(EntityRef::Timeframe(tf.timeframe_group_id.clone()))
            .or_default();
    }
    for area in &feed.areas {
        forward
            .entry(EntityRef::Area(area.area_id.clone()))
            .or_default();
    }
    for net in &feed.networks {
        forward
            .entry(EntityRef::Network(net.network_id.clone()))
            .or_default();
    }
    for flr in &feed.fare_leg_rules {
        if let Some(ref lg) = flr.leg_group_id {
            forward.entry(EntityRef::LegGroup(lg.clone())).or_default();
        }
    }
    for ftr in &feed.fare_transfer_rules {
        if let Some(ref lg) = ftr.from_leg_group_id {
            forward.entry(EntityRef::LegGroup(lg.clone())).or_default();
        }
        if let Some(ref lg) = ftr.to_leg_group_id {
            forward.entry(EntityRef::LegGroup(lg.clone())).or_default();
        }
    }
    for (i, _) in feed.fare_leg_rules.iter().enumerate() {
        forward.entry(EntityRef::FareLegRule(i)).or_default();
    }
    for (i, _) in feed.fare_transfer_rules.iter().enumerate() {
        forward.entry(EntityRef::FareTransferRule(i)).or_default();
    }
    for (i, _) in feed.stop_areas.iter().enumerate() {
        forward.entry(EntityRef::StopArea(i)).or_default();
    }
    for (i, _) in feed.route_networks.iter().enumerate() {
        forward.entry(EntityRef::RouteNetwork(i)).or_default();
    }
    for (i, _) in feed.fare_leg_join_rules.iter().enumerate() {
        forward.entry(EntityRef::FareLegJoinRule(i)).or_default();
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
    build_fares_v2_relations(feed, forward, reverse);
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

macro_rules! edge_opt {
    ($field:expr, $variant:ident, $rel:expr) => {
        (
            $field.as_ref().map(|v| EntityRef::$variant(v.clone())),
            $rel,
        )
    };
}
macro_rules! edge_req {
    ($expr:expr, $variant:ident, $rel:expr) => {
        (Some(EntityRef::$variant($expr.clone())), $rel)
    };
}

fn add_edges(
    forward: &mut RelMap,
    reverse: &mut RelMap,
    source: &EntityRef,
    edges: &[(Option<EntityRef>, RelationType)],
) {
    for (target, rel) in edges {
        if let Some(t) = target {
            add_relation(forward, reverse, source.clone(), t.clone(), *rel);
        }
    }
}

#[allow(clippy::too_many_lines)]
fn build_fares_v2_relations(feed: &GtfsFeed, forward: &mut RelMap, reverse: &mut RelMap) {
    use RelationType as R;

    for fp in &feed.fare_products {
        add_edges(
            forward,
            reverse,
            &EntityRef::FareProduct(fp.fare_product_id.clone()),
            &[
                edge_opt!(fp.fare_media_id, FareMedia, R::MediaOfFareProduct),
                edge_opt!(
                    fp.rider_category_id,
                    RiderCategory,
                    R::RiderCategoryOfFareProduct
                ),
            ],
        );
    }

    for (i, flr) in feed.fare_leg_rules.iter().enumerate() {
        add_edges(
            forward,
            reverse,
            &EntityRef::FareLegRule(i),
            &[
                edge_opt!(flr.leg_group_id, LegGroup, R::LegGroupOfFareLegRule),
                edge_opt!(flr.network_id, Network, R::NetworkOfFareLegRule),
                edge_opt!(flr.from_area_id, Area, R::FromAreaOfFareLegRule),
                edge_opt!(flr.to_area_id, Area, R::ToAreaOfFareLegRule),
                edge_opt!(
                    flr.from_timeframe_group_id,
                    Timeframe,
                    R::FromTimeframeOfFareLegRule
                ),
                edge_opt!(
                    flr.to_timeframe_group_id,
                    Timeframe,
                    R::ToTimeframeOfFareLegRule
                ),
                edge_req!(flr.fare_product_id, FareProduct, R::ProductOfFareLegRule),
            ],
        );
    }

    for (i, ftr) in feed.fare_transfer_rules.iter().enumerate() {
        add_edges(
            forward,
            reverse,
            &EntityRef::FareTransferRule(i),
            &[
                edge_opt!(
                    ftr.from_leg_group_id,
                    LegGroup,
                    R::FromLegGroupOfFareTransferRule
                ),
                edge_opt!(
                    ftr.to_leg_group_id,
                    LegGroup,
                    R::ToLegGroupOfFareTransferRule
                ),
                edge_opt!(
                    ftr.fare_product_id,
                    FareProduct,
                    R::ProductOfFareTransferRule
                ),
            ],
        );
    }

    for (i, sa) in feed.stop_areas.iter().enumerate() {
        add_edges(
            forward,
            reverse,
            &EntityRef::StopArea(i),
            &[
                edge_req!(sa.area_id, Area, R::AreaOfStopArea),
                edge_req!(sa.stop_id, Stop, R::StopOfStopArea),
            ],
        );
    }

    for (i, rn) in feed.route_networks.iter().enumerate() {
        add_edges(
            forward,
            reverse,
            &EntityRef::RouteNetwork(i),
            &[
                edge_req!(rn.network_id, Network, R::NetworkOfRouteNetwork),
                edge_req!(rn.route_id, Route, R::RouteOfRouteNetwork),
            ],
        );
    }

    for (i, fjr) in feed.fare_leg_join_rules.iter().enumerate() {
        add_edges(
            forward,
            reverse,
            &EntityRef::FareLegJoinRule(i),
            &[
                edge_req!(
                    fjr.from_network_id,
                    Network,
                    R::FromNetworkOfFareLegJoinRule
                ),
                edge_req!(fjr.to_network_id, Network, R::ToNetworkOfFareLegJoinRule),
                edge_opt!(fjr.from_stop_id, Stop, R::FromStopOfFareLegJoinRule),
                edge_opt!(fjr.to_stop_id, Stop, R::ToStopOfFareLegJoinRule),
            ],
        );
    }
}

fn build_booking_rule_reverse(feed: &GtfsFeed) -> HashMap<BookingRuleId, Vec<(TripId, u32)>> {
    let mut map: HashMap<BookingRuleId, Vec<(TripId, u32)>> = HashMap::new();
    for st in &feed.stop_times {
        if let Some(ref id) = st.pickup_booking_rule_id {
            map.entry(id.clone())
                .or_default()
                .push((st.trip_id.clone(), st.stop_sequence));
        }
        if let Some(ref id) = st.drop_off_booking_rule_id {
            map.entry(id.clone())
                .or_default()
                .push((st.trip_id.clone(), st.stop_sequence));
        }
    }
    map
}

fn build_location_group_reverse(feed: &GtfsFeed) -> HashMap<LocationGroupId, Vec<StopId>> {
    let mut map: HashMap<LocationGroupId, Vec<StopId>> = HashMap::new();
    for lgs in &feed.location_group_stops {
        map.entry(lgs.location_group_id.clone())
            .or_default()
            .push(lgs.stop_id.clone());
    }
    map
}

fn build_geojson_location_index(feed: &GtfsFeed) -> HashMap<String, usize> {
    feed.geojson_locations
        .iter()
        .enumerate()
        .map(|(i, loc)| (loc.id.clone(), i))
        .collect()
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
