use std::time::Instant;

use gapline_core::crud::query::{Filter, Filterable, Query, QueryError, parse};
use gapline_core::models::*;

// ===========================================================================
// Test helper: a simple struct implementing Filterable for eval tests
// ===========================================================================

struct TestRecord {
    fields: Vec<(&'static str, Option<String>)>,
}

impl TestRecord {
    fn new(fields: &[(&'static str, &str)]) -> Self {
        Self {
            fields: fields
                .iter()
                .map(|(k, v)| (*k, Some(v.to_string())))
                .collect(),
        }
    }

    fn with_none(fields: &[(&'static str, Option<&str>)]) -> Self {
        Self {
            fields: fields
                .iter()
                .map(|(k, v)| (*k, v.map(std::string::ToString::to_string)))
                .collect(),
        }
    }
}

impl Filterable for TestRecord {
    fn field_value(&self, field: &str) -> Option<String> {
        self.fields
            .iter()
            .find(|(k, _)| *k == field)
            .and_then(|(_, v)| v.clone())
    }

    fn valid_fields() -> &'static [&'static str] {
        &[
            "stop_id",
            "stop_name",
            "stop_sequence",
            "trip_id",
            "route_type",
        ]
    }
}

// ===========================================================================
// Parsing: equality (CA1)
// ===========================================================================

#[test]
fn parse_equality_simple() {
    let q = parse("stop_id=S01").unwrap();
    assert_eq!(q, Query::Filter(Filter::Eq("stop_id".into(), "S01".into())));
}

#[test]
fn parse_equality_with_spaces() {
    let q = parse("stop_id = S01").unwrap();
    assert_eq!(q, Query::Filter(Filter::Eq("stop_id".into(), "S01".into())));
}

// ===========================================================================
// Parsing: inequality (CA2)
// ===========================================================================

#[test]
fn parse_inequality() {
    let q = parse("stop_id!=S01").unwrap();
    assert_eq!(
        q,
        Query::Filter(Filter::Neq("stop_id".into(), "S01".into()))
    );
}

#[test]
fn parse_inequality_with_spaces() {
    let q = parse("stop_id != S01").unwrap();
    assert_eq!(
        q,
        Query::Filter(Filter::Neq("stop_id".into(), "S01".into()))
    );
}

// ===========================================================================
// Parsing: comparison operators (CA3)
// ===========================================================================

#[test]
fn parse_greater_than() {
    let q = parse("stop_sequence>10").unwrap();
    assert_eq!(
        q,
        Query::Filter(Filter::Gt("stop_sequence".into(), "10".into()))
    );
}

#[test]
fn parse_greater_or_equal() {
    let q = parse("stop_sequence>=10").unwrap();
    assert_eq!(
        q,
        Query::Filter(Filter::Gte("stop_sequence".into(), "10".into()))
    );
}

#[test]
fn parse_less_than() {
    let q = parse("stop_sequence<20").unwrap();
    assert_eq!(
        q,
        Query::Filter(Filter::Lt("stop_sequence".into(), "20".into()))
    );
}

#[test]
fn parse_less_or_equal() {
    let q = parse("stop_sequence<=20").unwrap();
    assert_eq!(
        q,
        Query::Filter(Filter::Lte("stop_sequence".into(), "20".into()))
    );
}

// ===========================================================================
// Parsing: AND (CA4)
// ===========================================================================

#[test]
fn parse_and_two_filters() {
    let q = parse("trip_id=T1 AND stop_sequence>5").unwrap();
    assert_eq!(
        q,
        Query::And(vec![
            Query::Filter(Filter::Eq("trip_id".into(), "T1".into())),
            Query::Filter(Filter::Gt("stop_sequence".into(), "5".into())),
        ])
    );
}

#[test]
fn parse_and_triple() {
    let q = parse("trip_id=T1 AND stop_sequence>5 AND stop_sequence<20").unwrap();
    assert_eq!(
        q,
        Query::And(vec![
            Query::Filter(Filter::Eq("trip_id".into(), "T1".into())),
            Query::Filter(Filter::Gt("stop_sequence".into(), "5".into())),
            Query::Filter(Filter::Lt("stop_sequence".into(), "20".into())),
        ])
    );
}

// ===========================================================================
// Parsing: OR
// ===========================================================================

#[test]
fn parse_or_two_filters() {
    let q = parse("trip_id=T1 OR trip_id=T2").unwrap();
    assert_eq!(
        q,
        Query::Or(vec![
            Query::Filter(Filter::Eq("trip_id".into(), "T1".into())),
            Query::Filter(Filter::Eq("trip_id".into(), "T2".into())),
        ])
    );
}

// ===========================================================================
// Parsing: AND/OR precedence (AND binds tighter)
// ===========================================================================

#[test]
fn parse_and_or_precedence() {
    // a=1 OR b=2 AND c=3 → Or([Filter(a=1), And([Filter(b=2), Filter(c=3)])])
    let q = parse("stop_id=S01 OR trip_id=T1 AND stop_sequence>5").unwrap();
    assert_eq!(
        q,
        Query::Or(vec![
            Query::Filter(Filter::Eq("stop_id".into(), "S01".into())),
            Query::And(vec![
                Query::Filter(Filter::Eq("trip_id".into(), "T1".into())),
                Query::Filter(Filter::Gt("stop_sequence".into(), "5".into())),
            ]),
        ])
    );
}

// ===========================================================================
// Parsing: spaces around operators (CA5)
// ===========================================================================

#[test]
fn parse_spaces_around_all_operators() {
    assert_eq!(
        parse("a = b").unwrap(),
        Query::Filter(Filter::Eq("a".into(), "b".into()))
    );
    assert_eq!(
        parse("a != b").unwrap(),
        Query::Filter(Filter::Neq("a".into(), "b".into()))
    );
    assert_eq!(
        parse("a > b").unwrap(),
        Query::Filter(Filter::Gt("a".into(), "b".into()))
    );
    assert_eq!(
        parse("a >= b").unwrap(),
        Query::Filter(Filter::Gte("a".into(), "b".into()))
    );
    assert_eq!(
        parse("a < b").unwrap(),
        Query::Filter(Filter::Lt("a".into(), "b".into()))
    );
    assert_eq!(
        parse("a <= b").unwrap(),
        Query::Filter(Filter::Lte("a".into(), "b".into()))
    );
}

// ===========================================================================
// Parsing: backtick-quoted values
// ===========================================================================

#[test]
fn parse_backtick_value_with_and() {
    let q = parse("stop_name=`Salt AND Pepper`").unwrap();
    assert_eq!(
        q,
        Query::Filter(Filter::Eq("stop_name".into(), "Salt AND Pepper".into()))
    );
}

#[test]
fn parse_backtick_value_with_or() {
    let q = parse("stop_name=`This OR That`").unwrap();
    assert_eq!(
        q,
        Query::Filter(Filter::Eq("stop_name".into(), "This OR That".into()))
    );
}

#[test]
fn parse_backtick_value_combined_with_and() {
    let q = parse("stop_name=`Gare Centrale AND Co` AND route_id=R1").unwrap();
    assert_eq!(
        q,
        Query::And(vec![
            Query::Filter(Filter::Eq(
                "stop_name".into(),
                "Gare Centrale AND Co".into()
            )),
            Query::Filter(Filter::Eq("route_id".into(), "R1".into())),
        ])
    );
}

// ===========================================================================
// Parsing: value with spaces (CA7, no backticks needed)
// ===========================================================================

#[test]
fn parse_value_with_spaces() {
    let q = parse("stop_name=Gare Centrale").unwrap();
    assert_eq!(
        q,
        Query::Filter(Filter::Eq("stop_name".into(), "Gare Centrale".into()))
    );
}

// ===========================================================================
// Parsing: error cases (CA6)
// ===========================================================================

#[test]
fn parse_error_empty_value() {
    let err = parse("stop_id=").unwrap_err();
    assert!(matches!(err, QueryError::EmptyValue(f) if f == "stop_id"));
}

#[test]
fn parse_error_empty_field() {
    let err = parse("=S01").unwrap_err();
    assert!(matches!(err, QueryError::EmptyField));
}

#[test]
fn parse_error_and_alone() {
    let err = parse("AND").unwrap_err();
    assert!(matches!(err, QueryError::UnexpectedOperator));
}

#[test]
fn parse_error_or_alone() {
    let err = parse("OR").unwrap_err();
    assert!(matches!(err, QueryError::UnexpectedOperator));
}

#[test]
fn parse_error_trailing_and() {
    let err = parse("stop_id=S01 AND").unwrap_err();
    assert!(matches!(err, QueryError::UnexpectedOperator));
}

#[test]
fn parse_error_leading_and() {
    let err = parse("AND stop_id=S01").unwrap_err();
    assert!(matches!(err, QueryError::UnexpectedOperator));
}

#[test]
fn parse_error_unknown_operator() {
    let err = parse("stop_id>>5").unwrap_err();
    assert!(matches!(err, QueryError::UnknownOperator(_)));
}

#[test]
fn parse_error_empty_input() {
    let err = parse("").unwrap_err();
    assert!(matches!(err, QueryError::InvalidExpression(_)));
}

#[test]
fn parse_error_whitespace_only() {
    let err = parse("   ").unwrap_err();
    assert!(matches!(err, QueryError::InvalidExpression(_)));
}

// ===========================================================================
// Evaluation: equality match (CA13)
// ===========================================================================

#[test]
fn eval_eq_match() {
    let q = parse("stop_id=S01").unwrap();
    let rec = TestRecord::new(&[("stop_id", "S01")]);
    assert!(q.matches(&rec));
}

#[test]
fn eval_eq_no_match() {
    let q = parse("stop_id=S01").unwrap();
    let rec = TestRecord::new(&[("stop_id", "S02")]);
    assert!(!q.matches(&rec));
}

// ===========================================================================
// Evaluation: inequality
// ===========================================================================

#[test]
fn eval_neq_match() {
    let q = parse("stop_id!=S01").unwrap();
    let rec = TestRecord::new(&[("stop_id", "S02")]);
    assert!(q.matches(&rec));
}

#[test]
fn eval_neq_no_match() {
    let q = parse("stop_id!=S01").unwrap();
    let rec = TestRecord::new(&[("stop_id", "S01")]);
    assert!(!q.matches(&rec));
}

#[test]
fn eval_neq_none_field_is_true() {
    let q = parse("stop_name!=X").unwrap();
    let rec = TestRecord::with_none(&[("stop_name", None)]);
    assert!(q.matches(&rec));
}

// ===========================================================================
// Evaluation: numeric comparisons (CA10, CA15)
// ===========================================================================

#[test]
fn eval_gt_numeric() {
    let q = parse("stop_sequence>10").unwrap();
    let rec = TestRecord::new(&[("stop_sequence", "15")]);
    assert!(q.matches(&rec));
}

#[test]
fn eval_gt_numeric_no_match() {
    let q = parse("stop_sequence>10").unwrap();
    let rec = TestRecord::new(&[("stop_sequence", "5")]);
    assert!(!q.matches(&rec));
}

#[test]
fn eval_gte_numeric_equal() {
    let q = parse("stop_sequence>=10").unwrap();
    let rec = TestRecord::new(&[("stop_sequence", "10")]);
    assert!(q.matches(&rec));
}

#[test]
fn eval_lt_numeric() {
    let q = parse("stop_sequence<20").unwrap();
    let rec = TestRecord::new(&[("stop_sequence", "15")]);
    assert!(q.matches(&rec));
}

#[test]
fn eval_lte_numeric_equal() {
    let q = parse("stop_sequence<=20").unwrap();
    let rec = TestRecord::new(&[("stop_sequence", "20")]);
    assert!(q.matches(&rec));
}

// ===========================================================================
// Evaluation: lexicographic fallback (CA10)
// ===========================================================================

#[test]
fn eval_gt_lexicographic_fallback() {
    let q = parse("stop_name>Abc").unwrap();
    let rec = TestRecord::new(&[("stop_name", "Bcd")]);
    assert!(q.matches(&rec));
}

#[test]
fn eval_gt_lexicographic_no_match() {
    let q = parse("stop_name>Bcd").unwrap();
    let rec = TestRecord::new(&[("stop_name", "Abc")]);
    assert!(!q.matches(&rec));
}

// ===========================================================================
// Evaluation: None field with comparison returns false
// ===========================================================================

#[test]
fn eval_gt_none_field_is_false() {
    let q = parse("stop_sequence>10").unwrap();
    let rec = TestRecord::with_none(&[("stop_sequence", None)]);
    assert!(!q.matches(&rec));
}

// ===========================================================================
// Evaluation: AND (CA16, CA17)
// ===========================================================================

#[test]
fn eval_and_all_match() {
    let q = parse("trip_id=T1 AND stop_sequence>5").unwrap();
    let rec = TestRecord::new(&[("trip_id", "T1"), ("stop_sequence", "15")]);
    assert!(q.matches(&rec));
}

#[test]
fn eval_and_one_fails() {
    let q = parse("trip_id=T1 AND stop_sequence>5").unwrap();
    let rec = TestRecord::new(&[("trip_id", "T2"), ("stop_sequence", "15")]);
    assert!(!q.matches(&rec));
}

// ===========================================================================
// Evaluation: OR
// ===========================================================================

#[test]
fn eval_or_first_matches() {
    let q = parse("trip_id=T1 OR trip_id=T2").unwrap();
    let rec = TestRecord::new(&[("trip_id", "T1")]);
    assert!(q.matches(&rec));
}

#[test]
fn eval_or_second_matches() {
    let q = parse("trip_id=T1 OR trip_id=T2").unwrap();
    let rec = TestRecord::new(&[("trip_id", "T2")]);
    assert!(q.matches(&rec));
}

#[test]
fn eval_or_none_match() {
    let q = parse("trip_id=T1 OR trip_id=T2").unwrap();
    let rec = TestRecord::new(&[("trip_id", "T3")]);
    assert!(!q.matches(&rec));
}

// ===========================================================================
// Evaluation: mixed AND + OR
// ===========================================================================

#[test]
fn eval_and_or_combined() {
    // stop_id=S01 OR (trip_id=T1 AND stop_sequence>5)
    let q = parse("stop_id=S01 OR trip_id=T1 AND stop_sequence>5").unwrap();

    // Matches via first OR branch
    let rec1 = TestRecord::new(&[("stop_id", "S01"), ("trip_id", "X"), ("stop_sequence", "1")]);
    assert!(q.matches(&rec1));

    // Matches via second OR branch (AND)
    let rec2 = TestRecord::new(&[
        ("stop_id", "S99"),
        ("trip_id", "T1"),
        ("stop_sequence", "10"),
    ]);
    assert!(q.matches(&rec2));

    // No match
    let rec3 = TestRecord::new(&[
        ("stop_id", "S99"),
        ("trip_id", "T1"),
        ("stop_sequence", "3"),
    ]);
    assert!(!q.matches(&rec3));
}

// ===========================================================================
// validate_fields
// ===========================================================================

#[test]
fn validate_fields_ok() {
    let q = parse("stop_id=S01 AND stop_name=Test").unwrap();
    assert!(q.validate_fields::<TestRecord>().is_ok());
}

#[test]
fn validate_fields_unknown() {
    let q = parse("unknown_field=X").unwrap();
    let err = q.validate_fields::<TestRecord>().unwrap_err();
    assert!(matches!(err, QueryError::UnknownField { field, .. } if field == "unknown_field"));
}

// ===========================================================================
// GTFS structs: Stop
// ===========================================================================

fn make_test_stop() -> Stop {
    Stop {
        stop_id: StopId::from("S01"),
        stop_code: Some("CODE1".into()),
        stop_name: Some("Gare Centrale".into()),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(48.8566)),
        stop_lon: Some(Longitude(2.3522)),
        zone_id: None,
        stop_url: None,
        location_type: Some(LocationType::Station),
        parent_station: None,
        stop_timezone: None,
        wheelchair_boarding: Some(WheelchairAccessible::Some),
        level_id: None,
        platform_code: Some("A".into()),
    }
}

#[test]
fn stop_field_value_id() {
    let stop = make_test_stop();
    assert_eq!(stop.field_value("stop_id"), Some("S01".into()));
}

#[test]
fn stop_field_value_name() {
    let stop = make_test_stop();
    assert_eq!(stop.field_value("stop_name"), Some("Gare Centrale".into()));
}

#[test]
fn stop_field_value_lat() {
    let stop = make_test_stop();
    assert_eq!(stop.field_value("stop_lat"), Some("48.8566".into()));
}

#[test]
fn stop_field_value_location_type_enum() {
    let stop = make_test_stop();
    // Station = 1
    assert_eq!(stop.field_value("location_type"), Some("1".into()));
}

#[test]
fn stop_field_value_wheelchair_enum() {
    let stop = make_test_stop();
    // WheelchairAccessible::Some = 1
    assert_eq!(stop.field_value("wheelchair_boarding"), Some("1".into()));
}

#[test]
fn stop_field_value_none() {
    let stop = make_test_stop();
    assert_eq!(stop.field_value("tts_stop_name"), None);
}

#[test]
fn stop_field_value_unknown() {
    let stop = make_test_stop();
    assert_eq!(stop.field_value("nonexistent"), None);
}

#[test]
fn stop_valid_fields() {
    let fields = Stop::valid_fields();
    assert!(fields.contains(&"stop_id"));
    assert!(fields.contains(&"location_type"));
    assert!(fields.contains(&"platform_code"));
    assert_eq!(fields.len(), 15);
}

// ===========================================================================
// GTFS structs: Trip
// ===========================================================================

#[test]
fn trip_field_values() {
    let trip = Trip {
        route_id: RouteId::from("R1"),
        service_id: ServiceId::from("SVC1"),
        trip_id: TripId::from("T1"),
        trip_headsign: Some("Downtown".into()),
        trip_short_name: None,
        direction_id: Some(DirectionId::Inbound),
        block_id: Some("BLK1".into()),
        shape_id: None,
        wheelchair_accessible: None,
        bikes_allowed: Some(BikesAllowed::Allowed),
    };
    assert_eq!(trip.field_value("trip_id"), Some("T1".into()));
    assert_eq!(trip.field_value("route_id"), Some("R1".into()));
    assert_eq!(trip.field_value("direction_id"), Some("1".into())); // Inbound = 1
    assert_eq!(trip.field_value("bikes_allowed"), Some("1".into())); // Allowed = 1
    assert_eq!(trip.field_value("shape_id"), None);
    assert_eq!(Trip::valid_fields().len(), 10);
}

// ===========================================================================
// GTFS structs: StopTime
// ===========================================================================

#[test]
fn stop_time_field_values() {
    let st = StopTime {
        trip_id: TripId::from("T1"),
        arrival_time: Some(GtfsTime::from_hms(8, 30, 0)),
        departure_time: Some(GtfsTime::from_hms(8, 31, 0)),
        stop_id: StopId::from("S01"),
        stop_sequence: 5,
        stop_headsign: None,
        pickup_type: Some(PickupType::Regular),
        drop_off_type: None,
        continuous_pickup: None,
        continuous_drop_off: None,
        shape_dist_traveled: Some(1234.5),
        timepoint: Some(Timepoint::Exact),
        start_pickup_drop_off_window: None,
        end_pickup_drop_off_window: None,
        pickup_booking_rule_id: None,
        drop_off_booking_rule_id: None,
        mean_duration_factor: None,
        mean_duration_offset: None,
        safe_duration_factor: None,
        safe_duration_offset: None,
    };
    assert_eq!(st.field_value("stop_sequence"), Some("5".into()));
    assert_eq!(st.field_value("arrival_time"), Some("08:30:00".into()));
    assert_eq!(st.field_value("pickup_type"), Some("0".into())); // Regular = 0
    assert_eq!(st.field_value("timepoint"), Some("1".into())); // Exact = 1
    assert_eq!(st.field_value("shape_dist_traveled"), Some("1234.5".into()));
    assert_eq!(StopTime::valid_fields().len(), 20);
}

// ===========================================================================
// GTFS structs: Calendar
// ===========================================================================

#[test]
fn calendar_bool_fields() {
    let cal = Calendar {
        service_id: ServiceId::from("SVC1"),
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        start_date: GtfsDate::default(),
        end_date: GtfsDate::default(),
    };
    assert_eq!(cal.field_value("monday"), Some("1".into()));
    assert_eq!(cal.field_value("saturday"), Some("0".into()));
    assert_eq!(cal.field_value("service_id"), Some("SVC1".into()));
    assert_eq!(Calendar::valid_fields().len(), 10);
}

// ===========================================================================
// GTFS structs: CalendarDate
// ===========================================================================

#[test]
fn calendar_date_field_values() {
    let cd = CalendarDate {
        service_id: ServiceId::from("SVC1"),
        date: GtfsDate::default(),
        exception_type: ExceptionType::Added,
    };
    assert_eq!(cd.field_value("exception_type"), Some("1".into())); // Added = 1
    assert_eq!(CalendarDate::valid_fields().len(), 3);
}

// ===========================================================================
// GTFS structs: Route
// ===========================================================================

#[test]
fn route_field_values() {
    let route = Route {
        route_id: RouteId::from("R1"),
        agency_id: Some(AgencyId::from("A1")),
        route_short_name: Some("1".into()),
        route_long_name: Some("Line 1".into()),
        route_desc: None,
        route_type: RouteType::Bus,
        route_url: None,
        route_color: Some(Color::from("FF0000")),
        route_text_color: None,
        route_sort_order: Some(42),
        continuous_pickup: None,
        continuous_drop_off: None,
        network_id: None,
    };
    assert_eq!(route.field_value("route_type"), Some("3".into())); // Bus = 3
    assert_eq!(route.field_value("route_color"), Some("FF0000".into()));
    assert_eq!(route.field_value("route_sort_order"), Some("42".into()));
    assert_eq!(Route::valid_fields().len(), 13);
}

// ===========================================================================
// GTFS structs: Shape
// ===========================================================================

#[test]
fn shape_field_values() {
    let shape = Shape {
        shape_id: ShapeId::from("SH1"),
        shape_pt_lat: Latitude(48.0),
        shape_pt_lon: Longitude(2.0),
        shape_pt_sequence: 1,
        shape_dist_traveled: None,
    };
    assert_eq!(shape.field_value("shape_id"), Some("SH1".into()));
    assert_eq!(shape.field_value("shape_pt_sequence"), Some("1".into()));
    assert_eq!(shape.field_value("shape_dist_traveled"), None);
    assert_eq!(Shape::valid_fields().len(), 5);
}

// ===========================================================================
// GTFS structs: Frequency
// ===========================================================================

#[test]
fn frequency_field_values() {
    let freq = Frequency {
        trip_id: TripId::from("T1"),
        start_time: GtfsTime::from_hms(6, 0, 0),
        end_time: GtfsTime::from_hms(22, 0, 0),
        headway_secs: 600,
        exact_times: Some(ExactTimes::FrequencyBased),
    };
    assert_eq!(freq.field_value("headway_secs"), Some("600".into()));
    assert_eq!(freq.field_value("exact_times"), Some("0".into())); // FrequencyBased = 0
    assert_eq!(Frequency::valid_fields().len(), 5);
}

// ===========================================================================
// GTFS structs: Transfer
// ===========================================================================

#[test]
fn transfer_field_values() {
    let tr = Transfer {
        from_stop_id: Some(StopId::from("S1")),
        to_stop_id: Some(StopId::from("S2")),
        from_route_id: None,
        to_route_id: None,
        from_trip_id: None,
        to_trip_id: None,
        transfer_type: TransferType::Timed,
        min_transfer_time: Some(120),
    };
    assert_eq!(tr.field_value("from_stop_id"), Some("S1".into()));
    assert_eq!(tr.field_value("transfer_type"), Some("1".into())); // Timed = 1
    assert_eq!(tr.field_value("min_transfer_time"), Some("120".into()));
    assert_eq!(tr.field_value("from_route_id"), None);
    assert_eq!(Transfer::valid_fields().len(), 8);
}

// ===========================================================================
// GTFS structs: Pathway
// ===========================================================================

#[test]
fn pathway_field_values() {
    let pw = Pathway {
        pathway_id: PathwayId::from("PW1"),
        from_stop_id: StopId::from("S1"),
        to_stop_id: StopId::from("S2"),
        pathway_mode: PathwayMode::Stairs,
        is_bidirectional: IsBidirectional::Bidirectional,
        length: Some(50.0),
        traversal_time: Some(30),
        stair_count: Some(12),
        max_slope: None,
        min_width: None,
        signposted_as: Some("Exit A".into()),
        reversed_signposted_as: None,
    };
    assert_eq!(pw.field_value("pathway_mode"), Some("2".into())); // Stairs = 2
    assert_eq!(pw.field_value("is_bidirectional"), Some("1".into())); // Bidirectional = 1
    assert_eq!(pw.field_value("stair_count"), Some("12".into()));
    assert_eq!(Pathway::valid_fields().len(), 12);
}

// ===========================================================================
// GTFS structs: Level
// ===========================================================================

#[test]
fn level_field_values() {
    let lvl = Level {
        level_id: LevelId::from("L1"),
        level_index: -1.0,
        level_name: Some("Underground".into()),
    };
    assert_eq!(lvl.field_value("level_index"), Some("-1".into()));
    assert_eq!(lvl.field_value("level_name"), Some("Underground".into()));
    assert_eq!(Level::valid_fields().len(), 3);
}

// ===========================================================================
// GTFS structs: FeedInfo
// ===========================================================================

#[test]
fn feed_info_field_values() {
    let fi = FeedInfo {
        feed_publisher_name: "Test Publisher".into(),
        feed_publisher_url: Url::from("https://example.com"),
        feed_lang: LanguageCode::from("en"),
        default_lang: None,
        feed_start_date: None,
        feed_end_date: None,
        feed_version: Some("1.0".into()),
        feed_contact_email: None,
        feed_contact_url: None,
    };
    assert_eq!(
        fi.field_value("feed_publisher_name"),
        Some("Test Publisher".into())
    );
    assert_eq!(fi.field_value("feed_version"), Some("1.0".into()));
    assert_eq!(fi.field_value("default_lang"), None);
    assert_eq!(FeedInfo::valid_fields().len(), 9);
}

// ===========================================================================
// GTFS structs: FareAttribute
// ===========================================================================

#[test]
fn fare_attribute_field_values() {
    let fa = FareAttribute {
        fare_id: FareId::from("F1"),
        price: 2.5,
        currency_type: CurrencyCode::from("EUR"),
        payment_method: 0,
        transfers: Some(2),
        agency_id: None,
        transfer_duration: Some(3600),
    };
    assert_eq!(fa.field_value("price"), Some("2.5".into()));
    assert_eq!(fa.field_value("payment_method"), Some("0".into()));
    assert_eq!(fa.field_value("transfers"), Some("2".into()));
    assert_eq!(FareAttribute::valid_fields().len(), 7);
}

// ===========================================================================
// GTFS structs: FareRule
// ===========================================================================

#[test]
fn fare_rule_field_values() {
    let fr = FareRule {
        fare_id: FareId::from("F1"),
        route_id: Some(RouteId::from("R1")),
        origin_id: Some("Z1".into()),
        destination_id: None,
        contains_id: None,
    };
    assert_eq!(fr.field_value("fare_id"), Some("F1".into()));
    assert_eq!(fr.field_value("route_id"), Some("R1".into()));
    assert_eq!(fr.field_value("destination_id"), None);
    assert_eq!(FareRule::valid_fields().len(), 5);
}

// ===========================================================================
// GTFS structs: Translation
// ===========================================================================

#[test]
fn translation_field_values() {
    let tr = Translation {
        table_name: "stops".into(),
        field_name: "stop_name".into(),
        language: LanguageCode::from("fr"),
        translation: "Gare".into(),
        record_id: Some("S01".into()),
        record_sub_id: None,
        field_value: None,
    };
    assert_eq!(tr.field_value("table_name"), Some("stops".into()));
    assert_eq!(tr.field_value("record_id"), Some("S01".into()));
    assert_eq!(tr.field_value("record_sub_id"), None);
    assert_eq!(Translation::valid_fields().len(), 7);
}

// ===========================================================================
// GTFS structs: Attribution
// ===========================================================================

#[test]
fn attribution_field_values() {
    let attr = Attribution {
        attribution_id: Some("AT1".into()),
        agency_id: None,
        route_id: None,
        trip_id: None,
        organization_name: "Transit Co".into(),
        is_producer: Some(1),
        is_operator: Some(0),
        is_authority: None,
        attribution_url: None,
        attribution_email: None,
        attribution_phone: None,
    };
    assert_eq!(
        attr.field_value("organization_name"),
        Some("Transit Co".into())
    );
    assert_eq!(attr.field_value("is_producer"), Some("1".into()));
    assert_eq!(attr.field_value("is_authority"), None);
    assert_eq!(Attribution::valid_fields().len(), 11);
}

// ===========================================================================
// GTFS structs: Agency
// ===========================================================================

#[test]
fn agency_field_values() {
    let ag = Agency {
        agency_id: Some(AgencyId::from("A1")),
        agency_name: "Transit Agency".into(),
        agency_url: Url::from("https://transit.example.com"),
        agency_timezone: Timezone::from("Europe/Paris"),
        agency_lang: Some(LanguageCode::from("fr")),
        agency_phone: None,
        agency_fare_url: None,
        agency_email: None,
    };
    assert_eq!(ag.field_value("agency_id"), Some("A1".into()));
    assert_eq!(ag.field_value("agency_name"), Some("Transit Agency".into()));
    assert_eq!(ag.field_value("agency_lang"), Some("fr".into()));
    assert_eq!(ag.field_value("agency_phone"), None);
    assert_eq!(Agency::valid_fields().len(), 8);
}

// ===========================================================================
// End-to-end: parse + validate + match on real GTFS struct
// ===========================================================================

#[test]
fn end_to_end_stop_query() {
    let stop = make_test_stop();
    let q = parse("stop_id=S01 AND location_type=1").unwrap();
    q.validate_fields::<Stop>().unwrap();
    assert!(q.matches(&stop));
}

#[test]
fn end_to_end_stop_query_no_match() {
    let stop = make_test_stop();
    let q = parse("stop_id=S01 AND location_type=0").unwrap();
    q.validate_fields::<Stop>().unwrap();
    assert!(!q.matches(&stop));
}

#[test]
fn end_to_end_validate_unknown_field_on_stop() {
    let q = parse("nonexistent=X").unwrap();
    let err = q.validate_fields::<Stop>().unwrap_err();
    assert!(matches!(err, QueryError::UnknownField { field, .. } if field == "nonexistent"));
}

// ===========================================================================
// Performance (CA11, CA18): 1M records, equality filter < 1s
// ===========================================================================

#[test]
fn performance_1m_records() {
    let q = parse("stop_id=TARGET").unwrap();
    let records: Vec<TestRecord> = (0..1_000_000)
        .map(|i| {
            if i == 999_999 {
                TestRecord::new(&[("stop_id", "TARGET")])
            } else {
                TestRecord::new(&[("stop_id", "OTHER")])
            }
        })
        .collect();

    let start = Instant::now();
    let count = records.iter().filter(|r| q.matches(*r)).count();
    let elapsed = start.elapsed();

    assert_eq!(count, 1);
    assert!(
        elapsed.as_secs() < 1,
        "filtering 1M records took {elapsed:?}, expected < 1s"
    );
}

// ===========================================================================
// LIKE
// ===========================================================================

#[test]
fn parse_like_simple() {
    let q = parse("stop_name LIKE Gare%").unwrap();
    assert_eq!(
        q,
        Query::Filter(Filter::Like("stop_name".into(), "Gare%".into()))
    );
}

#[test]
fn parse_like_with_backticks_and_and() {
    let q = parse("stop_name LIKE `foo bar%` AND route_type=1").unwrap();
    assert_eq!(
        q,
        Query::And(vec![
            Query::Filter(Filter::Like("stop_name".into(), "foo bar%".into())),
            Query::Filter(Filter::Eq("route_type".into(), "1".into())),
        ])
    );
}

#[test]
fn parse_like_empty_value_errors() {
    let err = parse("stop_name LIKE ").unwrap_err();
    assert!(matches!(err, QueryError::EmptyValue(f) if f == "stop_name"));
}

#[test]
fn parse_like_empty_field_errors() {
    let err = parse(" LIKE foo").unwrap_err();
    assert!(matches!(err, QueryError::EmptyField));
}

#[test]
fn eval_like_prefix() {
    let q = parse("stop_name LIKE `Gare%`").unwrap();
    assert!(q.matches(&TestRecord::new(&[("stop_name", "Gare du Nord")])));
    assert!(q.matches(&TestRecord::new(&[("stop_name", "Gare")])));
    assert!(!q.matches(&TestRecord::new(&[("stop_name", "Petite Gare")])));
}

#[test]
fn eval_like_substring() {
    let q = parse("stop_name LIKE `%central%`").unwrap();
    assert!(q.matches(&TestRecord::new(&[("stop_name", "central")])));
    assert!(q.matches(&TestRecord::new(&[("stop_name", "Gare centrale")])));
    assert!(!q.matches(&TestRecord::new(&[("stop_name", "Gare du Nord")])));
}

#[test]
fn eval_like_underscore() {
    let q = parse("stop_id LIKE S_1").unwrap();
    assert!(q.matches(&TestRecord::new(&[("stop_id", "SA1")])));
    assert!(!q.matches(&TestRecord::new(&[("stop_id", "S1")])));
    assert!(!q.matches(&TestRecord::new(&[("stop_id", "SAA1")])));
}

#[test]
fn eval_like_case_sensitive() {
    let q = parse("stop_name LIKE Gare").unwrap();
    assert!(q.matches(&TestRecord::new(&[("stop_name", "Gare")])));
    assert!(!q.matches(&TestRecord::new(&[("stop_name", "gare")])));
}

#[test]
fn eval_like_none_field_does_not_match() {
    let q = parse("stop_name LIKE `%`").unwrap();
    let rec = TestRecord::with_none(&[("stop_name", None)]);
    assert!(!q.matches(&rec));
}
