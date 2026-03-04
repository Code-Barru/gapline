use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// A structured validation finding with full diagnostic context.
///
/// Each `ValidationError` represents a single issue found during GTFS feed
/// validation. It carries enough context for the user to locate and understand
/// the problem: which rule triggered it, where in the feed it occurred, and what
/// value was invalid.
///
/// Instances are constructed with [`ValidationError::new`] and enriched
/// through a fluent builder API. All optional context fields default to `None`.
///
/// # Examples
///
/// ```
/// use headway::validation::{ValidationError, Severity};
///
/// // Structural error (no file/line context needed)
/// let structural = ValidationError::new("missing_required_file", "1", Severity::Error)
///     .message("Required file agency.txt is missing")
///     .file("agency.txt");
///
/// // Field-level error with full context
/// let field_err = ValidationError::new("invalid_date", "3", Severity::Error)
///     .message("Invalid date format: 2026-13-01")
///     .file("calendar.txt")
///     .line(42)
///     .field("start_date")
///     .value("2026-13-01");
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub(crate) rule_id: String,
    pub(crate) section: String,
    pub(crate) severity: Severity,
    pub(crate) message: String,
    pub(crate) file_name: Option<String>,
    pub(crate) line_number: Option<usize>,
    pub(crate) field_name: Option<String>,
    pub(crate) value: Option<String>,
}

impl ValidationError {
    /// Creates a new `ValidationError` with the required identification fields.
    ///
    /// All optional context fields (`file_name`, `line_number`, `field_name`,
    /// `value`) default to `None` and `message` defaults to an empty string.
    /// Use the builder methods to add context.
    ///
    /// # Arguments
    ///
    /// * `rule_id` -- Unique rule identifier matching the GTFS specification
    ///   reference document.
    /// * `section` -- Specification section number (e.g. `"1"`, `"7.3"`).
    /// * `severity` -- How severe this finding is.
    pub fn new(rule_id: impl Into<String>, section: impl Into<String>, severity: Severity) -> Self {
        Self {
            rule_id: rule_id.into(),
            section: section.into(),
            severity,
            message: String::new(),
            file_name: None,
            line_number: None,
            field_name: None,
            value: None,
        }
    }

    /// Sets the human-readable error message.
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Sets the GTFS file name where the issue was found (e.g. `"stops.txt"`).
    #[must_use]
    pub fn file(mut self, file: impl Into<String>) -> Self {
        self.file_name = Some(file.into());
        self
    }

    /// Sets the 1-indexed line number within the file.
    #[must_use]
    pub fn line(mut self, line: usize) -> Self {
        self.line_number = Some(line);
        self
    }

    /// Sets the CSV field/column name that contains the invalid value.
    #[must_use]
    pub fn field(mut self, field: impl Into<String>) -> Self {
        self.field_name = Some(field.into());
        self
    }

    /// Sets the actual value that triggered the error.
    #[must_use]
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }
}

/// Classification of a validation finding's severity.
///
/// Variants are ordered from least to most severe: `Info < Warning < Error`.
/// This ordering is used by [`PartialOrd`] and [`Ord`], allowing findings to be
/// sorted or filtered by severity.
///
/// When serialized to JSON via serde, variants use lowercase names
/// (`"info"`, `"warning"`, `"error"`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational finding -- no action required.
    Info,
    /// Best practice suggestion -- the feed is valid but could be improved.
    Warning,
    /// Specification violation -- the feed is invalid and must be fixed.
    Error,
}

impl Display for Severity {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Error => write!(f, "ERROR"),
        }
    }
}
