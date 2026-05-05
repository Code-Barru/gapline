use std::path::Path;
use std::sync::Arc;

use gapline_core::Dataset;
use gapline_core::config::Config;
use gapline_core::crud::{GtfsTarget, parse};
use gapline_core::validation::validate;

// ── Send + Sync ─────────────────────────────────────────────────

#[test]
fn dataset_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Dataset>();
}

// ── canonical ValidationEngine path ───────────────────────────────────

#[test]
fn validation_engine_canonical_path_compiles() {
    use gapline_core::validation::ValidationEngine;
    let _ = std::mem::size_of::<ValidationEngine>();
}

// ── legacy ValidationEngine path still works ──────────────────────────

#[test]
fn validation_engine_legacy_path_compiles() {
    use gapline_core::validation::engine::ValidationEngine;
    let _ = std::mem::size_of::<ValidationEngine>();
}

// ── GtfsTarget accessible from crud root ───────────────────────────────

#[test]
fn gtfs_target_from_crud_root() {
    let _ = GtfsTarget::Stops;
}

// ── Test 5 - Dataset::empty ────────────────────────────────────────────

#[test]
fn dataset_empty_has_no_records() {
    let dataset = Dataset::empty();
    assert!(dataset.feed().agencies.is_empty());
    assert!(dataset.feed().stops.is_empty());
    assert!(dataset.feed().routes.is_empty());
}

// ── Test 1 - Dataset::from_path on valid feed ──────────────────────────

#[test]
fn dataset_from_path_valid_feed() {
    let path = Path::new("../gtfs/minimal.zip");
    if !path.exists() {
        return; // fixture not present in CI
    }
    let (dataset, parse_errors) = Dataset::from_path(path).expect("should open valid feed");
    assert!(
        parse_errors.is_empty(),
        "minimal.zip should have no parse errors"
    );
    assert!(
        !dataset.feed().agencies.is_empty() || !dataset.feed().stops.is_empty(),
        "feed should have loaded records"
    );
}

// ── equivalence: structural + semantic == validation::validate ─────────
//
// Dataset::validate = semantic only. For full-pipeline equivalence, we
// assemble structural + semantic manually and compare with the free function.

fn full_pipeline_report(
    path: &Path,
    config: &Config,
) -> gapline_core::validation::ValidationReport {
    use gapline_core::parser::FeedLoader;
    use gapline_core::validation::ValidationReport;
    let mut source = FeedLoader::open(path).unwrap();
    source.preload().unwrap();
    let structural = Dataset::validate_structural(&source, config);
    if structural.has_errors() {
        return structural;
    }
    let (dataset, parse_errors) = Dataset::from_source(&source);
    let semantic = dataset.validate_semantic(config, &parse_errors);
    let mut all_errors = structural.errors().to_vec();
    all_errors.extend(semantic.errors().to_vec());
    ValidationReport::from(all_errors)
}

#[test]
fn validate_equivalence_valid_feed() {
    let path = Path::new("../gtfs/minimal.zip");
    if !path.exists() {
        return;
    }
    let config = Config::default();
    let report_dataset = full_pipeline_report(path, &config);
    let report_free = validate(path, Arc::new(config)).unwrap();

    assert_eq!(
        report_dataset.error_count(),
        report_free.error_count(),
        "error counts must match"
    );
    assert_eq!(
        report_dataset.warning_count(),
        report_free.warning_count(),
        "warning counts must match"
    );
    assert_eq!(
        report_dataset.info_count(),
        report_free.info_count(),
        "info counts must match"
    );
}

// ── integrity rebuilt after delete ─────────────────────────────────────

#[test]
fn integrity_rebuilt_after_mutation() {
    use gapline_core::integrity::EntityRef;
    use gapline_core::models::AgencyId;
    use gapline_core::models::{Agency, GtfsFeed};

    let agency_id: AgencyId = "A1".into();
    let agency = Agency {
        agency_id: Some(agency_id.clone()),
        agency_name: "Test Agency".into(),
        agency_url: gapline_core::models::Url::from("http://example.com"),
        agency_timezone: gapline_core::models::Timezone::from("UTC"),
        agency_lang: None,
        agency_phone: None,
        agency_fare_url: None,
        agency_email: None,
    };
    let mut feed = GtfsFeed::default();
    feed.agencies.push(agency);

    let mut dataset = Dataset::from_feed(feed);
    assert!(
        dataset
            .integrity()
            .entity_exists(&EntityRef::Agency(agency_id.clone()))
    );

    let query = parse("agency_id=A1").unwrap();
    dataset.delete(GtfsTarget::Agency, &query).unwrap();

    assert!(
        !dataset
            .integrity()
            .entity_exists(&EntityRef::Agency(agency_id)),
        "integrity must not reference deleted agency"
    );
}
