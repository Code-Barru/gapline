//! GTFS feed validation subsystem.
//!
//! This module provides the building blocks for validating GTFS feeds against the
//! full GTFS Schedule specification (sections 1-8 and 13). The design is
//! trait-based: each validation rule implements the `ValidationRule` trait, enabling
//! modular rule addition and parallel execution via [rayon](https://docs.rs/rayon).
//!
//! ## Key Types
//!
//! - `ValidationError` -- A structured error with full context (file, line,
//! field, value, severity, rule ID). Built using a fluent builder pattern.
//! - `Severity` -- Classification of findings as `Error`, `Warning`, or `Info`.
//! - `ValidationReport` -- Aggregated summary counts by severity.
//! - `ValidationRule` -- Trait that all validation rules must implement.
//! - `StructuralValidationRule` -- Trait for pre-parsing rules (sections 1 & 2).
//! - `GtfsFeed` -- In-memory GTFS feed data model (see [`crate::models::GtfsFeed`]).

pub mod best_practices;
pub mod csv_formatting;
pub mod engine;
mod error;
pub mod fares_v2_semantic;
pub mod field_definition;
pub mod field_type;
pub mod file_structure;
pub mod flex_semantic;
pub mod foreign_key;
pub mod locations_geojson_semantic;
pub mod primary_key;
pub mod realtime_semantic;
mod report;
pub mod rt_rules;
mod rules;
pub mod schedule_time_validation;
mod structural_rule;
pub mod third_party;
pub(crate) mod utils;
mod validate;

pub use engine::ValidationEngine;
pub use error::{Severity, ValidationError};
pub use report::ValidationReport;
pub use rt_rules::{RtValidationContext, RtValidationRule, ScheduleIndex};
pub use rules::ValidationRule;
pub use structural_rule::StructuralValidationRule;
pub use validate::validate;

/// Returns every rule ID registered in a default-configured validation
/// engine, sorted and deduplicated. Intended for CLI completion and
/// discoverability (`--disable-rule <TAB>`).
///
/// Built once on first call via [`std::sync::LazyLock`]: the underlying
/// engine instantiation allocates a few dozen `Box<dyn Rule>` but is cheap
/// compared to actually running validation, and the result is cached for
/// the lifetime of the process.
#[must_use]
pub fn all_rule_ids() -> &'static [&'static str] {
    use std::sync::{Arc, LazyLock};

    static IDS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
        let engine = engine::ValidationEngine::new(Arc::new(crate::config::Config::default()));
        let mut ids: Vec<&'static str> = engine
            .pre_rules()
            .iter()
            .map(|r| r.rule_id())
            .chain(engine.post_rules().iter().map(|r| r.rule_id()))
            .chain(engine.rt_rules().iter().map(|r| r.rule_id()))
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    });
    &IDS
}
