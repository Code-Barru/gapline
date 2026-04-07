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
//!   field, value, severity, rule ID). Built using a fluent builder pattern.
//! - `Severity` -- Classification of findings as `Error`, `Warning`, or `Info`.
//! - `ValidationReport` -- Aggregated summary counts by severity.
//! - `ValidationRule` -- Trait that all validation rules must implement.
//! - `StructuralValidationRule` -- Trait for pre-parsing rules (sections 1 & 2).
//! - `GtfsFeed` -- In-memory GTFS feed data model (see [`crate::models::GtfsFeed`]).

pub mod best_practices;
pub mod csv_formatting;
pub mod engine;
mod error;
pub mod field_definition;
pub mod field_type;
pub mod file_structure;
pub mod foreign_key;
pub mod primary_key;
mod report;
mod rules;
pub mod schedule_time_validation;
mod structural_rule;
pub mod third_party;
pub(crate) mod utils;
mod validate;

pub use error::{Severity, ValidationError};
pub use report::ValidationReport;
pub use rules::ValidationRule;
pub use structural_rule::StructuralValidationRule;
pub use validate::validate;
