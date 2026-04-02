use std::fmt::Write as _;
use std::io::BufReader;

use headway_core::models::*;
use headway_core::parser::file_parsers;

fn reader(data: &[u8]) -> BufReader<&[u8]> {
    BufReader::new(data)
}

// -- TC1: agency.txt minimal
#[test]
fn parse_agency_minimal() {
    let csv = b"agency_id,agency_name,agency_url,agency_timezone\nSTM,STM,http://stm.info,America/Montreal\n";
    let (agencies, errors) = file_parsers::agency::parse(reader(csv));

    assert_eq!(agencies.len(), 1);
    assert!(errors.is_empty());
    assert_eq!(agencies[0].agency_id.as_ref().unwrap().as_ref(), "STM");
    assert_eq!(agencies[0].agency_name, "STM");
    assert_eq!(agencies[0].agency_url.as_ref(), "http://stm.info");
    assert_eq!(agencies[0].agency_timezone.as_ref(), "America/Montreal");
}

// -- TC2: stops.txt with absent optional fields
#[test]
fn parse_stops_optional_absent() {
    let csv = b"stop_id,stop_name,stop_lat,stop_lon\nS1,Gare,45.5,-73.6\n";
    let (stops, errors) = file_parsers::stops::parse(reader(csv));

    assert_eq!(stops.len(), 1);
    assert!(errors.is_empty());
    assert_eq!(stops[0].stop_id.as_ref(), "S1");
    assert!(stops[0].stop_desc.is_none());
    assert!((stops[0].stop_lat.unwrap().0 - 45.5).abs() < f64::EPSILON);
}

// -- TC3: routes.txt with route_type enum
#[test]
fn parse_routes_enum() {
    let csv = b"route_id,route_type\nR1,3\n";
    let (routes, errors) = file_parsers::routes::parse(reader(csv));

    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].route_type, RouteType::Bus);
    // route_short_name and route_long_name are optional -> no error
    // agency_id is optional and not required
    // route_type is required -> no error
    let type_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.field_name == "route_type")
        .collect();
    assert!(type_errors.is_empty());
}

// -- TC4: stop_times.txt
#[test]
fn parse_stop_times() {
    let csv =
        b"trip_id,arrival_time,departure_time,stop_id,stop_sequence\nT1,08:30:00,08:31:00,S1,1\n";
    let (st, errors) = file_parsers::stop_times::parse(reader(csv));

    assert_eq!(st.len(), 1);
    assert!(errors.is_empty());
    assert_eq!(st[0].arrival_time.unwrap(), GtfsTime::from_hms(8, 30, 0));
    assert_eq!(st[0].stop_sequence, 1);
}

// -- TC5: GtfsTime > 24h
#[test]
fn parse_stop_times_over_24h() {
    let csv =
        b"trip_id,arrival_time,departure_time,stop_id,stop_sequence\nT1,25:30:00,25:31:00,S1,1\n";
    let (st, errors) = file_parsers::stop_times::parse(reader(csv));

    assert!(errors.is_empty());
    assert_eq!(st[0].arrival_time.unwrap().hours(), 25);
    assert_eq!(st[0].arrival_time.unwrap().minutes(), 30);
}

// -- TC6: calendar.txt
#[test]
fn parse_calendar() {
    let csv = b"service_id,monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date\nSVC1,1,1,1,1,1,0,0,20240101,20241231\n";
    let (cals, errors) = file_parsers::calendar::parse(reader(csv));

    assert_eq!(cals.len(), 1);
    assert!(errors.is_empty());
    assert!(cals[0].monday);
    assert!(!cals[0].saturday);
    assert_eq!(cals[0].start_date.to_string(), "20240101");
}

// -- TC7: shapes.txt
#[test]
fn parse_shapes() {
    let csv = b"shape_id,shape_pt_lat,shape_pt_lon,shape_pt_sequence\nSH1,45.5,-73.6,1\n";
    let (shapes, errors) = file_parsers::shapes::parse(reader(csv));

    assert_eq!(shapes.len(), 1);
    assert!(errors.is_empty());
    assert!((shapes[0].shape_pt_lat.0 - 45.5).abs() < f64::EPSILON);
    assert!((shapes[0].shape_pt_lon.0 - (-73.6)).abs() < f64::EPSILON);
}

// -- TC8: BOM stripping
#[test]
fn parse_agency_with_bom() {
    let csv = b"\xEF\xBB\xBFagency_id,agency_name,agency_url,agency_timezone\nSTM,STM,http://stm.info,America/Montreal\n";
    let (agencies, errors) = file_parsers::agency::parse(reader(csv));

    assert_eq!(agencies.len(), 1);
    assert!(errors.is_empty());
    assert_eq!(agencies[0].agency_id.as_ref().unwrap().as_ref(), "STM");
}

// -- TC11: unknown column ignored
#[test]
fn unknown_column_ignored() {
    let csv = b"agency_id,agency_name,agency_url,agency_timezone,agency_color\nSTM,STM,http://stm.info,America/Montreal,#FF0000\n";
    let (agencies, errors) = file_parsers::agency::parse(reader(csv));

    assert_eq!(agencies.len(), 1);
    assert!(errors.is_empty());
}

// -- TC12: empty value for optional field -> None
#[test]
fn empty_value_optional_field() {
    let csv = b"stop_id,stop_name,stop_desc,stop_lat,stop_lon\nS1,Gare,,45.5,-73.6\n";
    let (stops, _) = file_parsers::stops::parse(reader(csv));
    assert!(stops[0].stop_desc.is_none());
}

// -- TC13: empty value for required field -> default + error
#[test]
fn empty_value_required_field() {
    let csv = b"agency_id,agency_name,agency_url,agency_timezone\nSTM,,http://stm.info,America/Montreal\n";
    let (agencies, errors) = file_parsers::agency::parse(reader(csv));

    assert_eq!(agencies[0].agency_name, "");
    let name_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.field_name == "agency_name")
        .collect();
    assert_eq!(name_errors.len(), 1);
}

// -- TC16: feed_info.txt single row
#[test]
fn parse_feed_info_single_row() {
    let csv = b"feed_publisher_name,feed_publisher_url,feed_lang\nACME,http://acme.com,en\n";
    let (info, line_count, errors) = file_parsers::feed_info::parse(reader(csv));

    assert!(errors.is_empty());
    assert_eq!(line_count, 1);
    let info = info.unwrap();
    assert_eq!(info.feed_publisher_name, "ACME");
    assert_eq!(info.feed_lang.0, "en");
}

// -- TC17: feed_info.txt absent -> None (empty data)
#[test]
fn parse_feed_info_empty() {
    let csv = b"feed_publisher_name,feed_publisher_url,feed_lang\n";
    let (info, line_count, errors) = file_parsers::feed_info::parse(reader(csv));
    assert!(info.is_none());
    assert_eq!(line_count, 0);
    assert!(errors.is_empty());
}

// -- TC18: invalid type collected
#[test]
fn invalid_type_collected() {
    let csv = b"stop_id,stop_name,stop_lat,stop_lon\nS1,Gare,not_a_number,-73.6\n";
    let (stops, errors) = file_parsers::stops::parse(reader(csv));

    assert!(stops[0].stop_lat.is_none());
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field_name, "stop_lat");
    assert_eq!(errors[0].value, "not_a_number");
}

// -- calendar_dates
#[test]
fn parse_calendar_dates() {
    let csv = b"service_id,date,exception_type\nSVC1,20240101,2\n";
    let (dates, errors) = file_parsers::calendar_dates::parse(reader(csv));

    assert!(errors.is_empty());
    assert_eq!(dates[0].exception_type, ExceptionType::Removed);
}

// -- frequencies
#[test]
fn parse_frequencies() {
    let csv = b"trip_id,start_time,end_time,headway_secs\nT1,06:00:00,09:00:00,300\n";
    let (freqs, errors) = file_parsers::frequencies::parse(reader(csv));

    assert!(errors.is_empty());
    assert_eq!(freqs[0].headway_secs, 300);
}

// -- transfers
#[test]
fn parse_transfers() {
    let csv = b"from_stop_id,to_stop_id,transfer_type,min_transfer_time\nS1,S2,2,120\n";
    let (transfers, errors) = file_parsers::transfers::parse(reader(csv));

    assert!(errors.is_empty());
    assert_eq!(transfers[0].transfer_type, TransferType::MinimumTime);
    assert_eq!(transfers[0].min_transfer_time, Some(120));
}

// -- pathways
#[test]
fn parse_pathways() {
    let csv = b"pathway_id,from_stop_id,to_stop_id,pathway_mode,is_bidirectional\nP1,S1,S2,2,1\n";
    let (pathways, errors) = file_parsers::pathways::parse(reader(csv));

    assert!(errors.is_empty());
    assert_eq!(pathways[0].pathway_mode, PathwayMode::Stairs);
    assert_eq!(pathways[0].is_bidirectional, IsBidirectional::Bidirectional);
}

// -- levels
#[test]
fn parse_levels() {
    let csv = b"level_id,level_index,level_name\nL1,0.0,Ground\n";
    let (levels, errors) = file_parsers::levels::parse(reader(csv));

    assert!(errors.is_empty());
    assert_eq!(levels[0].level_id.as_ref(), "L1");
    assert_eq!(levels[0].level_name.as_deref(), Some("Ground"));
}

// -- fare_attributes
#[test]
fn parse_fare_attributes() {
    let csv = b"fare_id,price,currency_type,payment_method\nF1,2.50,CAD,0\n";
    let (fares, errors) = file_parsers::fare_attributes::parse(reader(csv));

    assert!(errors.is_empty());
    assert!((fares[0].price - 2.50).abs() < f64::EPSILON);
    assert_eq!(fares[0].currency_type.0, "CAD");
}

// -- fare_rules
#[test]
fn parse_fare_rules() {
    let csv = b"fare_id,route_id\nF1,R1\n";
    let (rules, errors) = file_parsers::fare_rules::parse(reader(csv));

    assert!(errors.is_empty());
    assert_eq!(rules[0].route_id.as_ref().unwrap().as_ref(), "R1");
}

// -- translations
#[test]
fn parse_translations() {
    let csv = b"table_name,field_name,language,translation\nagency,agency_name,fr,STM\n";
    let (trans, errors) = file_parsers::translations::parse(reader(csv));

    assert!(errors.is_empty());
    assert_eq!(trans[0].table_name, "agency");
    assert_eq!(trans[0].translation, "STM");
}

// -- attributions
#[test]
fn parse_attributions() {
    let csv = b"organization_name,is_producer\nACME,1\n";
    let (attrs, errors) = file_parsers::attributions::parse(reader(csv));

    assert!(errors.is_empty());
    assert_eq!(attrs[0].organization_name, "ACME");
    assert_eq!(attrs[0].is_producer, Some(1));
}

// -- TC15: Performance - 100k stop_times
// Run with: cargo test --release -- --ignored
#[test]
#[ignore = "Must be run in release mode"]
fn parse_stop_times_100k_performance() {
    let header = "trip_id,arrival_time,departure_time,stop_id,stop_sequence\n";
    let mut csv = String::with_capacity(header.len() + 100_000 * 40);
    csv.push_str(header);
    for i in 0..100_000u32 {
        let _ = writeln!(csv, "T1,08:00:00,08:01:00,S{i},{i}");
    }

    let start = std::time::Instant::now();
    let (st, errors) = file_parsers::stop_times::parse(reader(csv.as_bytes()));
    let elapsed = start.elapsed();

    assert_eq!(st.len(), 100_000);
    assert!(errors.is_empty());
    assert!(
        elapsed.as_secs() < 1,
        "parsing 100k stop_times took {elapsed:?}, expected < 1s"
    );
}
