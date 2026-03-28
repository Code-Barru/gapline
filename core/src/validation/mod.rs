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
//! - `GtfsFeed` -- Placeholder for the in-memory GTFS feed data model.

pub mod csv_formating;
pub mod engine;
mod error;
pub mod file_structure;
mod report;
mod rules;
mod structural_rule;
mod validate;

pub use error::{Severity, ValidationError};
pub use report::ValidationReport;
pub use rules::{GtfsFeed, ValidationRule};
pub use structural_rule::StructuralValidationRule;
pub use validate::validate;
