use std::sync::Arc;

use gapline_core::integrity::{EntityRef, IntegrityIndex, RelationType};
use gapline_core::models::*;

// ---------------------------------------------------------------------------
// Test helpers - small factory functions for GTFS entities
// ---------------------------------------------------------------------------

fn make_agency(id: &str) -> Agency {
    Agency {
        agency_id: Some(AgencyId::from(id)),
        agency_name: format!("Agency {id}"),
        agency_url: Url::from("http://example.com"),
        agency_timezone: Timezone::from("America/Montreal"),
        agency_lang: None,
        agency_phone: None,
        agency_fare_url: None,
        agency_email: None,
    }
}

fn make_route(id: &str, agency_id: Option<&str>) -> Route {
    Route {
        route_id: RouteId::from(id),
        agency_id: agency_id.map(AgencyId::from),
        route_short_name: None,
        route_long_name: None,
        route_desc: None,
        route_type: RouteType::Bus,
        route_url: None,
        route_color: None,
        route_text_color: None,
        route_sort_order: None,
        continuous_pickup: None,
        continuous_drop_off: None,
        network_id: None,
    }
}

fn make_calendar(service_id: &str) -> Calendar {
    Calendar {
        service_id: ServiceId::from(service_id),
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        start_date: GtfsDate::default(),
        end_date: GtfsDate::default(),
    }
}

fn make_trip(id: &str, route: &str, service: &str) -> Trip {
    Trip {
        route_id: RouteId::from(route),
        service_id: ServiceId::from(service),
        trip_id: TripId::from(id),
        trip_headsign: None,
        trip_short_name: None,
        direction_id: None,
        block_id: None,
        shape_id: None,
        wheelchair_accessible: None,
        bikes_allowed: None,
    }
}

fn make_stop(id: &str) -> Stop {
    Stop {
        stop_id: StopId::from(id),
        stop_code: None,
        stop_name: None,
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: None,
        stop_lon: None,
        zone_id: None,
        stop_url: None,
        location_type: None,
        parent_station: None,
        stop_timezone: None,
        wheelchair_boarding: None,
        level_id: None,
        platform_code: None,
    }
}

fn make_stop_time(trip: &str, stop: &str, sequence: u32) -> StopTime {
    StopTime {
        trip_id: TripId::from(trip),
        arrival_time: None,
        departure_time: None,
        stop_id: StopId::from(stop),
        stop_sequence: sequence,
        stop_headsign: None,
        pickup_type: None,
        drop_off_type: None,
        continuous_pickup: None,
        continuous_drop_off: None,
        shape_dist_traveled: None,
        timepoint: None,
        start_pickup_drop_off_window: None,
        end_pickup_drop_off_window: None,
        pickup_booking_rule_id: None,
        drop_off_booking_rule_id: None,
        mean_duration_factor: None,
        mean_duration_offset: None,
        safe_duration_factor: None,
        safe_duration_offset: None,
    }
}

/// Build a minimal but complete feed: 1 agency, 1 route, 1 calendar,
/// 1 trip, 2 stops, 2 `stop_times`.
fn minimal_feed() -> GtfsFeed {
    let mut feed = GtfsFeed::default();
    feed.agencies.push(make_agency("A1"));
    feed.stops.push(make_stop("S1"));
    feed.stops.push(make_stop("S2"));
    feed.routes.push(make_route("R1", Some("A1")));
    feed.calendars.push(make_calendar("SVC1"));
    feed.trips.push(make_trip("T1", "R1", "SVC1"));
    feed.stop_times.push(make_stop_time("T1", "S1", 1));
    feed.stop_times.push(make_stop_time("T1", "S2", 2));
    feed
}

// ---------------------------------------------------------------------------
// Test 1: Minimal feed - all entities registered, relations correct
// ---------------------------------------------------------------------------
#[test]
fn test_minimal_feed() {
    let feed = minimal_feed();
    let index = IntegrityIndex::build_from_feed(&feed);

    // 8 entities: A1, S1, S2, R1, SVC1, T1, ST(T1,1), ST(T1,2)
    assert!(index.entity_exists(&EntityRef::Agency(AgencyId::from("A1"))));
    assert!(index.entity_exists(&EntityRef::Stop(StopId::from("S1"))));
    assert!(index.entity_exists(&EntityRef::Stop(StopId::from("S2"))));
    assert!(index.entity_exists(&EntityRef::Route(RouteId::from("R1"))));
    assert!(index.entity_exists(&EntityRef::Service(ServiceId::from("SVC1"))));
    assert!(index.entity_exists(&EntityRef::Trip(TripId::from("T1"))));
    assert!(index.entity_exists(&EntityRef::StopTime(TripId::from("T1"), 1)));
    assert!(index.entity_exists(&EntityRef::StopTime(TripId::from("T1"), 2)));

    // Route R1 references Agency A1
    let route_refs = index.get_references(&EntityRef::Route(RouteId::from("R1")));
    assert!(route_refs.iter().any(|(entity, relation)| {
        *entity == EntityRef::Agency(AgencyId::from("A1"))
            && *relation == RelationType::AgencyOfRoute
    }));
}

// ---------------------------------------------------------------------------
// Test 2: Empty feed
// ---------------------------------------------------------------------------
#[test]
fn test_empty_feed() {
    let feed = GtfsFeed::default();
    let index = IntegrityIndex::build_from_feed(&feed);
    assert!(index.forward.is_empty());
    assert!(index.reverse.is_empty());
}

// ---------------------------------------------------------------------------
// Test 3: Dependents of a trip (3 stop_times)
// ---------------------------------------------------------------------------
#[test]
fn test_dependents_of_trip() {
    let mut feed = GtfsFeed::default();
    feed.trips.push(make_trip("T1", "R1", "SVC1"));
    feed.stop_times.push(make_stop_time("T1", "S1", 1));
    feed.stop_times.push(make_stop_time("T1", "S2", 2));
    feed.stop_times.push(make_stop_time("T1", "S3", 3));

    let index = IntegrityIndex::build_from_feed(&feed);
    let dependents = index.find_dependents(&EntityRef::Trip(TripId::from("T1")));

    let stop_time_dependents: Vec<_> = dependents
        .iter()
        .filter(|(_, relation)| *relation == RelationType::TripOfStopTime)
        .collect();
    assert_eq!(stop_time_dependents.len(), 3);
}

// ---------------------------------------------------------------------------
// Test 4: Dependents of a route (2 trips)
// ---------------------------------------------------------------------------
#[test]
fn test_dependents_of_route() {
    let mut feed = GtfsFeed::default();
    feed.routes.push(make_route("R1", None));
    feed.trips.push(make_trip("T1", "R1", "SVC1"));
    feed.trips.push(make_trip("T2", "R1", "SVC1"));

    let index = IntegrityIndex::build_from_feed(&feed);
    let dependents = index.find_dependents(&EntityRef::Route(RouteId::from("R1")));

    let trip_dependents: Vec<_> = dependents
        .iter()
        .filter(|(_, relation)| *relation == RelationType::RouteOfTrip)
        .collect();
    assert_eq!(trip_dependents.len(), 2);
}

// ---------------------------------------------------------------------------
// Test 5: Recursive dependents (route -> trips -> stop_times)
// ---------------------------------------------------------------------------
#[test]
fn test_recursive_dependents() {
    let mut feed = GtfsFeed::default();
    feed.routes.push(make_route("R1", None));
    feed.trips.push(make_trip("T1", "R1", "SVC1"));
    feed.trips.push(make_trip("T2", "R1", "SVC1"));
    feed.stop_times.push(make_stop_time("T1", "S1", 1));
    feed.stop_times.push(make_stop_time("T1", "S2", 2));
    feed.stop_times.push(make_stop_time("T1", "S3", 3));
    feed.stop_times.push(make_stop_time("T2", "S1", 1));
    feed.stop_times.push(make_stop_time("T2", "S2", 2));

    let index = IntegrityIndex::build_from_feed(&feed);
    let all = index.find_dependents_recursive(&EntityRef::Route(RouteId::from("R1")));

    // 2 trips + 5 stop_times = 7
    assert_eq!(all.len(), 7);

    let trip_count = all
        .iter()
        .filter(|(entity, _)| matches!(entity, EntityRef::Trip(_)))
        .count();
    assert_eq!(trip_count, 2);

    let stop_time_count = all
        .iter()
        .filter(|(entity, _)| matches!(entity, EntityRef::StopTime(_, _)))
        .count();
    assert_eq!(stop_time_count, 5);
}

// ---------------------------------------------------------------------------
// Test 6: Entity with no dependents
// ---------------------------------------------------------------------------
#[test]
fn test_no_dependents() {
    let mut feed = GtfsFeed::default();
    feed.stops.push(make_stop("S1"));

    let index = IntegrityIndex::build_from_feed(&feed);
    let dependents = index.find_dependents(&EntityRef::Stop(StopId::from("S1")));
    assert!(dependents.is_empty());
}

// ---------------------------------------------------------------------------
// Test 7: Parent station self-reference
// ---------------------------------------------------------------------------
#[test]
fn test_parent_station() {
    let mut feed = GtfsFeed::default();

    let mut parent = make_stop("S1");
    parent.location_type = Some(LocationType::Station);
    feed.stops.push(parent);

    let mut child = make_stop("S2");
    child.parent_station = Some(StopId::from("S1"));
    feed.stops.push(child);

    let index = IntegrityIndex::build_from_feed(&feed);

    // Forward: S2 -> S1 (ParentStation)
    let child_refs = index.get_references(&EntityRef::Stop(StopId::from("S2")));
    assert!(child_refs.iter().any(|(entity, relation)| {
        *entity == EntityRef::Stop(StopId::from("S1")) && *relation == RelationType::ParentStation
    }));

    // Reverse: S1 has dependent S2
    let parent_deps = index.find_dependents(&EntityRef::Stop(StopId::from("S1")));
    assert!(parent_deps.iter().any(|(entity, relation)| {
        *entity == EntityRef::Stop(StopId::from("S2")) && *relation == RelationType::ParentStation
    }));
}

// ---------------------------------------------------------------------------
// Test 8: Transfers
// ---------------------------------------------------------------------------
#[test]
fn test_transfers() {
    let mut feed = GtfsFeed::default();
    feed.stops.push(make_stop("S1"));
    feed.stops.push(make_stop("S2"));
    feed.transfers.push(Transfer {
        from_stop_id: Some(StopId::from("S1")),
        to_stop_id: Some(StopId::from("S2")),
        from_route_id: None,
        to_route_id: None,
        from_trip_id: None,
        to_trip_id: None,
        transfer_type: TransferType::Recommended,
        min_transfer_time: None,
    });

    let index = IntegrityIndex::build_from_feed(&feed);

    // Forward: Transfer(0) -> S1 (TransferFromStop) + S2 (TransferToStop)
    let transfer_refs = index.get_references(&EntityRef::Transfer(0));
    assert!(transfer_refs.iter().any(|(entity, relation)| {
        *entity == EntityRef::Stop(StopId::from("S1"))
            && *relation == RelationType::TransferFromStop
    }));
    assert!(transfer_refs.iter().any(|(entity, relation)| {
        *entity == EntityRef::Stop(StopId::from("S2")) && *relation == RelationType::TransferToStop
    }));

    // Reverse: S1 has Transfer(0) as dependent
    let stop_deps = index.find_dependents(&EntityRef::Stop(StopId::from("S1")));
    assert!(stop_deps.iter().any(|(entity, relation)| {
        *entity == EntityRef::Transfer(0) && *relation == RelationType::TransferFromStop
    }));
}

// ---------------------------------------------------------------------------
// Test 9: Fare rules
// ---------------------------------------------------------------------------
#[test]
fn test_fare_rules() {
    let mut feed = GtfsFeed::default();
    feed.fare_attributes.push(FareAttribute {
        fare_id: FareId::from("F1"),
        price: 3.25,
        currency_type: CurrencyCode::from("CAD"),
        payment_method: 0,
        transfers: None,
        agency_id: None,
        transfer_duration: None,
    });
    feed.fare_rules.push(FareRule {
        fare_id: FareId::from("F1"),
        route_id: Some(RouteId::from("R1")),
        origin_id: None,
        destination_id: None,
        contains_id: None,
    });

    let index = IntegrityIndex::build_from_feed(&feed);

    let fare_rule_refs = index.get_references(&EntityRef::FareRule(0));
    assert!(fare_rule_refs.iter().any(|(entity, relation)| {
        *entity == EntityRef::Fare(FareId::from("F1")) && *relation == RelationType::FareOfFareRule
    }));
    assert!(fare_rule_refs.iter().any(|(entity, relation)| {
        *entity == EntityRef::Route(RouteId::from("R1"))
            && *relation == RelationType::RouteOfFareRule
    }));
}

// ---------------------------------------------------------------------------
// Test 10: Thread safety - Arc<IntegrityIndex> shared between threads
// ---------------------------------------------------------------------------
#[test]
fn test_thread_safety() {
    let feed = minimal_feed();
    let index = Arc::new(IntegrityIndex::build_from_feed(&feed));

    let index_clone = Arc::clone(&index);
    let handle = std::thread::spawn(move || {
        index_clone.entity_exists(&EntityRef::Agency(AgencyId::from("A1")))
    });

    assert!(index.entity_exists(&EntityRef::Agency(AgencyId::from("A1"))));
    assert!(handle.join().unwrap());
}

// ---------------------------------------------------------------------------
// Test 11: Entity exists / not exists
// ---------------------------------------------------------------------------
#[test]
fn test_entity_exists_true_false() {
    let feed = minimal_feed();
    let index = IntegrityIndex::build_from_feed(&feed);

    assert!(index.entity_exists(&EntityRef::Stop(StopId::from("S1"))));
    assert!(!index.entity_exists(&EntityRef::Stop(StopId::from("INEXISTANT"))));
}

// ---------------------------------------------------------------------------
// Test 12: Forward/reverse symmetry
// ---------------------------------------------------------------------------
#[test]
fn test_forward_reverse_symmetry() {
    let feed = minimal_feed();
    let index = IntegrityIndex::build_from_feed(&feed);

    for (source, targets) in &index.forward {
        for (target, relation) in targets {
            let reverse_entries = index
                .reverse
                .get(target)
                .expect("target should exist in reverse index");
            assert!(
                reverse_entries
                    .iter()
                    .any(|(rev_source, rev_relation)| rev_source == source
                        && *rev_relation == *relation),
                "Missing reverse entry: {source:?} --{relation:?}--> {target:?}"
            );
        }
    }

    for (target, sources) in &index.reverse {
        for (source, relation) in sources {
            let forward_entries = index
                .forward
                .get(source)
                .expect("source should exist in forward index");
            assert!(
                forward_entries
                    .iter()
                    .any(|(fwd_target, fwd_relation)| fwd_target == target
                        && *fwd_relation == *relation),
                "Missing forward entry: {source:?} --{relation:?}--> {target:?}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Test 13: Optional FK None not indexed
// ---------------------------------------------------------------------------
#[test]
fn test_optional_fk_none() {
    let mut feed = GtfsFeed::default();
    feed.trips.push(make_trip("T1", "R1", "SVC1"));

    let index = IntegrityIndex::build_from_feed(&feed);
    let references = index.get_references(&EntityRef::Trip(TripId::from("T1")));

    // Should have RouteOfTrip and ServiceOfTrip, but NOT ShapeOfTrip
    assert!(
        !references
            .iter()
            .any(|(_, relation)| *relation == RelationType::ShapeOfTrip)
    );
    assert_eq!(references.len(), 2);
}

// ---------------------------------------------------------------------------
// Test 14: Agency with no ID
// ---------------------------------------------------------------------------
#[test]
fn test_agency_optional_id() {
    let mut feed = GtfsFeed::default();
    feed.agencies.push(Agency {
        agency_id: None,
        agency_name: "No ID Agency".to_string(),
        agency_url: Url::from("http://example.com"),
        agency_timezone: Timezone::from("UTC"),
        agency_lang: None,
        agency_phone: None,
        agency_fare_url: None,
        agency_email: None,
    });

    let index = IntegrityIndex::build_from_feed(&feed);
    // Agency with None id should not be registered
    assert!(index.forward.is_empty());
}
