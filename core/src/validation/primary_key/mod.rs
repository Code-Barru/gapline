//! Section 6 — Primary Key Uniqueness.
//!
//! Detects duplicate primary keys (simple and composite) across all GTFS
//! files. Each duplicate is an ERROR because it creates ambiguity in
//! referential integrity and CRUD operations.

use std::collections::HashSet;
use std::hash::Hash;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const RULE_ID: &str = "duplicate_key";
const SECTION: &str = "6";

fn err(file: &str, field: &str, value: &str, line: usize) -> ValidationError {
    ValidationError::new(RULE_ID, SECTION, Severity::Error)
        .file(file)
        .line(line)
        .field(field)
        .value(value)
        .message(format!("Duplicate {field}: '{value}'"))
}

fn check_simple<T>(
    records: &[T],
    file: &str,
    field: &str,
    key: impl Fn(&T) -> &str,
    errors: &mut Vec<ValidationError>,
) {
    let mut seen = HashSet::with_capacity(records.len());
    for (i, record) in records.iter().enumerate() {
        let k = key(record);
        if !seen.insert(k) {
            errors.push(err(file, field, k, i + 2));
        }
    }
}

fn check_composite<'a, T: 'a, K: Eq + Hash>(
    records: &'a [T],
    file: &str,
    field: &str,
    key: impl Fn(&'a T) -> K,
    display: impl Fn(&T) -> String,
    errors: &mut Vec<ValidationError>,
) {
    let mut seen = HashSet::with_capacity(records.len());
    for (i, record) in records.iter().enumerate() {
        if !seen.insert(key(record)) {
            errors.push(err(file, field, &display(record), i + 2));
        }
    }
}

/// Checks primary keys for files that are always present in a valid feed.
fn check_required_files(feed: &GtfsFeed, errors: &mut Vec<ValidationError>) {
    {
        let mut seen = HashSet::new();
        for (i, a) in feed.agencies.iter().enumerate() {
            if let Some(id) = &a.agency_id
                && !seen.insert(id.as_ref())
            {
                errors.push(err("agency.txt", "agency_id", id.as_ref(), i + 2));
            }
        }
    }

    check_simple(
        &feed.stops,
        "stops.txt",
        "stop_id",
        |s| s.stop_id.as_ref(),
        errors,
    );
    check_simple(
        &feed.routes,
        "routes.txt",
        "route_id",
        |r| r.route_id.as_ref(),
        errors,
    );
    check_simple(
        &feed.trips,
        "trips.txt",
        "trip_id",
        |t| t.trip_id.as_ref(),
        errors,
    );

    check_composite(
        &feed.stop_times,
        "stop_times.txt",
        "trip_id, stop_sequence",
        |st| (st.trip_id.as_ref(), st.stop_sequence),
        |st| format!("({}, {})", st.trip_id, st.stop_sequence),
        errors,
    );
}

/// Checks simple primary keys for optional GTFS files.
fn check_optional_simple_files(feed: &GtfsFeed, errors: &mut Vec<ValidationError>) {
    if feed.has_file("calendar.txt") {
        check_simple(
            &feed.calendars,
            "calendar.txt",
            "service_id",
            |c| c.service_id.as_ref(),
            errors,
        );
    }
    if feed.has_file("pathways.txt") {
        check_simple(
            &feed.pathways,
            "pathways.txt",
            "pathway_id",
            |p| p.pathway_id.as_ref(),
            errors,
        );
    }
    if feed.has_file("levels.txt") {
        check_simple(
            &feed.levels,
            "levels.txt",
            "level_id",
            |l| l.level_id.as_ref(),
            errors,
        );
    }
    if feed.has_file("fare_attributes.txt") {
        check_simple(
            &feed.fare_attributes,
            "fare_attributes.txt",
            "fare_id",
            |f| f.fare_id.as_ref(),
            errors,
        );
    }
}

/// Checks composite primary keys for optional GTFS files.
fn check_optional_composite_files(feed: &GtfsFeed, errors: &mut Vec<ValidationError>) {
    if feed.has_file("calendar_dates.txt") {
        check_composite(
            &feed.calendar_dates,
            "calendar_dates.txt",
            "service_id, date",
            |cd| (cd.service_id.as_ref(), cd.date),
            |cd| format!("({}, {})", cd.service_id, cd.date),
            errors,
        );
    }

    if feed.has_file("shapes.txt") {
        check_composite(
            &feed.shapes,
            "shapes.txt",
            "shape_id, shape_pt_sequence",
            |s| (s.shape_id.as_ref(), s.shape_pt_sequence),
            |s| format!("({}, {})", s.shape_id, s.shape_pt_sequence),
            errors,
        );
    }

    if feed.has_file("frequencies.txt") {
        check_composite(
            &feed.frequencies,
            "frequencies.txt",
            "trip_id, start_time",
            |f| (f.trip_id.as_ref(), f.start_time),
            |f| format!("({}, {})", f.trip_id, f.start_time),
            errors,
        );
    }

    if feed.has_file("transfers.txt") {
        check_composite(
            &feed.transfers,
            "transfers.txt",
            "from_stop_id, to_stop_id, from_trip_id, to_trip_id",
            |t| {
                (
                    t.from_stop_id.as_ref().map(AsRef::as_ref),
                    t.to_stop_id.as_ref().map(AsRef::as_ref),
                    t.from_trip_id.as_ref().map(AsRef::as_ref),
                    t.to_trip_id.as_ref().map(AsRef::as_ref),
                )
            },
            |t| {
                format!(
                    "({}, {}, {}, {})",
                    t.from_stop_id.as_ref().map_or("", AsRef::as_ref),
                    t.to_stop_id.as_ref().map_or("", AsRef::as_ref),
                    t.from_trip_id.as_ref().map_or("", AsRef::as_ref),
                    t.to_trip_id.as_ref().map_or("", AsRef::as_ref),
                )
            },
            errors,
        );
    }
}

/// Checks special-case primary keys: `feed_info` (max 1 row) and `attributions`.
fn check_special_files(feed: &GtfsFeed, errors: &mut Vec<ValidationError>) {
    if feed.has_file("feed_info.txt") && feed.feed_info_line_count > 1 {
        errors.push(
            ValidationError::new(RULE_ID, SECTION, Severity::Error)
                .file("feed_info.txt")
                .line(3)
                .field("feed_publisher_name")
                .message("feed_info.txt must contain exactly one data row"),
        );
    }

    if feed.has_file("attributions.txt")
        && feed.attributions.iter().any(|a| a.attribution_id.is_some())
    {
        let mut seen = HashSet::with_capacity(feed.attributions.len());
        for (i, a) in feed.attributions.iter().enumerate() {
            if let Some(id) = a.attribution_id.as_deref()
                && !seen.insert(id)
            {
                errors.push(err("attributions.txt", "attribution_id", id, i + 2));
            }
        }
    }
}

pub struct PrimaryKeyUniquenessRule;

impl ValidationRule for PrimaryKeyUniquenessRule {
    fn rule_id(&self) -> &'static str {
        RULE_ID
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        check_required_files(feed, &mut errors);
        check_optional_simple_files(feed, &mut errors);
        check_optional_composite_files(feed, &mut errors);
        check_special_files(feed, &mut errors);
        errors
    }
}

/// Registers all section 6 (Primary Key Uniqueness) rules with the engine.
pub fn register_rules(engine: &mut crate::validation::engine::ValidationEngine) {
    engine.register_rule(Box::new(PrimaryKeyUniquenessRule));
}
