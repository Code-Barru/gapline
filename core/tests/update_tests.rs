use gapline_core::crud::query::parse;
use gapline_core::crud::read::GtfsTarget;
use gapline_core::crud::update::{UpdateError, apply_update, validate_update};
use gapline_core::models::*;

use chrono::NaiveDate;

fn s(val: &str) -> String {
    val.to_string()
}

fn sets(pairs: &[&str]) -> Vec<String> {
    pairs.iter().map(|p| s(p)).collect()
}

fn empty_feed() -> GtfsFeed {
    GtfsFeed::default()
}

fn gtfs_date(y: i32, m: u32, d: u32) -> GtfsDate {
    GtfsDate(NaiveDate::from_ymd_opt(y, m, d).unwrap())
}

fn make_stop(id: &str, name: &str, lat: f64, lon: f64) -> Stop {
    Stop {
        stop_id: StopId::from(id),
        stop_code: None,
        stop_name: Some(name.to_string()),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(lat)),
        stop_lon: Some(Longitude(lon)),
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

fn make_route(id: &str, agency_id: &str) -> Route {
    Route {
        route_id: RouteId::from(id),
        agency_id: Some(AgencyId::from(agency_id)),
        route_short_name: Some("1".into()),
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

fn make_trip(route_id: &str, service_id: &str, trip_id: &str) -> Trip {
    Trip {
        route_id: RouteId::from(route_id),
        service_id: ServiceId::from(service_id),
        trip_id: TripId::from(trip_id),
        trip_headsign: None,
        trip_short_name: None,
        direction_id: None,
        block_id: None,
        shape_id: None,
        wheelchair_accessible: None,
        bikes_allowed: None,
    }
}

fn make_stop_time(trip_id: &str, stop_id: &str, seq: u32) -> StopTime {
    StopTime {
        trip_id: TripId::from(trip_id),
        arrival_time: None,
        departure_time: None,
        stop_id: StopId::from(stop_id),
        stop_sequence: seq,
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

fn make_agency(id: &str) -> Agency {
    Agency {
        agency_id: Some(AgencyId::from(id)),
        agency_name: "Test Agency".into(),
        agency_url: Url::from("http://example.com"),
        agency_timezone: Timezone::from("America/Montreal"),
        agency_lang: None,
        agency_phone: None,
        agency_fare_url: None,
        agency_email: None,
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
        start_date: gtfs_date(2025, 1, 1),
        end_date: gtfs_date(2025, 12, 31),
    }
}

fn feed_with_deps() -> GtfsFeed {
    let mut feed = empty_feed();
    feed.agencies.push(make_agency("A1"));
    feed.stops.push(make_stop("S01", "Stop One", 45.5, -73.6));
    feed.stops.push(make_stop("S02", "Stop Two", 45.6, -73.5));
    feed.stops
        .push(make_stop("S_UNUSED", "Unused Stop", 45.7, -73.4));
    feed.routes.push(make_route("R1", "A1"));
    feed.routes.push(make_route("R2", "A1"));
    feed.calendars.push(make_calendar("SVC1"));
    feed.trips.push(make_trip("R1", "SVC1", "T1"));
    feed.stop_times.push(make_stop_time("T1", "S01", 1));
    feed.stop_times.push(make_stop_time("T1", "S02", 2));
    feed
}

#[test]
fn update_simple_stop_name() {
    let mut feed = feed_with_deps();
    let query = parse("stop_id=S01").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Stops,
        &query,
        &sets(&["stop_name=New Name"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    let result = apply_update(&mut feed, &plan).unwrap();
    assert_eq!(result.count, 1);
    assert_eq!(feed.stops[0].stop_name.as_deref(), Some("New Name"));
    // Ensure other stops unchanged
    assert_eq!(feed.stops[1].stop_name.as_deref(), Some("Stop Two"));
}

#[test]
fn update_multiple_stop_times() {
    let mut feed = feed_with_deps();
    let query = parse("trip_id=T1").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::StopTimes,
        &query,
        &sets(&["departure_time=09:00:00"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 2);
    apply_update(&mut feed, &plan).unwrap();
    for st in &feed.stop_times {
        assert_eq!(st.departure_time.unwrap().to_string(), "09:00:00");
    }
}

#[test]
fn update_pk_referenced_by_dependents() {
    let feed = feed_with_deps();
    let query = parse("stop_id=S01").unwrap();
    let result = validate_update(
        &feed,
        GtfsTarget::Stops,
        &query,
        &sets(&["stop_id=S99"]),
        false,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, UpdateError::PrimaryKeyReferenced { .. }));
}

#[test]
fn update_pk_unreferenced() {
    let mut feed = feed_with_deps();
    let query = parse("stop_id=S_UNUSED").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Stops,
        &query,
        &sets(&["stop_id=S99"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    apply_update(&mut feed, &plan).unwrap();
    assert_eq!(feed.stops[2].stop_id.as_ref(), "S99");
}

#[test]
fn update_fk_invalid() {
    let feed = feed_with_deps();
    let query = parse("trip_id=T1").unwrap();
    let result = validate_update(
        &feed,
        GtfsTarget::Trips,
        &query,
        &sets(&["route_id=R_INEXISTANT"]),
        false,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, UpdateError::ForeignKeyViolation { .. }));
}

#[test]
fn update_fk_valid() {
    let mut feed = feed_with_deps();
    let query = parse("trip_id=T1").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Trips,
        &query,
        &sets(&["route_id=R2"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    apply_update(&mut feed, &plan).unwrap();
    assert_eq!(feed.trips[0].route_id.as_ref(), "R2");
}

#[test]
fn update_no_match() {
    let feed = feed_with_deps();
    let query = parse("stop_id=INEXISTANT").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Stops,
        &query,
        &sets(&["stop_name=X"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 0);
}

#[test]
fn update_invalid_type() {
    let feed = feed_with_deps();
    let query = parse("stop_id=S01").unwrap();
    let result = validate_update(
        &feed,
        GtfsTarget::Stops,
        &query,
        &sets(&["stop_lat=pas_un_nombre"]),
        false,
    );
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        UpdateError::InvalidFieldValue { .. }
    ));
}

#[test]
fn update_unknown_field() {
    let feed = feed_with_deps();
    let query = parse("stop_id=S01").unwrap();
    let result = validate_update(
        &feed,
        GtfsTarget::Stops,
        &query,
        &sets(&["nonexistent=value"]),
        false,
    );
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        UpdateError::UnknownField { .. }
    ));
}

#[test]
fn update_empty_assignments() {
    let feed = feed_with_deps();
    let query = parse("stop_id=S01").unwrap();
    let result = validate_update(&feed, GtfsTarget::Stops, &query, &[], false);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UpdateError::EmptyAssignments));
}

#[test]
fn update_pk_to_existing_value() {
    let feed = feed_with_deps();
    // S_UNUSED exists but has no dependents, try changing its stop_id to S02 which already exists
    let query = parse("stop_id=S_UNUSED").unwrap();
    let result = validate_update(
        &feed,
        GtfsTarget::Stops,
        &query,
        &sets(&["stop_id=S02"]),
        false,
    );
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        UpdateError::DuplicatePrimaryKey { .. }
    ));
}

#[test]
fn update_agency_name() {
    let mut feed = empty_feed();
    feed.agencies.push(make_agency("A1"));
    let query = parse("agency_id=A1").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Agency,
        &query,
        &sets(&["agency_name=New Agency Name"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    apply_update(&mut feed, &plan).unwrap();
    assert_eq!(feed.agencies[0].agency_name, "New Agency Name");
}

#[test]
fn update_route_short_name() {
    let mut feed = empty_feed();
    feed.agencies.push(make_agency("A1"));
    feed.routes.push(make_route("R1", "A1"));
    let query = parse("route_id=R1").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Routes,
        &query,
        &sets(&["route_short_name=Express"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    apply_update(&mut feed, &plan).unwrap();
    assert_eq!(feed.routes[0].route_short_name.as_deref(), Some("Express"));
}

#[test]
fn update_calendar_monday() {
    let mut feed = empty_feed();
    feed.calendars.push(make_calendar("SVC1"));
    let query = parse("service_id=SVC1").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Calendar,
        &query,
        &sets(&["monday=0"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    apply_update(&mut feed, &plan).unwrap();
    assert!(!feed.calendars[0].monday);
}

#[test]
fn update_calendar_date_exception_type() {
    let mut feed = empty_feed();
    feed.calendars.push(make_calendar("SVC1"));
    feed.calendar_dates.push(CalendarDate {
        service_id: ServiceId::from("SVC1"),
        date: gtfs_date(2025, 6, 15),
        exception_type: ExceptionType::Added,
    });
    let query = parse("service_id=SVC1").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::CalendarDates,
        &query,
        &sets(&["exception_type=2"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    apply_update(&mut feed, &plan).unwrap();
    assert!(matches!(
        feed.calendar_dates[0].exception_type,
        ExceptionType::Removed
    ));
}

#[test]
fn update_shape_pt_lat() {
    let mut feed = empty_feed();
    feed.shapes.push(Shape {
        shape_id: ShapeId::from("SH1"),
        shape_pt_lat: Latitude(45.5),
        shape_pt_lon: Longitude(-73.6),
        shape_pt_sequence: 1,
        shape_dist_traveled: None,
    });
    let query = parse("shape_id=SH1").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Shapes,
        &query,
        &sets(&["shape_pt_lat=46.0"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    apply_update(&mut feed, &plan).unwrap();
    assert!((feed.shapes[0].shape_pt_lat.0 - 46.0).abs() < f64::EPSILON);
}

#[test]
fn update_frequency_headway_secs() {
    let mut feed = feed_with_deps();
    feed.frequencies.push(Frequency {
        trip_id: TripId::from("T1"),
        start_time: "08:00:00".parse::<GtfsTime>().unwrap(),
        end_time: "10:00:00".parse::<GtfsTime>().unwrap(),
        headway_secs: 600,
        exact_times: None,
    });
    let query = parse("trip_id=T1").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Frequencies,
        &query,
        &sets(&["headway_secs=300"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    apply_update(&mut feed, &plan).unwrap();
    assert_eq!(feed.frequencies[0].headway_secs, 300);
}

#[test]
fn update_transfer_type() {
    let mut feed = feed_with_deps();
    feed.transfers.push(Transfer {
        from_stop_id: Some(StopId::from("S01")),
        to_stop_id: Some(StopId::from("S02")),
        from_route_id: None,
        to_route_id: None,
        from_trip_id: None,
        to_trip_id: None,
        transfer_type: TransferType::Recommended,
        min_transfer_time: None,
    });
    let query = parse("from_stop_id=S01").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Transfers,
        &query,
        &sets(&["transfer_type=2"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    apply_update(&mut feed, &plan).unwrap();
    assert!(matches!(
        feed.transfers[0].transfer_type,
        TransferType::MinimumTime
    ));
}

#[test]
fn update_pathway_length() {
    let mut feed = feed_with_deps();
    feed.pathways.push(Pathway {
        pathway_id: PathwayId::from("PW1"),
        from_stop_id: StopId::from("S01"),
        to_stop_id: StopId::from("S02"),
        pathway_mode: PathwayMode::Walkway,
        is_bidirectional: IsBidirectional::Bidirectional,
        length: Some(100.0),
        traversal_time: None,
        stair_count: None,
        max_slope: None,
        min_width: None,
        signposted_as: None,
        reversed_signposted_as: None,
    });
    let query = parse("pathway_id=PW1").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Pathways,
        &query,
        &sets(&["length=250.5"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    apply_update(&mut feed, &plan).unwrap();
    assert!((feed.pathways[0].length.unwrap() - 250.5).abs() < f64::EPSILON);
}

#[test]
fn update_level_name() {
    let mut feed = empty_feed();
    feed.levels.push(Level {
        level_id: LevelId::from("L1"),
        level_index: 0.0,
        level_name: Some("Ground".into()),
    });
    let query = parse("level_id=L1").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Levels,
        &query,
        &sets(&["level_name=Mezzanine"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    apply_update(&mut feed, &plan).unwrap();
    assert_eq!(feed.levels[0].level_name.as_deref(), Some("Mezzanine"));
}

#[test]
fn update_feed_info_publisher_name() {
    let mut feed = empty_feed();
    feed.feed_info = Some(FeedInfo {
        feed_publisher_name: "Old Publisher".into(),
        feed_publisher_url: Url::from("http://example.com"),
        feed_lang: LanguageCode::from("en"),
        default_lang: None,
        feed_start_date: None,
        feed_end_date: None,
        feed_version: None,
        feed_contact_email: None,
        feed_contact_url: None,
    });
    let query = parse("feed_publisher_name=Old Publisher").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::FeedInfo,
        &query,
        &sets(&["feed_publisher_name=New Publisher"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    apply_update(&mut feed, &plan).unwrap();
    assert_eq!(
        feed.feed_info.as_ref().unwrap().feed_publisher_name,
        "New Publisher"
    );
}

#[test]
fn update_fare_attribute_price() {
    let mut feed = empty_feed();
    feed.fare_attributes.push(FareAttribute {
        fare_id: FareId::from("F1"),
        price: 2.50,
        currency_type: CurrencyCode::from("CAD"),
        payment_method: 0,
        transfers: Some(0),
        agency_id: None,
        transfer_duration: None,
    });
    let query = parse("fare_id=F1").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::FareAttributes,
        &query,
        &sets(&["price=3.75"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    apply_update(&mut feed, &plan).unwrap();
    assert!((feed.fare_attributes[0].price - 3.75).abs() < f64::EPSILON);
}

#[test]
fn update_fare_rule_origin_id() {
    let mut feed = empty_feed();
    feed.agencies.push(make_agency("A1"));
    feed.routes.push(make_route("R1", "A1"));
    feed.fare_attributes.push(FareAttribute {
        fare_id: FareId::from("F1"),
        price: 2.50,
        currency_type: CurrencyCode::from("CAD"),
        payment_method: 0,
        transfers: Some(0),
        agency_id: None,
        transfer_duration: None,
    });
    feed.fare_rules.push(FareRule {
        fare_id: FareId::from("F1"),
        route_id: Some(RouteId::from("R1")),
        origin_id: Some("zone_A".into()),
        destination_id: None,
        contains_id: None,
    });
    let query = parse("fare_id=F1").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::FareRules,
        &query,
        &sets(&["origin_id=zone_B"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    apply_update(&mut feed, &plan).unwrap();
    assert_eq!(feed.fare_rules[0].origin_id.as_deref(), Some("zone_B"));
}

#[test]
fn update_translation_text() {
    let mut feed = empty_feed();
    feed.translations.push(Translation {
        table_name: "stops".into(),
        field_name: "stop_name".into(),
        language: LanguageCode::from("fr"),
        translation: "Ancien Nom".into(),
        record_id: Some("S01".into()),
        record_sub_id: None,
        field_value: None,
    });
    let query = parse("language=fr").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Translations,
        &query,
        &sets(&["translation=Nouveau Nom"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    apply_update(&mut feed, &plan).unwrap();
    assert_eq!(feed.translations[0].translation, "Nouveau Nom");
}

#[test]
fn update_attribution_organization_name() {
    let mut feed = empty_feed();
    feed.attributions.push(Attribution {
        attribution_id: Some("ATTR1".into()),
        agency_id: None,
        route_id: None,
        trip_id: None,
        organization_name: "Old Org".into(),
        is_producer: Some(1),
        is_operator: None,
        is_authority: None,
        attribution_url: None,
        attribution_email: None,
        attribution_phone: None,
    });
    let query = parse("organization_name=Old Org").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Attributions,
        &query,
        &sets(&["organization_name=New Org"]),
        false,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    apply_update(&mut feed, &plan).unwrap();
    assert_eq!(feed.attributions[0].organization_name, "New Org");
}

#[test]
fn cascade_updates_dependent_fk() {
    let mut feed = feed_with_deps();
    // S01 is referenced by 2 stop_times - cascade should update them
    let query = parse("stop_id=S01").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Stops,
        &query,
        &sets(&["stop_id=S99"]),
        true,
    )
    .unwrap();
    assert_eq!(plan.matched_count, 1);
    assert!(plan.cascade.is_some());
    let cascade = plan.cascade.as_ref().unwrap();
    assert_eq!(cascade.old_value, "S01");
    assert_eq!(cascade.new_value, "S99");

    let result = apply_update(&mut feed, &plan).unwrap();
    // Primary target + stop_times modified
    assert!(result.modified_targets.contains(&GtfsTarget::Stops));
    assert!(result.modified_targets.contains(&GtfsTarget::StopTimes));
    // Stop renamed
    assert_eq!(feed.stops[0].stop_id.as_ref(), "S99");
    // Stop_times FK cascaded
    assert_eq!(feed.stop_times[0].stop_id.as_ref(), "S99");
}

#[test]
fn cascade_updates_multiple_fk_fields() {
    let mut feed = feed_with_deps();
    // Add a transfer referencing S01 in both from and to
    feed.transfers.push(Transfer {
        from_stop_id: Some(StopId::from("S01")),
        to_stop_id: Some(StopId::from("S01")),
        from_route_id: None,
        to_route_id: None,
        from_trip_id: None,
        to_trip_id: None,
        transfer_type: TransferType::Recommended,
        min_transfer_time: None,
    });
    let query = parse("stop_id=S01").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Stops,
        &query,
        &sets(&["stop_id=S99"]),
        true,
    )
    .unwrap();
    let result = apply_update(&mut feed, &plan).unwrap();
    assert!(result.modified_targets.contains(&GtfsTarget::Transfers));
    // Both from_stop_id and to_stop_id should be cascaded
    assert_eq!(
        feed.transfers[0].from_stop_id.as_ref().unwrap().as_ref(),
        "S99"
    );
    assert_eq!(
        feed.transfers[0].to_stop_id.as_ref().unwrap().as_ref(),
        "S99"
    );
}

#[test]
fn cascade_no_dependents() {
    let mut feed = feed_with_deps();
    // S_UNUSED has no dependents
    let query = parse("stop_id=S_UNUSED").unwrap();
    let plan = validate_update(
        &feed,
        GtfsTarget::Stops,
        &query,
        &sets(&["stop_id=S99"]),
        true,
    )
    .unwrap();
    assert!(plan.cascade.is_none());
    let result = apply_update(&mut feed, &plan).unwrap();
    assert_eq!(result.modified_targets, vec![GtfsTarget::Stops]);
    assert_eq!(feed.stops[2].stop_id.as_ref(), "S99");
}

#[test]
fn cascade_still_checks_new_pk_unique() {
    let feed = feed_with_deps();
    // S01 has dependents, cascade=true, but S02 already exists → DuplicatePrimaryKey
    let query = parse("stop_id=S01").unwrap();
    let result = validate_update(
        &feed,
        GtfsTarget::Stops,
        &query,
        &sets(&["stop_id=S02"]),
        true,
    );
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        UpdateError::DuplicatePrimaryKey { .. }
    ));
}

#[test]
fn no_cascade_errors_on_referenced_pk() {
    let feed = feed_with_deps();
    // S01 referenced, cascade=false → PrimaryKeyReferenced error (regression test)
    let query = parse("stop_id=S01").unwrap();
    let result = validate_update(
        &feed,
        GtfsTarget::Stops,
        &query,
        &sets(&["stop_id=S99"]),
        false,
    );
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        UpdateError::PrimaryKeyReferenced { .. }
    ));
}
