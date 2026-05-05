//! Tests for section 10 - Fares v2 Validation.
//!
//! Covers field-definition (conditional) rules and foreign-key rules across
//! the 11 Fares v2 files. Records are constructed directly (no CSV roundtrip)
//! to keep each test focused on a single validation concern.

use chrono::NaiveDate;
use gapline_core::models::*;
use gapline_core::validation::fares_v2_semantic::rules::{
    CircularTransferRule, EmptyAreaRule, InvalidTransferCountRule, NegativeAmountRule,
    TimeframeOverlapRule, UnusedFareProductRule, ZeroAmountRule, ZeroDurationLimitRule,
};
use gapline_core::validation::field_definition::fares_v2::FaresV2FieldDefinitionRule;
use gapline_core::validation::foreign_key::fare_leg_join_rules::{
    FareLegJoinRulesFromNetworkFkRule, FareLegJoinRulesFromStopFkRule,
    FareLegJoinRulesToNetworkFkRule, FareLegJoinRulesToStopFkRule,
};
use gapline_core::validation::foreign_key::fare_leg_rules_areas::{
    FareLegRulesFromAreaFkRule, FareLegRulesToAreaFkRule,
};
use gapline_core::validation::foreign_key::fare_leg_rules_network::FareLegRulesNetworkFkRule;
use gapline_core::validation::foreign_key::fare_leg_rules_product::FareLegRulesProductFkRule;
use gapline_core::validation::foreign_key::fare_leg_rules_timeframes::{
    FareLegRulesFromTimeframeFkRule, FareLegRulesToTimeframeFkRule,
};
use gapline_core::validation::foreign_key::fare_products_media::FareProductsMediaFkRule;
use gapline_core::validation::foreign_key::fare_products_rider::FareProductsRiderFkRule;
use gapline_core::validation::foreign_key::fare_transfer_rules_legs::{
    FareTransferRulesFromLegFkRule, FareTransferRulesToLegFkRule,
};
use gapline_core::validation::foreign_key::fare_transfer_rules_product::FareTransferRulesProductFkRule;
use gapline_core::validation::foreign_key::route_networks::{
    RouteNetworksNetworkFkRule, RouteNetworksRouteFkRule,
};
use gapline_core::validation::foreign_key::stop_areas::{StopAreasAreaFkRule, StopAreasStopFkRule};
use gapline_core::validation::foreign_key::timeframes_service::TimeframesServiceFkRule;
use gapline_core::validation::{Severity, ValidationError, ValidationRule};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn date(y: i32, m: u32, d: u32) -> GtfsDate {
    GtfsDate(NaiveDate::from_ymd_opt(y, m, d).unwrap())
}

fn calendar(service_id: &str) -> Calendar {
    Calendar {
        service_id: ServiceId::from(service_id),
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        start_date: date(2024, 1, 1),
        end_date: date(2024, 12, 31),
    }
}

fn stop(id: &str) -> Stop {
    Stop {
        stop_id: StopId::from(id),
        stop_code: None,
        stop_name: Some("S".to_string()),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(45.0)),
        stop_lon: Some(Longitude(-73.0)),
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

fn route(id: &str) -> Route {
    Route {
        route_id: RouteId::from(id),
        agency_id: None,
        route_short_name: Some("1".to_string()),
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

fn fare_media(id: &str, ty: FareMediaType) -> FareMedia {
    FareMedia {
        fare_media_id: FareMediaId::from(id),
        fare_media_name: None,
        fare_media_type: ty,
    }
}

fn fare_product(id: &str, media: Option<&str>, rider: Option<&str>) -> FareProduct {
    FareProduct {
        fare_product_id: FareProductId::from(id),
        fare_product_name: None,
        fare_media_id: media.map(FareMediaId::from),
        amount: 3.50,
        currency: CurrencyCode::from("USD".to_string()),
        rider_category_id: rider.map(RiderCategoryId::from),
    }
}

fn fare_transfer_rule(
    from_leg: Option<&str>,
    to_leg: Option<&str>,
    duration_limit: Option<u32>,
    duration_type: Option<DurationLimitType>,
    transfer_type: FareTransferType,
    product: Option<&str>,
) -> FareTransferRule {
    FareTransferRule {
        from_leg_group_id: from_leg.map(LegGroupId::from),
        to_leg_group_id: to_leg.map(LegGroupId::from),
        transfer_count: None,
        duration_limit,
        duration_limit_type: duration_type,
        fare_transfer_type: transfer_type,
        fare_product_id: product.map(FareProductId::from),
    }
}

fn rider_category(id: &str, min: Option<u32>, max: Option<u32>) -> RiderCategory {
    RiderCategory {
        rider_category_id: RiderCategoryId::from(id),
        rider_category_name: "Cat".to_string(),
        min_age: min,
        max_age: max,
        eligibility_url: None,
    }
}

fn timeframe(id: &str, start: GtfsTime, end: GtfsTime, service: &str) -> Timeframe {
    Timeframe {
        timeframe_group_id: TimeframeId::from(id),
        start_time: start,
        end_time: end,
        service_id: ServiceId::from(service),
    }
}

fn area(id: &str) -> Area {
    Area {
        area_id: AreaId::from(id),
        area_name: None,
    }
}

fn stop_area(area_id: &str, stop_id: &str) -> StopArea {
    StopArea {
        area_id: AreaId::from(area_id),
        stop_id: StopId::from(stop_id),
    }
}

fn network(id: &str) -> Network {
    Network {
        network_id: NetworkId::from(id),
        network_name: None,
    }
}

fn route_network(network_id: &str, route_id: &str) -> RouteNetwork {
    RouteNetwork {
        network_id: NetworkId::from(network_id),
        route_id: RouteId::from(route_id),
    }
}

fn fare_leg_join_rule(
    from_net: &str,
    to_net: &str,
    from_stop: Option<&str>,
    to_stop: Option<&str>,
) -> FareLegJoinRule {
    FareLegJoinRule {
        from_network_id: NetworkId::from(from_net),
        to_network_id: NetworkId::from(to_net),
        from_stop_id: from_stop.map(StopId::from),
        to_stop_id: to_stop.map(StopId::from),
    }
}

/// A fully-valid Fares v2 feed: every FK resolves, every conditional rule passes.
fn valid_v2_feed() -> GtfsFeed {
    let mut feed = GtfsFeed::default();
    feed.stops.push(stop("S1"));
    feed.stops.push(stop("S2"));
    feed.routes.push(route("R1"));
    feed.calendars.push(calendar("SVC1"));

    feed.fare_media.push(fare_media("FM1", FareMediaType::None));
    feed.rider_categories
        .push(rider_category("RC1", Some(18), Some(64)));
    feed.fare_products
        .push(fare_product("FP1", Some("FM1"), Some("RC1")));
    feed.areas.push(area("AR1"));
    feed.areas.push(area("AR2"));
    feed.timeframes.push(timeframe(
        "TF1",
        GtfsTime::from_hms(6, 0, 0),
        GtfsTime::from_hms(10, 0, 0),
        "SVC1",
    ));
    feed.networks.push(network("NET1"));
    feed.fare_leg_rules.push(FareLegRule {
        leg_group_id: Some(LegGroupId::from("LG1")),
        network_id: Some(NetworkId::from("NET1")),
        from_area_id: Some(AreaId::from("AR1")),
        to_area_id: Some(AreaId::from("AR2")),
        from_timeframe_group_id: Some(TimeframeId::from("TF1")),
        to_timeframe_group_id: Some(TimeframeId::from("TF1")),
        fare_product_id: FareProductId::from("FP1"),
        rule_priority: None,
    });
    feed.fare_transfer_rules.push(fare_transfer_rule(
        Some("LG1"),
        Some("LG1"),
        Some(3600),
        Some(DurationLimitType::DepartureToArrival),
        FareTransferType::Sum,
        Some("FP1"),
    ));
    feed.stop_areas.push(stop_area("AR1", "S1"));
    feed.stop_areas.push(stop_area("AR2", "S2"));
    feed.route_networks.push(route_network("NET1", "R1"));
    feed.fare_leg_join_rules
        .push(fare_leg_join_rule("NET1", "NET1", Some("S1"), Some("S2")));
    feed
}

fn all_v2_rules() -> Vec<Box<dyn ValidationRule>> {
    vec![
        Box::new(FaresV2FieldDefinitionRule),
        Box::new(FareProductsMediaFkRule),
        Box::new(FareProductsRiderFkRule),
        Box::new(FareLegRulesProductFkRule),
        Box::new(FareLegRulesFromAreaFkRule),
        Box::new(FareLegRulesToAreaFkRule),
        Box::new(FareLegRulesFromTimeframeFkRule),
        Box::new(FareLegRulesToTimeframeFkRule),
        Box::new(FareLegRulesNetworkFkRule),
        Box::new(FareTransferRulesFromLegFkRule),
        Box::new(FareTransferRulesToLegFkRule),
        Box::new(FareTransferRulesProductFkRule),
        Box::new(StopAreasAreaFkRule),
        Box::new(StopAreasStopFkRule),
        Box::new(TimeframesServiceFkRule),
        Box::new(RouteNetworksNetworkFkRule),
        Box::new(RouteNetworksRouteFkRule),
        Box::new(FareLegJoinRulesFromNetworkFkRule),
        Box::new(FareLegJoinRulesToNetworkFkRule),
        Box::new(FareLegJoinRulesFromStopFkRule),
        Box::new(FareLegJoinRulesToStopFkRule),
    ]
}

fn run_all(feed: &GtfsFeed) -> Vec<ValidationError> {
    all_v2_rules()
        .iter()
        .flat_map(|r| r.validate(feed))
        .collect()
}

fn count_errors(errors: &[ValidationError], severity: Severity) -> usize {
    errors.iter().filter(|e| e.severity == severity).count()
}

// ---------------------------------------------------------------------------
// Feed-level invariants
// ---------------------------------------------------------------------------

#[test]
fn valid_v2_feed_emits_no_errors() {
    let feed = valid_v2_feed();
    let errors = run_all(&feed);
    assert!(errors.is_empty(), "expected 0 errors, got: {errors:?}");
}

#[test]
fn empty_feed_emits_no_section_10_errors() {
    let feed = GtfsFeed::default();
    let errors = run_all(&feed);
    assert!(errors.is_empty(), "expected 0 errors, got: {errors:?}");
}

#[test]
fn fares_v1_only_feed_emits_no_section_10_errors() {
    // A feed with v1 fare files but no v2 collections must not produce
    // any section-10 cross-references.
    let mut feed = GtfsFeed::default();
    feed.stops.push(stop("S1"));
    feed.routes.push(route("R1"));
    feed.calendars.push(calendar("SVC1"));
    feed.fare_attributes.push(FareAttribute {
        fare_id: FareId::from("F1"),
        price: 2.50,
        currency_type: CurrencyCode::from("USD".to_string()),
        payment_method: 1,
        transfers: Some(0),
        agency_id: None,
        transfer_duration: None,
    });

    let errors = run_all(&feed);
    assert!(errors.is_empty(), "expected 0 errors, got: {errors:?}");
}

// ---------------------------------------------------------------------------
// Foreign-key orphans (CA5–CA10 + extended)
// ---------------------------------------------------------------------------

#[test]
fn fare_product_with_orphan_media_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.fare_products[0].fare_media_id = Some(FareMediaId::from("GHOST"));

    let errors = FareProductsMediaFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("fare_media_id"));
    assert_eq!(errors[0].file_name.as_deref(), Some("fare_products.txt"));
}

#[test]
fn fare_product_with_orphan_rider_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.fare_products[0].rider_category_id = Some(RiderCategoryId::from("GHOST"));

    let errors = FareProductsRiderFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("rider_category_id"));
}

#[test]
fn fare_leg_rule_with_orphan_product_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.fare_leg_rules[0].fare_product_id = FareProductId::from("GHOST");

    let errors = FareLegRulesProductFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("fare_product_id"));
}

#[test]
fn fare_leg_rule_with_orphan_from_area_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.fare_leg_rules[0].from_area_id = Some(AreaId::from("GHOST"));

    let errors = FareLegRulesFromAreaFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("from_area_id"));
}

#[test]
fn fare_leg_rule_with_orphan_to_area_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.fare_leg_rules[0].to_area_id = Some(AreaId::from("GHOST"));

    let errors = FareLegRulesToAreaFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("to_area_id"));
}

#[test]
fn fare_leg_rule_with_orphan_from_timeframe_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.fare_leg_rules[0].from_timeframe_group_id = Some(TimeframeId::from("GHOST"));

    let errors = FareLegRulesFromTimeframeFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].field_name.as_deref(),
        Some("from_timeframe_group_id")
    );
}

#[test]
fn fare_leg_rule_with_orphan_to_timeframe_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.fare_leg_rules[0].to_timeframe_group_id = Some(TimeframeId::from("GHOST"));

    let errors = FareLegRulesToTimeframeFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

#[test]
fn fare_leg_rule_with_orphan_network_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.fare_leg_rules[0].network_id = Some(NetworkId::from("GHOST"));

    let errors = FareLegRulesNetworkFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("network_id"));
}

#[test]
fn fare_transfer_rule_with_orphan_from_leg_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.fare_transfer_rules[0].from_leg_group_id = Some(LegGroupId::from("GHOST"));

    let errors = FareTransferRulesFromLegFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("from_leg_group_id"));
}

#[test]
fn fare_transfer_rule_with_orphan_to_leg_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.fare_transfer_rules[0].to_leg_group_id = Some(LegGroupId::from("GHOST"));

    let errors = FareTransferRulesToLegFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

#[test]
fn fare_transfer_rule_with_orphan_product_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.fare_transfer_rules[0].fare_product_id = Some(FareProductId::from("GHOST"));

    let errors = FareTransferRulesProductFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

#[test]
fn stop_area_with_orphan_area_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.stop_areas[0].area_id = AreaId::from("GHOST");

    let errors = StopAreasAreaFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("area_id"));
}

#[test]
fn stop_area_with_orphan_stop_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.stop_areas[0].stop_id = StopId::from("GHOST");

    let errors = StopAreasStopFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("stop_id"));
}

#[test]
fn timeframe_with_orphan_service_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.timeframes[0].service_id = ServiceId::from("GHOST");

    let errors = TimeframesServiceFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("service_id"));
}

#[test]
fn timeframe_service_in_calendar_dates_only_is_valid() {
    let mut feed = valid_v2_feed();
    feed.calendars.clear();
    feed.calendar_dates.push(CalendarDate {
        service_id: ServiceId::from("SVC1"),
        date: date(2024, 6, 1),
        exception_type: ExceptionType::Added,
    });

    let errors = TimeframesServiceFkRule.validate(&feed);
    assert!(errors.is_empty(), "expected 0 errors, got: {errors:?}");
}

#[test]
fn route_network_with_orphan_network_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.route_networks[0].network_id = NetworkId::from("GHOST");

    let errors = RouteNetworksNetworkFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

#[test]
fn route_network_with_orphan_route_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.route_networks[0].route_id = RouteId::from("GHOST");

    let errors = RouteNetworksRouteFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

#[test]
fn fare_leg_join_rule_with_orphan_from_network_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.fare_leg_join_rules[0].from_network_id = NetworkId::from("GHOST");

    let errors = FareLegJoinRulesFromNetworkFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

#[test]
fn fare_leg_join_rule_with_orphan_to_network_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.fare_leg_join_rules[0].to_network_id = NetworkId::from("GHOST");

    let errors = FareLegJoinRulesToNetworkFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

#[test]
fn fare_leg_join_rule_with_orphan_from_stop_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.fare_leg_join_rules[0].from_stop_id = Some(StopId::from("GHOST"));

    let errors = FareLegJoinRulesFromStopFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

#[test]
fn fare_leg_join_rule_with_orphan_to_stop_emits_fk_error() {
    let mut feed = valid_v2_feed();
    feed.fare_leg_join_rules[0].to_stop_id = Some(StopId::from("GHOST"));

    let errors = FareLegJoinRulesToStopFkRule.validate(&feed);
    assert_eq!(errors.len(), 1);
}

// ---------------------------------------------------------------------------
// Field-definition conditional rules
// ---------------------------------------------------------------------------

#[test]
fn duration_limit_without_type_emits_error() {
    let mut feed = GtfsFeed::default();
    feed.fare_transfer_rules.push(fare_transfer_rule(
        None,
        None,
        Some(3600),
        None,
        FareTransferType::FromLeg,
        None,
    ));

    let errors = FaresV2FieldDefinitionRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("duration_limit_type"));
}

#[test]
fn duration_type_without_limit_emits_error() {
    let mut feed = GtfsFeed::default();
    feed.fare_transfer_rules.push(fare_transfer_rule(
        None,
        None,
        None,
        Some(DurationLimitType::DepartureToArrival),
        FareTransferType::FromLeg,
        None,
    ));

    let errors = FaresV2FieldDefinitionRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("duration_limit"));
}

#[test]
fn min_age_greater_than_max_age_emits_error() {
    let mut feed = GtfsFeed::default();
    feed.rider_categories
        .push(rider_category("RC1", Some(70), Some(50)));

    let errors = FaresV2FieldDefinitionRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("min_age"));
}

#[test]
fn min_age_only_passes() {
    let mut feed = GtfsFeed::default();
    feed.rider_categories
        .push(rider_category("RC1", Some(65), None));

    let errors = FaresV2FieldDefinitionRule.validate(&feed);
    assert!(errors.is_empty());
}

#[test]
fn timeframe_with_start_after_end_emits_error() {
    let mut feed = GtfsFeed::default();
    feed.timeframes.push(timeframe(
        "TF1",
        GtfsTime::from_hms(10, 0, 0),
        GtfsTime::from_hms(6, 0, 0),
        "SVC1",
    ));

    let errors = FaresV2FieldDefinitionRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Error), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("start_time"));
}

#[test]
fn fare_leg_rule_without_any_criterion_emits_warning() {
    let mut feed = GtfsFeed::default();
    feed.fare_leg_rules.push(FareLegRule {
        leg_group_id: None,
        network_id: None,
        from_area_id: None,
        to_area_id: None,
        from_timeframe_group_id: None,
        to_timeframe_group_id: None,
        fare_product_id: FareProductId::from("FP1"),
        rule_priority: None,
    });

    let errors = FaresV2FieldDefinitionRule.validate(&feed);
    assert_eq!(count_errors(&errors, Severity::Warning), 1);
}

// ---------------------------------------------------------------------------
// Multi-file aggregation
// ---------------------------------------------------------------------------

#[test]
fn errors_from_multiple_files_all_reported() {
    let mut feed = valid_v2_feed();
    // Inject orphans in three separate files.
    feed.fare_products[0].fare_media_id = Some(FareMediaId::from("GHOST_FM"));
    feed.fare_leg_rules[0].fare_product_id = FareProductId::from("GHOST_FP");
    feed.stop_areas[0].area_id = AreaId::from("GHOST_AR");

    let errors = run_all(&feed);
    let files: std::collections::HashSet<_> = errors
        .iter()
        .filter_map(|e| e.file_name.as_deref())
        .collect();
    assert!(files.contains("fare_products.txt"));
    assert!(files.contains("fare_leg_rules.txt"));
    assert!(files.contains("stop_areas.txt"));
    assert!(errors.iter().all(|e| e.severity == Severity::Error));
}

// ---------------------------------------------------------------------------
// Semantic rules
// ---------------------------------------------------------------------------

fn loaded_v2_feed() -> GtfsFeed {
    let mut feed = valid_v2_feed();
    feed.loaded_files.insert("fare_products.txt".to_string());
    feed
}

fn run_semantic(feed: &GtfsFeed) -> Vec<ValidationError> {
    let rules: Vec<Box<dyn ValidationRule>> = vec![
        Box::new(NegativeAmountRule),
        Box::new(ZeroAmountRule),
        Box::new(TimeframeOverlapRule),
        Box::new(InvalidTransferCountRule),
        Box::new(ZeroDurationLimitRule),
        Box::new(CircularTransferRule),
        Box::new(UnusedFareProductRule),
        Box::new(EmptyAreaRule),
    ];
    rules.iter().flat_map(|r| r.validate(feed)).collect()
}

#[test]
fn semantic_valid_feed_clean() {
    let errors = run_semantic(&loaded_v2_feed());
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn semantic_skipped_when_no_v2() {
    assert!(run_semantic(&GtfsFeed::default()).is_empty());
}

#[test]
fn semantic_negative_amount() {
    let mut feed = loaded_v2_feed();
    feed.fare_products[0].amount = -1.50;
    let errors = NegativeAmountRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "fares_negative_amount");
    assert_eq!(errors[0].severity, Severity::Error);
}

#[test]
fn semantic_zero_amount() {
    let mut feed = loaded_v2_feed();
    feed.fare_products[0].amount = 0.0;
    let errors = ZeroAmountRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].severity, Severity::Warning);
}

#[test]
fn semantic_timeframe_overlap() {
    let mut feed = loaded_v2_feed();
    feed.timeframes.clear();
    feed.timeframes.push(timeframe(
        "TF1",
        GtfsTime::from_hms(6, 0, 0),
        GtfsTime::from_hms(10, 0, 0),
        "SVC1",
    ));
    feed.timeframes.push(timeframe(
        "TF1",
        GtfsTime::from_hms(9, 0, 0),
        GtfsTime::from_hms(12, 0, 0),
        "SVC1",
    ));
    let errors = TimeframeOverlapRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].severity, Severity::Warning);
}

#[test]
fn semantic_timeframe_adjacent_ok() {
    let mut feed = loaded_v2_feed();
    feed.timeframes.clear();
    feed.timeframes.push(timeframe(
        "TF1",
        GtfsTime::from_hms(6, 0, 0),
        GtfsTime::from_hms(10, 0, 0),
        "SVC1",
    ));
    feed.timeframes.push(timeframe(
        "TF1",
        GtfsTime::from_hms(10, 0, 0),
        GtfsTime::from_hms(14, 0, 0),
        "SVC1",
    ));
    assert!(TimeframeOverlapRule.validate(&feed).is_empty());
}

#[test]
fn semantic_invalid_transfer_count() {
    let mut feed = loaded_v2_feed();
    for n in [0, -1] {
        feed.fare_transfer_rules[0].transfer_count = Some(n);
        let errors = InvalidTransferCountRule.validate(&feed);
        assert_eq!(errors.len(), 1, "transfer_count={n}");
        assert_eq!(errors[0].severity, Severity::Error);
    }
}

#[test]
fn semantic_zero_duration_limit() {
    let mut feed = loaded_v2_feed();
    feed.fare_transfer_rules[0].duration_limit = Some(0);
    let errors = ZeroDurationLimitRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].severity, Severity::Error);
}

fn push_transfer(feed: &mut GtfsFeed, from: &str, to: &str) {
    feed.fare_transfer_rules.push(fare_transfer_rule(
        Some(from),
        Some(to),
        None,
        None,
        FareTransferType::Sum,
        Some("FP1"),
    ));
}

#[test]
fn semantic_circular_transfer_two_node() {
    let mut feed = loaded_v2_feed();
    feed.fare_transfer_rules.clear();
    push_transfer(&mut feed, "A", "B");
    push_transfer(&mut feed, "B", "A");
    assert_eq!(CircularTransferRule.validate(&feed).len(), 2);
}

#[test]
fn semantic_circular_transfer_three_node() {
    let mut feed = loaded_v2_feed();
    feed.fare_transfer_rules.clear();
    push_transfer(&mut feed, "A", "B");
    push_transfer(&mut feed, "B", "C");
    push_transfer(&mut feed, "C", "A");
    assert_eq!(CircularTransferRule.validate(&feed).len(), 3);
}

#[test]
fn semantic_circular_transfer_dag_ok() {
    let mut feed = loaded_v2_feed();
    feed.fare_transfer_rules.clear();
    push_transfer(&mut feed, "A", "B");
    push_transfer(&mut feed, "B", "C");
    assert!(CircularTransferRule.validate(&feed).is_empty());
}

#[test]
fn semantic_unused_product() {
    let mut feed = loaded_v2_feed();
    feed.fare_products.push(fare_product("ORPHAN", None, None));
    let errors = UnusedFareProductRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].value.as_deref(), Some("ORPHAN"));
}

#[test]
fn semantic_product_used_via_transfer_rule_only() {
    let mut feed = loaded_v2_feed();
    feed.fare_products.push(fare_product("FP2", None, None));
    feed.fare_transfer_rules.push(fare_transfer_rule(
        Some("LG1"),
        Some("LG1"),
        None,
        None,
        FareTransferType::Sum,
        Some("FP2"),
    ));
    let errors = UnusedFareProductRule.validate(&feed);
    assert!(errors.iter().all(|e| e.value.as_deref() != Some("FP2")));
}

#[test]
fn semantic_empty_area() {
    let mut feed = loaded_v2_feed();
    feed.areas.push(area("EMPTY"));
    let errors = EmptyAreaRule.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].value.as_deref(), Some("EMPTY"));
}
