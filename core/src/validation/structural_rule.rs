//! Shared trait for pre-parsing validation rules.
//!
//! Both `file_structure` (section 1) and `csv_formatting` (section 2) rules operate
//! on a [`FeedSource`] before any CSV data is loaded into memory.

use crate::parser::FeedSource;
use crate::validation::{Severity, ValidationError};

/// Trait for validation rules that operate on the raw feed structure.
///
/// Unlike [`ValidationRule`](super::ValidationRule) which takes a parsed
/// `GtfsFeed`, structural rules only need access to file names and raw CSV
/// content via [`FeedSource`]. This separation enforces the architectural gate:
/// structural validation runs first, and only if it passes does CSV parsing and
/// data loading begin.
///
/// Implementations must be [`Send`] + [`Sync`] for parallel execution with
/// [rayon](https://docs.rs/rayon).
///
/// # Implementing a Rule
///
/// ```no_run
/// use headway_core::parser::FeedSource;
/// use headway_core::validation::StructuralValidationRule;
/// use headway_core::validation::{ValidationError, Severity};
///
/// pub struct MissingRequiredFileRule;
///
/// impl StructuralValidationRule for MissingRequiredFileRule {
///     fn rule_id(&self) -> &'static str { "missing_required_file" }
///     fn section(&self) -> &'static str { "1" }
///     fn severity(&self) -> Severity { Severity::Error }
///
///     fn validate(&self, source: &FeedSource) -> Vec<ValidationError> {
///         // Check file_names() for required files...
///         vec![]
///     }
/// }
/// ```
pub trait StructuralValidationRule: Send + Sync {
    /// Unique identifier for this rule (e.g. `"missing_required_file"`).
    ///
    /// Must match the error code from the GTFS specification reference document.
    fn rule_id(&self) -> &'static str;

    /// GTFS specification section that defines this rule (e.g. `"1"`).
    fn section(&self) -> &'static str;

    /// Default severity level for findings produced by this rule.
    fn severity(&self) -> Severity;

    /// Runs the structural validation check against the raw feed source.
    ///
    /// Returns an empty `Vec` if no issues are found, or one or more
    /// [`ValidationError`]s with full diagnostic context otherwise.
    fn validate(&self, source: &FeedSource) -> Vec<ValidationError>;
}
