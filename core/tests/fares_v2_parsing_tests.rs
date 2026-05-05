use std::io::Write;
use std::path::Path;

use gapline_core::crud::{GtfsTarget, apply_create, validate_create};
use gapline_core::integrity::{EntityRef, IntegrityIndex, RelationType};
use gapline_core::models::{AreaId, FareMediaType, FareProductId, NetworkId, RiderCategoryId};
use gapline_core::parser::FeedLoader;
use gapline_core::parser::error::ParseErrorKind;

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
            "stop_times.txt",
            "trip_id,arrival_time,departure_time,stop_id,stop_sequence\nT1,08:00:00,08:01:00,S1,1\n",
        ),
        (
            "calendar.txt",
            "service_id,monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date\nSVC1,1,1,1,1,1,0,0,20240101,20241231\n",
        ),
    ]
}

const FARE_MEDIA: &str = "fare_media_id,fare_media_name,fare_media_type\n\
    FM1,No media,0\n\
    FM2,Paper,1\n\
    FM3,Mobile,4\n";

const FARE_PRODUCTS: &str = "fare_product_id,fare_product_name,fare_media_id,amount,currency,rider_category_id\n\
    FP1,Single,FM1,3.50,USD,\n\
    FP2,Day,FM3,7.00,USD,RC1\n";

const FARE_LEG_RULES: &str = "leg_group_id,network_id,from_area_id,to_area_id,from_timeframe_group_id,to_timeframe_group_id,fare_product_id,rule_priority\n\
    LG1,NET1,AR1,AR2,TF1,TF1,FP1,1\n";

const FARE_TRANSFER_RULES: &str = "from_leg_group_id,to_leg_group_id,transfer_count,duration_limit,duration_limit_type,fare_transfer_type,fare_product_id\n\
    LG1,LG1,1,3600,0,2,FP1\n";

const RIDER_CATEGORIES: &str = "rider_category_id,rider_category_name,min_age,max_age,eligibility_url\n\
    RC1,Adult,18,64,\n\
    RC2,Senior,65,,\n";

const TIMEFRAMES: &str = "timeframe_group_id,start_time,end_time,service_id\n\
    TF1,06:00:00,10:00:00,SVC1\n";

const AREAS: &str = "area_id,area_name\n\
    AR1,Downtown\n\
    AR2,Uptown\n";

const STOP_AREAS: &str = "area_id,stop_id\n\
    AR1,S1\n\
    AR2,S2\n";

const NETWORKS: &str = "network_id,network_name\n\
    NET1,Metro\n";

const ROUTE_NETWORKS: &str = "network_id,route_id\n\
    NET1,R1\n";

const FARE_LEG_JOIN_RULES: &str = "from_network_id,to_network_id,from_stop_id,to_stop_id\n\
    NET1,NET1,S1,S2\n";

fn full_v2_files() -> Vec<(&'static str, &'static str)> {
    let mut files = minimal_base();
    files.push(("fare_media.txt", FARE_MEDIA));
    files.push(("fare_products.txt", FARE_PRODUCTS));
    files.push(("fare_leg_rules.txt", FARE_LEG_RULES));
    files.push(("fare_transfer_rules.txt", FARE_TRANSFER_RULES));
    files.push(("rider_categories.txt", RIDER_CATEGORIES));
    files.push(("timeframes.txt", TIMEFRAMES));
    files.push(("areas.txt", AREAS));
    files.push(("stop_areas.txt", STOP_AREAS));
    files.push(("networks.txt", NETWORKS));
    files.push(("route_networks.txt", ROUTE_NETWORKS));
    files.push(("fare_leg_join_rules.txt", FARE_LEG_JOIN_RULES));
    files
}

#[test]
fn full_fares_v2_feed_populates_all_collections() {
    let tmp = tempfile::tempdir().unwrap();
    write_files(tmp.path(), &full_v2_files());

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);

    assert!(errors.is_empty());
    assert_eq!(feed.fare_media.len(), 3);
    assert_eq!(feed.fare_products.len(), 2);
    assert_eq!(feed.fare_leg_rules.len(), 1);
    assert_eq!(feed.fare_transfer_rules.len(), 1);
    assert_eq!(feed.rider_categories.len(), 2);
    assert_eq!(feed.timeframes.len(), 1);
    assert_eq!(feed.areas.len(), 2);
    assert_eq!(feed.stop_areas.len(), 2);
    assert_eq!(feed.networks.len(), 1);
    assert_eq!(feed.route_networks.len(), 1);
    assert_eq!(feed.fare_leg_join_rules.len(), 1);
    assert!(feed.has_fares_v2());
}

#[test]
fn feed_without_fares_v2() {
    let tmp = tempfile::tempdir().unwrap();
    write_files(tmp.path(), &minimal_base());

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);

    assert!(errors.is_empty());
    assert!(!feed.has_fares_v2());
    assert!(feed.fare_media.is_empty());
    assert!(feed.fare_products.is_empty());
    assert!(feed.fare_leg_rules.is_empty());
    assert!(feed.areas.is_empty());
}

#[test]
fn fares_v1_and_v2_coexist() {
    let tmp = tempfile::tempdir().unwrap();
    let mut files = minimal_base();
    files.push((
        "fare_attributes.txt",
        "fare_id,price,currency_type,payment_method,transfers\nF1,2.50,USD,1,0\n",
    ));
    files.push(("fare_products.txt", FARE_PRODUCTS));
    write_files(tmp.path(), &files);

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);

    assert!(errors.is_empty());
    assert_eq!(feed.fare_attributes.len(), 1);
    assert_eq!(feed.fare_products.len(), 2);
    assert!(feed.has_fares_v2());
}

#[test]
fn fare_media_parses_three_entries() {
    let tmp = tempfile::tempdir().unwrap();
    let mut files = minimal_base();
    files.push(("fare_media.txt", FARE_MEDIA));
    write_files(tmp.path(), &files);

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);

    assert!(errors.is_empty());
    assert_eq!(feed.fare_media.len(), 3);
    assert_eq!(feed.fare_media[0].fare_media_type, FareMediaType::None);
    assert_eq!(
        feed.fare_media[1].fare_media_type,
        FareMediaType::PhysicalPaperTicket
    );
    assert_eq!(feed.fare_media[2].fare_media_type, FareMediaType::MobileApp);
}

#[test]
fn fare_products_amount_and_currency_parsed() {
    let tmp = tempfile::tempdir().unwrap();
    let mut files = minimal_base();
    files.push(("fare_products.txt", FARE_PRODUCTS));
    write_files(tmp.path(), &files);

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);

    assert!(errors.is_empty());
    let fp = &feed.fare_products[0];
    assert!((fp.amount - 3.50).abs() < f64::EPSILON);
    assert_eq!(fp.currency.as_ref(), "USD");
    assert_eq!(
        feed.fare_products[1].rider_category_id,
        Some(RiderCategoryId::from("RC1"))
    );
}

#[test]
fn timeframes_times_parsed() {
    let tmp = tempfile::tempdir().unwrap();
    let mut files = minimal_base();
    files.push(("timeframes.txt", TIMEFRAMES));
    write_files(tmp.path(), &files);

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);

    assert!(errors.is_empty());
    let tf = &feed.timeframes[0];
    assert_eq!(tf.start_time.hours(), 6);
    assert_eq!(tf.end_time.hours(), 10);
}

#[test]
fn areas_indexed_by_integrity() {
    let tmp = tempfile::tempdir().unwrap();
    let mut files = minimal_base();
    files.push(("areas.txt", AREAS));
    write_files(tmp.path(), &files);

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);
    assert!(errors.is_empty());
    let idx = IntegrityIndex::build_from_feed(&feed);
    assert!(idx.entity_exists(&EntityRef::Area(AreaId::from("AR1"))));
    assert!(idx.entity_exists(&EntityRef::Area(AreaId::from("AR2"))));
}

#[test]
fn fare_product_reverse_index_finds_dependents() {
    let tmp = tempfile::tempdir().unwrap();
    write_files(tmp.path(), &full_v2_files());

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);
    assert!(errors.is_empty());

    let idx = IntegrityIndex::build_from_feed(&feed);
    let deps = idx.find_dependents(&EntityRef::FareProduct(FareProductId::from("FP1")));
    assert!(deps.iter().any(|(e, r)| matches!(
        (e, r),
        (
            EntityRef::FareLegRule(_),
            RelationType::ProductOfFareLegRule
        )
    )));
    assert!(deps.iter().any(|(e, r)| matches!(
        (e, r),
        (
            EntityRef::FareTransferRule(_),
            RelationType::ProductOfFareTransferRule
        )
    )));
}

#[test]
fn fare_media_type_3_is_cemv() {
    let tmp = tempfile::tempdir().unwrap();
    let mut files = minimal_base();
    files.push((
        "fare_media.txt",
        "fare_media_id,fare_media_name,fare_media_type\nFMC,cEMV card,3\n",
    ));
    write_files(tmp.path(), &files);

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);
    assert!(errors.is_empty());
    assert_eq!(feed.fare_media[0].fare_media_type, FareMediaType::Cemv);
}

#[test]
fn invalid_fare_media_type_emits_error() {
    let tmp = tempfile::tempdir().unwrap();
    let mut files = minimal_base();
    files.push((
        "fare_media.txt",
        "fare_media_id,fare_media_name,fare_media_type\nFMX,Bogus,99\n",
    ));
    write_files(tmp.path(), &files);

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (_feed, errors) = FeedLoader::load(&source);
    assert!(errors.iter().any(
        |e| matches!(e.kind, ParseErrorKind::InvalidEnum) && e.field_name == "fare_media_type"
    ));
}

#[test]
fn create_fare_media_record() {
    let tmp = tempfile::tempdir().unwrap();
    write_files(tmp.path(), &minimal_base());
    let source = FeedLoader::open(tmp.path()).unwrap();
    let (mut feed, _) = FeedLoader::load(&source);

    let assignments = ["fare_media_id=FMNEW".into(), "fare_media_type=3".into()];
    let plan = validate_create(&feed, GtfsTarget::FareMedia, &assignments).unwrap();
    apply_create(&mut feed, plan);

    assert_eq!(feed.fare_media.len(), 1);
    assert_eq!(feed.fare_media[0].fare_media_id.as_ref(), "FMNEW");
    assert_eq!(feed.fare_media[0].fare_media_type, FareMediaType::Cemv);
}

#[test]
fn create_fare_product_record() {
    let tmp = tempfile::tempdir().unwrap();
    write_files(tmp.path(), &minimal_base());
    let source = FeedLoader::open(tmp.path()).unwrap();
    let (mut feed, _) = FeedLoader::load(&source);

    let assignments = [
        "fare_product_id=FPNEW".into(),
        "amount=4.25".into(),
        "currency=USD".into(),
    ];
    let plan = validate_create(&feed, GtfsTarget::FareProducts, &assignments).unwrap();
    apply_create(&mut feed, plan);

    assert_eq!(feed.fare_products.len(), 1);
    assert!((feed.fare_products[0].amount - 4.25).abs() < f64::EPSILON);
}

#[test]
fn create_area_record() {
    let tmp = tempfile::tempdir().unwrap();
    write_files(tmp.path(), &minimal_base());
    let source = FeedLoader::open(tmp.path()).unwrap();
    let (mut feed, _) = FeedLoader::load(&source);

    let assignments = ["area_id=AR42".into(), "area_name=Test".into()];
    let plan = validate_create(&feed, GtfsTarget::Areas, &assignments).unwrap();
    apply_create(&mut feed, plan);

    assert_eq!(feed.areas.len(), 1);
    assert_eq!(feed.areas[0].area_id, AreaId::from("AR42"));
}

#[test]
fn route_network_relations_in_integrity_index() {
    let tmp = tempfile::tempdir().unwrap();
    write_files(tmp.path(), &full_v2_files());

    let source = FeedLoader::open(tmp.path()).unwrap();
    let (feed, errors) = FeedLoader::load(&source);
    assert!(errors.is_empty());

    let idx = IntegrityIndex::build_from_feed(&feed);
    let net_deps = idx.find_dependents(&EntityRef::Network(NetworkId::from("NET1")));
    assert!(
        net_deps
            .iter()
            .any(|(_, r)| *r == RelationType::NetworkOfRouteNetwork)
    );
}
