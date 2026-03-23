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
//! - `GtfsFeed` -- Placeholder for the in-memory GTFS feed data model.

mod error;
mod report;
mod rules;

pub use error::{Severity, ValidationError};
pub use report::ValidationReport;
pub use rules::{GtfsFeed, ValidationRule};
