use crate::validation::{Severity, ValidationError};

/// Trait that all GTFS validation rules must implement.
///
/// Each struct implementing `ValidationRule` represents a single, self-contained
/// validation check derived from the GTFS specification reference document. Rules
/// are required to be [`Send`] + [`Sync`] so the validation engine can execute
/// them in parallel via [rayon](https://docs.rs/rayon).
///
/// # Implementing a Rule
///
/// ```ignore
/// use headway::validation::{ValidationRule, ValidationError, Severity, GtfsFeed};
///
/// pub struct MissingRequiredFileRule;
///
/// impl ValidationRule for MissingRequiredFileRule {
///     fn rule_id(&self) -> &'static str { "missing_required_file" }
///     fn section(&self) -> &'static str { "1" }
///     fn severity(&self) -> Severity { Severity::Error }
///
///     fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
///         // Check for missing required files and return errors
///         vec![]
///     }
/// }
/// ```
pub trait ValidationRule: Send + Sync {
    /// Unique identifier for this rule (e.g. `"missing_required_file"`).
    ///
    /// Must match the error code from the GTFS specification reference document.
    fn rule_id(&self) -> &'static str;

    /// GTFS specification section that defines this rule (e.g. `"1"`, `"7.3"`).
    fn section(&self) -> &'static str;

    /// Default severity level for findings produced by this rule.
    fn severity(&self) -> Severity;

    /// Runs the validation check against the given feed.
    ///
    /// Returns an empty `Vec` if no issues are found, or one or more
    /// [`ValidationError`]s with full diagnostic context otherwise.
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError>;
}

/// Placeholder for the in-memory GTFS feed data model.
///
/// This struct will eventually hold all parsed GTFS records (`agencies`, `routes`,
/// `trips`, `stops`, `stop_times`, etc.) with type-safe ID wrappers. It is the primary
/// input to all validation rules.
///
/// **Status:** temporary empty struct -- will be replaced when the parser and
/// data model modules are implemented.
pub struct GtfsFeed {}
