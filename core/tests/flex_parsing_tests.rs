use std::io::Write;
use std::path::Path;

use gapline_core::integrity::IntegrityIndex;
use gapline_core::models::{BookingRuleId, BookingType, LocationGroupId};
use gapline_core::parser::FeedLoader;

fn write_files(dir: &Path, files: &[(&str, &str)]) {
    for (name, content) in files {
        let mut f = std::fs::File::create(dir.join(name)).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }
}

fn minimal_base() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "agency.txt",
            "agency_id,agency_name,agency_url,agency_timezone\nA,A,http://a,America/Montreal\n",
        ),
        ("stops.txt", "stop_id,stop_name\nS1,Gare\nS2,Other\n"),
        ("routes.txt", "route_id,route_type\nR1,3\n"),
        ("trips.txt", "route_id,service_id,trip_id\nR1,SVC1,T1\n"),
        (
            "calendar.txt",
            "service_id,monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date\nSVC1,1,1,1,1,1,0,0,20240101,20241231\n",
        ),
    ]
}

const STOP_TIMES_PLAIN: &str = "trip_id,arrival_time,departure_time,stop_id,stop_sequence\n\
    T1,08:00:00,08:01:00,S1,1\n";

const STOP_TIMES_FLEX: &str = "trip_id,arrival_time,departure_time,stop_id,stop_sequence,start_pickup_drop_off_window,end_pickup_drop_off_window,pickup_booking_rule_id,drop_off_booking_rule_id,mean_duration_factor,mean_duration_offset,safe_duration_factor,safe_duration_offset\n\
    T1,,,S1,1,08:00:00,09:00:00,BR1,BR2,1.5,30.0,2.0,60.0\n\
    T1,08:30:00,08:31:00,S2,2,,,,,,,,\n";

const BOOKING_RULES: &str = "booking_rule_id,booking_type,prior_notice_duration_min,phone_number,info_url,custom_col\n\
    BR1,0,,,,ignore-me\n\
    BR2,1,30,+15145551234,http://a/booking,\n\
    BR3,2,1440,,,\n";

const LOCATION_GROUPS: &str = "location_group_id,location_group_name\n\
    LG1,North zone\n\
    LG2,South zone\n";

const LOCATION_GROUP_STOPS: &str = "location_group_id,stop_id\n\
    LG1,S1\n\
    LG1,S2\n\
    LG2,S2\n";

#[test]
fn full_flex_feed_parses_all_collections() {
    let tmp = tempfile::tempdir().unwrap();
    let mut files = minimal_base();
    files.push(("stop_times.txt", STOP_TIMES_FLEX));
    files.push(("booking_rules.txt", BOOKING_RULES));
    files.push(("location_groups.txt", LOCATION_GROUPS));
    files.push(("location_group_stops.txt", LOCATION_GROUP_STOPS));
    write_files(tmp.path(), &files);

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);

    assert!(errors.is_empty());
    assert_eq!(feed.booking_rules.len(), 3);
    assert_eq!(feed.location_groups.len(), 2);
    assert_eq!(feed.location_group_stops.len(), 3);
    assert_eq!(feed.stop_times.len(), 2);

    let st1 = &feed.stop_times[0];
    assert!(st1.start_pickup_drop_off_window.is_some());
    assert!(st1.end_pickup_drop_off_window.is_some());
    assert_eq!(
        st1.pickup_booking_rule_id.as_ref().map(AsRef::as_ref),
        Some("BR1")
    );
    assert_eq!(
        st1.drop_off_booking_rule_id.as_ref().map(AsRef::as_ref),
        Some("BR2")
    );
    assert_eq!(st1.mean_duration_factor, Some(1.5));
    assert_eq!(st1.safe_duration_offset, Some(60.0));

    assert!(feed.has_flex());
}

#[test]
fn feed_without_flex() {
    let tmp = tempfile::tempdir().unwrap();
    let mut files = minimal_base();
    files.push(("stop_times.txt", STOP_TIMES_PLAIN));
    write_files(tmp.path(), &files);

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);

    assert!(errors.is_empty());
    assert!(feed.booking_rules.is_empty());
    assert!(feed.location_groups.is_empty());
    assert!(feed.location_group_stops.is_empty());
    assert!(!feed.has_flex());

    let st = &feed.stop_times[0];
    assert!(st.start_pickup_drop_off_window.is_none());
    assert!(st.pickup_booking_rule_id.is_none());
    assert!(st.mean_duration_factor.is_none());
}

#[test]
fn booking_rules_parse_typed_fields() {
    let tmp = tempfile::tempdir().unwrap();
    let mut files = minimal_base();
    files.push(("stop_times.txt", STOP_TIMES_PLAIN));
    files.push(("booking_rules.txt", BOOKING_RULES));
    write_files(tmp.path(), &files);

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, _) = FeedLoader::load(&source);
    assert_eq!(feed.booking_rules.len(), 3);
    assert_eq!(feed.booking_rules[0].booking_type, BookingType::RealTime);
    assert_eq!(feed.booking_rules[1].booking_type, BookingType::SameDay);
    assert_eq!(feed.booking_rules[2].booking_type, BookingType::PriorDays);
    assert_eq!(feed.booking_rules[1].prior_notice_duration_min, Some(30));
}

#[test]
fn unknown_column_in_booking_rules_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    let mut files = minimal_base();
    files.push(("stop_times.txt", STOP_TIMES_PLAIN));
    files.push(("booking_rules.txt", BOOKING_RULES));
    write_files(tmp.path(), &files);

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);
    let booking_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.file_name == "booking_rules.txt")
        .collect();
    assert!(booking_errors.is_empty());
    assert_eq!(feed.booking_rules.len(), 3);
}

#[test]
fn reverse_index_resolves_booking_rule_dependents() {
    let tmp = tempfile::tempdir().unwrap();
    let mut files = minimal_base();
    files.push(("stop_times.txt", STOP_TIMES_FLEX));
    files.push(("booking_rules.txt", BOOKING_RULES));
    files.push(("location_groups.txt", LOCATION_GROUPS));
    files.push(("location_group_stops.txt", LOCATION_GROUP_STOPS));
    write_files(tmp.path(), &files);

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, _) = FeedLoader::load(&source);
    let index = IntegrityIndex::build_from_feed(&feed);

    let br1 = BookingRuleId::from("BR1");
    let dependents_br1 = index.stop_times_for_booking_rule(&br1);
    assert_eq!(dependents_br1.len(), 1);
    assert_eq!(dependents_br1[0].0.as_ref(), "T1");
    assert_eq!(dependents_br1[0].1, 1);

    let br2 = BookingRuleId::from("BR2");
    let dependents_br2 = index.stop_times_for_booking_rule(&br2);
    assert_eq!(dependents_br2.len(), 1);

    let br3 = BookingRuleId::from("BR3");
    assert!(index.stop_times_for_booking_rule(&br3).is_empty());

    let lg1 = LocationGroupId::from("LG1");
    let stops_lg1 = index.stops_for_location_group(&lg1);
    assert_eq!(stops_lg1.len(), 2);

    let lg2 = LocationGroupId::from("LG2");
    let stops_lg2 = index.stops_for_location_group(&lg2);
    assert_eq!(stops_lg2.len(), 1);
}
