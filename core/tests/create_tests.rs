use gapline_core::crud::create::{CreateError, apply_create, validate_create};
use gapline_core::crud::read::GtfsTarget;
use gapline_core::models::*;

use chrono::NaiveDate;

// ===========================================================================
// Helpers
// ===========================================================================

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

fn feed_with_routes_and_calendar() -> GtfsFeed {
    let mut feed = empty_feed();
    feed.agencies.push(Agency {
        agency_id: Some(AgencyId::from("A1")),
        agency_name: "Agency".into(),
        agency_url: Url::from("http://example.com"),
        agency_timezone: Timezone::from("America/Montreal"),
        agency_lang: None,
        agency_phone: None,
        agency_fare_url: None,
        agency_email: None,
    });
    feed.routes.push(Route {
        route_id: RouteId::from("R1"),
        agency_id: Some(AgencyId::from("A1")),
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
    });
    feed.calendars.push(Calendar {
        service_id: ServiceId::from("SVC1"),
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        start_date: gtfs_date(2025, 1, 1),
        end_date: gtfs_date(2025, 12, 31),
    });
    feed.stops.push(Stop {
        stop_id: StopId::from("S01"),
        stop_code: None,
        stop_name: Some("Existing Stop".into()),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(45.5)),
        stop_lon: Some(Longitude(-73.6)),
        zone_id: None,
        stop_url: None,
        location_type: None,
        parent_station: None,
        stop_timezone: None,
        wheelchair_boarding: None,
        level_id: None,
        platform_code: None,
    });
    feed.trips.push(Trip {
        route_id: RouteId::from("R1"),
        service_id: ServiceId::from("SVC1"),
        trip_id: TripId::from("T1"),
        trip_headsign: None,
        trip_short_name: None,
        direction_id: None,
        block_id: None,
        shape_id: None,
        wheelchair_accessible: None,
        bikes_allowed: None,
    });
    feed
}

// ===========================================================================
// Stops
// ===========================================================================

#[test]
fn create_stop_success() {
    let mut feed = empty_feed();
    let plan = validate_create(
        &feed,
        GtfsTarget::Stops,
        &sets(&[
            "stop_id=S99",
            "stop_name=New",
            "stop_lat=45.5",
            "stop_lon=-73.6",
        ]),
    )
    .unwrap();

    assert_eq!(plan.file_name, "stops.txt");
    assert_eq!(plan.assignments.len(), 4);

    apply_create(&mut feed, plan);
    assert_eq!(feed.stops.len(), 1);
    assert_eq!(feed.stops[0].stop_id.as_ref(), "S99");
}

#[test]
fn create_stop_missing_stop_name() {
    let feed = empty_feed();
    let err = validate_create(
        &feed,
        GtfsTarget::Stops,
        &sets(&["stop_id=S99", "stop_lat=45.5", "stop_lon=-73.6"]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::MissingRequiredField(f) if f == "stop_name"));
}

#[test]
fn create_stop_unknown_field() {
    let feed = empty_feed();
    let err = validate_create(
        &feed,
        GtfsTarget::Stops,
        &sets(&["stop_id=S99", "bogus_field=x"]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::UnknownField { field, .. } if field == "bogus_field"));
}

#[test]
fn create_stop_duplicate_pk() {
    let feed = feed_with_routes_and_calendar();
    let err = validate_create(
        &feed,
        GtfsTarget::Stops,
        &sets(&[
            "stop_id=S01",
            "stop_name=Dup",
            "stop_lat=45.5",
            "stop_lon=-73.6",
        ]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::DuplicatePrimaryKey { .. }));
}

#[test]
fn create_stop_invalid_lat() {
    let feed = empty_feed();
    let err = validate_create(
        &feed,
        GtfsTarget::Stops,
        &sets(&[
            "stop_id=S99",
            "stop_name=X",
            "stop_lat=not_a_number",
            "stop_lon=-73.6",
        ]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::InvalidFieldValue { field, .. } if field == "stop_lat"));
}

#[test]
fn create_stop_location_type_2_requires_parent() {
    let mut feed = empty_feed();
    feed.stops.push(Stop {
        stop_id: StopId::from("STATION"),
        stop_code: None,
        stop_name: Some("Station".into()),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(45.5)),
        stop_lon: Some(Longitude(-73.6)),
        zone_id: None,
        stop_url: None,
        location_type: Some(LocationType::Station),
        parent_station: None,
        stop_timezone: None,
        wheelchair_boarding: None,
        level_id: None,
        platform_code: None,
    });

    let err = validate_create(
        &feed,
        GtfsTarget::Stops,
        &sets(&["stop_id=E1", "location_type=2"]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::MissingRequiredField(f) if f == "parent_station"));
}

#[test]
fn create_stop_location_type_1_forbids_parent() {
    let feed = empty_feed();
    let err = validate_create(
        &feed,
        GtfsTarget::Stops,
        &sets(&[
            "stop_id=ST1",
            "stop_name=Station",
            "stop_lat=45.5",
            "stop_lon=-73.6",
            "location_type=1",
            "parent_station=OTHER",
        ]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::ForbiddenField { field, .. } if field == "parent_station"));
}

// ===========================================================================
// Trips
// ===========================================================================

#[test]
fn create_trip_success() {
    let mut feed = feed_with_routes_and_calendar();
    let plan = validate_create(
        &feed,
        GtfsTarget::Trips,
        &sets(&["trip_id=T99", "route_id=R1", "service_id=SVC1"]),
    )
    .unwrap();

    apply_create(&mut feed, plan);
    assert_eq!(feed.trips.len(), 2);
    assert_eq!(feed.trips[1].trip_id.as_ref(), "T99");
}

#[test]
fn create_trip_fk_route_violation() {
    let feed = feed_with_routes_and_calendar();
    let err = validate_create(
        &feed,
        GtfsTarget::Trips,
        &sets(&["trip_id=T99", "route_id=NONEXISTENT", "service_id=SVC1"]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::ForeignKeyViolation { field, .. } if field == "route_id"));
}

#[test]
fn create_trip_fk_service_violation() {
    let feed = feed_with_routes_and_calendar();
    let err = validate_create(
        &feed,
        GtfsTarget::Trips,
        &sets(&["trip_id=T99", "route_id=R1", "service_id=NONEXISTENT"]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::ForeignKeyViolation { field, .. } if field == "service_id"));
}

// ===========================================================================
// StopTimes
// ===========================================================================

#[test]
fn create_stop_time_success() {
    let mut feed = feed_with_routes_and_calendar();
    let plan = validate_create(
        &feed,
        GtfsTarget::StopTimes,
        &sets(&["trip_id=T1", "stop_id=S01", "stop_sequence=1"]),
    )
    .unwrap();

    apply_create(&mut feed, plan);
    assert_eq!(feed.stop_times.len(), 1);
}

#[test]
fn create_stop_time_composite_pk_dup() {
    let mut feed = feed_with_routes_and_calendar();
    feed.stop_times.push(StopTime {
        trip_id: TripId::from("T1"),
        arrival_time: None,
        departure_time: None,
        stop_id: StopId::from("S01"),
        stop_sequence: 1,
        stop_headsign: None,
        pickup_type: None,
        drop_off_type: None,
        continuous_pickup: None,
        continuous_drop_off: None,
        shape_dist_traveled: None,
        timepoint: None,
    });

    let err = validate_create(
        &feed,
        GtfsTarget::StopTimes,
        &sets(&["trip_id=T1", "stop_id=S01", "stop_sequence=1"]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::DuplicatePrimaryKey { .. }));
}

#[test]
fn create_stop_time_fk_trip_violation() {
    let feed = feed_with_routes_and_calendar();
    let err = validate_create(
        &feed,
        GtfsTarget::StopTimes,
        &sets(&["trip_id=NONEXISTENT", "stop_id=S01", "stop_sequence=1"]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::ForeignKeyViolation { field, .. } if field == "trip_id"));
}

#[test]
fn create_stop_time_fk_stop_violation() {
    let feed = feed_with_routes_and_calendar();
    let err = validate_create(
        &feed,
        GtfsTarget::StopTimes,
        &sets(&["trip_id=T1", "stop_id=NONEXISTENT", "stop_sequence=1"]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::ForeignKeyViolation { field, .. } if field == "stop_id"));
}

// ===========================================================================
// Calendar
// ===========================================================================

#[test]
fn create_calendar_success() {
    let mut feed = empty_feed();
    let plan = validate_create(
        &feed,
        GtfsTarget::Calendar,
        &sets(&[
            "service_id=SVC2",
            "monday=1",
            "tuesday=1",
            "wednesday=1",
            "thursday=1",
            "friday=1",
            "saturday=0",
            "sunday=0",
            "start_date=20260301",
            "end_date=20260630",
        ]),
    )
    .unwrap();

    apply_create(&mut feed, plan);
    assert_eq!(feed.calendars.len(), 1);
    assert_eq!(feed.calendars[0].service_id.as_ref(), "SVC2");
    assert!(feed.calendars[0].monday);
    assert!(!feed.calendars[0].saturday);
}

#[test]
fn create_calendar_missing_day() {
    let feed = empty_feed();
    let err = validate_create(
        &feed,
        GtfsTarget::Calendar,
        &sets(&[
            "service_id=SVC2",
            "tuesday=1",
            "wednesday=1",
            "thursday=1",
            "friday=1",
            "saturday=0",
            "sunday=0",
            "start_date=20260301",
            "end_date=20260630",
        ]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::MissingRequiredField(f) if f == "monday"));
}

// ===========================================================================
// CalendarDates
// ===========================================================================

#[test]
fn create_calendar_date_success() {
    let mut feed = feed_with_routes_and_calendar();
    let plan = validate_create(
        &feed,
        GtfsTarget::CalendarDates,
        &sets(&["service_id=SVC1", "date=20260401", "exception_type=1"]),
    )
    .unwrap();

    apply_create(&mut feed, plan);
    assert_eq!(feed.calendar_dates.len(), 1);
}

#[test]
fn create_calendar_date_invalid_enum() {
    let feed = feed_with_routes_and_calendar();
    let err = validate_create(
        &feed,
        GtfsTarget::CalendarDates,
        &sets(&["service_id=SVC1", "date=20260401", "exception_type=5"]),
    )
    .unwrap_err();

    assert!(
        matches!(err, CreateError::InvalidFieldValue { field, .. } if field == "exception_type")
    );
}

#[test]
fn create_calendar_date_composite_pk_dup() {
    let mut feed = feed_with_routes_and_calendar();
    feed.calendar_dates.push(CalendarDate {
        service_id: ServiceId::from("SVC1"),
        date: gtfs_date(2026, 4, 1),
        exception_type: ExceptionType::Added,
    });

    let err = validate_create(
        &feed,
        GtfsTarget::CalendarDates,
        &sets(&["service_id=SVC1", "date=20260401", "exception_type=1"]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::DuplicatePrimaryKey { .. }));
}

// ===========================================================================
// Agency
// ===========================================================================

#[test]
fn create_agency_success() {
    let mut feed = empty_feed();
    let plan = validate_create(
        &feed,
        GtfsTarget::Agency,
        &sets(&[
            "agency_id=A2",
            "agency_name=New Agency",
            "agency_url=http://new.com",
            "agency_timezone=America/Montreal",
        ]),
    )
    .unwrap();

    apply_create(&mut feed, plan);
    assert_eq!(feed.agencies.len(), 1);
}

// ===========================================================================
// Routes
// ===========================================================================

#[test]
fn create_route_success() {
    let mut feed = feed_with_routes_and_calendar();
    let plan = validate_create(
        &feed,
        GtfsTarget::Routes,
        &sets(&["route_id=R2", "route_type=3", "agency_id=A1"]),
    )
    .unwrap();

    apply_create(&mut feed, plan);
    assert_eq!(feed.routes.len(), 2);
}

#[test]
fn create_route_fk_agency_violation() {
    let feed = feed_with_routes_and_calendar();
    let err = validate_create(
        &feed,
        GtfsTarget::Routes,
        &sets(&["route_id=R2", "route_type=3", "agency_id=NONEXISTENT"]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::ForeignKeyViolation { field, .. } if field == "agency_id"));
}

// ===========================================================================
// Shapes
// ===========================================================================

#[test]
fn create_shape_success() {
    let mut feed = empty_feed();
    let plan = validate_create(
        &feed,
        GtfsTarget::Shapes,
        &sets(&[
            "shape_id=SH1",
            "shape_pt_lat=45.5",
            "shape_pt_lon=-73.6",
            "shape_pt_sequence=1",
        ]),
    )
    .unwrap();

    apply_create(&mut feed, plan);
    assert_eq!(feed.shapes.len(), 1);
}

#[test]
fn create_shape_composite_pk_dup() {
    let mut feed = empty_feed();
    feed.shapes.push(Shape {
        shape_id: ShapeId::from("SH1"),
        shape_pt_lat: Latitude(45.5),
        shape_pt_lon: Longitude(-73.6),
        shape_pt_sequence: 1,
        shape_dist_traveled: None,
    });

    let err = validate_create(
        &feed,
        GtfsTarget::Shapes,
        &sets(&[
            "shape_id=SH1",
            "shape_pt_lat=45.6",
            "shape_pt_lon=-73.7",
            "shape_pt_sequence=1",
        ]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::DuplicatePrimaryKey { .. }));
}

// ===========================================================================
// Frequencies
// ===========================================================================

#[test]
fn create_frequency_success() {
    let mut feed = feed_with_routes_and_calendar();
    let plan = validate_create(
        &feed,
        GtfsTarget::Frequencies,
        &sets(&[
            "trip_id=T1",
            "start_time=06:00:00",
            "end_time=09:00:00",
            "headway_secs=600",
        ]),
    )
    .unwrap();

    apply_create(&mut feed, plan);
    assert_eq!(feed.frequencies.len(), 1);
}

#[test]
fn create_frequency_fk_trip_violation() {
    let feed = feed_with_routes_and_calendar();
    let err = validate_create(
        &feed,
        GtfsTarget::Frequencies,
        &sets(&[
            "trip_id=NONEXISTENT",
            "start_time=06:00:00",
            "end_time=09:00:00",
            "headway_secs=600",
        ]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::ForeignKeyViolation { field, .. } if field == "trip_id"));
}

// ===========================================================================
// Transfers
// ===========================================================================

#[test]
fn create_transfer_success() {
    let mut feed = feed_with_routes_and_calendar();
    let plan = validate_create(
        &feed,
        GtfsTarget::Transfers,
        &sets(&["from_stop_id=S01", "to_stop_id=S01", "transfer_type=0"]),
    )
    .unwrap();

    apply_create(&mut feed, plan);
    assert_eq!(feed.transfers.len(), 1);
}

#[test]
fn create_transfer_fk_stop_violation() {
    let feed = feed_with_routes_and_calendar();
    let err = validate_create(
        &feed,
        GtfsTarget::Transfers,
        &sets(&[
            "from_stop_id=NONEXISTENT",
            "to_stop_id=S01",
            "transfer_type=0",
        ]),
    )
    .unwrap_err();

    assert!(
        matches!(err, CreateError::ForeignKeyViolation { field, .. } if field == "from_stop_id")
    );
}

// ===========================================================================
// Pathways
// ===========================================================================

#[test]
fn create_pathway_success() {
    let mut feed = feed_with_routes_and_calendar();
    let plan = validate_create(
        &feed,
        GtfsTarget::Pathways,
        &sets(&[
            "pathway_id=PW1",
            "from_stop_id=S01",
            "to_stop_id=S01",
            "pathway_mode=1",
            "is_bidirectional=1",
        ]),
    )
    .unwrap();

    apply_create(&mut feed, plan);
    assert_eq!(feed.pathways.len(), 1);
}

#[test]
fn create_pathway_fk_stop_violation() {
    let feed = feed_with_routes_and_calendar();
    let err = validate_create(
        &feed,
        GtfsTarget::Pathways,
        &sets(&[
            "pathway_id=PW1",
            "from_stop_id=NONEXISTENT",
            "to_stop_id=S01",
            "pathway_mode=1",
            "is_bidirectional=1",
        ]),
    )
    .unwrap_err();

    assert!(
        matches!(err, CreateError::ForeignKeyViolation { field, .. } if field == "from_stop_id")
    );
}

// ===========================================================================
// Levels
// ===========================================================================

#[test]
fn create_level_success() {
    let mut feed = empty_feed();
    let plan = validate_create(
        &feed,
        GtfsTarget::Levels,
        &sets(&["level_id=L1", "level_index=0"]),
    )
    .unwrap();

    apply_create(&mut feed, plan);
    assert_eq!(feed.levels.len(), 1);
}

// ===========================================================================
// FeedInfo
// ===========================================================================

#[test]
fn create_feed_info_success() {
    let mut feed = empty_feed();
    let plan = validate_create(
        &feed,
        GtfsTarget::FeedInfo,
        &sets(&[
            "feed_publisher_name=Test Publisher",
            "feed_publisher_url=http://example.com",
            "feed_lang=en",
        ]),
    )
    .unwrap();

    apply_create(&mut feed, plan);
    assert!(feed.feed_info.is_some());
}

#[test]
fn create_feed_info_already_exists() {
    let mut feed = empty_feed();
    feed.feed_info = Some(FeedInfo {
        feed_publisher_name: "Existing".into(),
        feed_publisher_url: Url::from("http://example.com"),
        feed_lang: LanguageCode::from("en"),
        default_lang: None,
        feed_start_date: None,
        feed_end_date: None,
        feed_version: None,
        feed_contact_email: None,
        feed_contact_url: None,
    });

    let err = validate_create(
        &feed,
        GtfsTarget::FeedInfo,
        &sets(&[
            "feed_publisher_name=New",
            "feed_publisher_url=http://new.com",
            "feed_lang=fr",
        ]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::FeedInfoAlreadyExists));
}

// ===========================================================================
// FareAttributes
// ===========================================================================

#[test]
fn create_fare_attribute_success() {
    let mut feed = empty_feed();
    let plan = validate_create(
        &feed,
        GtfsTarget::FareAttributes,
        &sets(&[
            "fare_id=F1",
            "price=2.50",
            "currency_type=CAD",
            "payment_method=0",
            "transfers=0",
        ]),
    )
    .unwrap();

    apply_create(&mut feed, plan);
    assert_eq!(feed.fare_attributes.len(), 1);
}

// ===========================================================================
// FareRules
// ===========================================================================

#[test]
fn create_fare_rule_success() {
    let mut feed = feed_with_routes_and_calendar();
    feed.fare_attributes.push(FareAttribute {
        fare_id: FareId::from("F1"),
        price: 2.50,
        currency_type: CurrencyCode::from("CAD"),
        payment_method: 0,
        transfers: Some(0),
        agency_id: None,
        transfer_duration: None,
    });

    let plan = validate_create(
        &feed,
        GtfsTarget::FareRules,
        &sets(&["fare_id=F1", "route_id=R1"]),
    )
    .unwrap();

    apply_create(&mut feed, plan);
    assert_eq!(feed.fare_rules.len(), 1);
}

#[test]
fn create_fare_rule_fk_violation() {
    let feed = empty_feed();
    let err = validate_create(
        &feed,
        GtfsTarget::FareRules,
        &sets(&["fare_id=NONEXISTENT"]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::ForeignKeyViolation { field, .. } if field == "fare_id"));
}

// ===========================================================================
// Translations
// ===========================================================================

#[test]
fn create_translation_success() {
    let mut feed = empty_feed();
    let plan = validate_create(
        &feed,
        GtfsTarget::Translations,
        &sets(&[
            "table_name=stops",
            "field_name=stop_name",
            "language=fr",
            "translation=Gare Centrale",
        ]),
    )
    .unwrap();

    apply_create(&mut feed, plan);
    assert_eq!(feed.translations.len(), 1);
}

// ===========================================================================
// Attributions
// ===========================================================================

#[test]
fn create_attribution_success() {
    let mut feed = empty_feed();
    let plan = validate_create(
        &feed,
        GtfsTarget::Attributions,
        &sets(&["organization_name=Test Org"]),
    )
    .unwrap();

    apply_create(&mut feed, plan);
    assert_eq!(feed.attributions.len(), 1);
}

// ===========================================================================
// Transversal
// ===========================================================================

#[test]
fn create_duplicate_assignment() {
    let feed = empty_feed();
    let err = validate_create(
        &feed,
        GtfsTarget::Stops,
        &sets(&["stop_id=S1", "stop_id=S2"]),
    )
    .unwrap_err();

    assert!(matches!(err, CreateError::DuplicateAssignment(f) if f == "stop_id"));
}

#[test]
fn create_invalid_assignment_format() {
    let feed = empty_feed();
    let err = validate_create(&feed, GtfsTarget::Stops, &sets(&["no_equals_sign"])).unwrap_err();

    assert!(matches!(err, CreateError::InvalidAssignment(_)));
}

#[test]
fn create_empty_assignments() {
    let feed = empty_feed();
    let err = validate_create(&feed, GtfsTarget::Stops, &[]).unwrap_err();

    assert!(matches!(err, CreateError::EmptyAssignments));
}
