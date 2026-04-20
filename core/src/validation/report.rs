use serde::Serialize;

use crate::validation::{Severity, ValidationError};

/// Aggregated summary of validation findings by severity.
///
/// A `ValidationReport` is built from a list of [`ValidationError`]s and provides
/// quick access to counts per severity level. It does not store the individual
/// errors -- it only holds the tallied counts.
///
/// # Examples
///
/// ```
/// use gapline_core::validation::{ValidationError, ValidationReport, Severity};
///
/// let errors = vec![
///     ValidationError::new("rule_a", "1", Severity::Error)
///         .message("Something is wrong"),
///     ValidationError::new("rule_b", "8", Severity::Warning)
///         .message("Consider improving this"),
/// ];
///
/// let report = ValidationReport::from(errors);
/// assert_eq!(report.error_count(), 1);
/// assert_eq!(report.warning_count(), 1);
/// assert!(report.has_errors());
/// ```
#[derive(Serialize)]
pub struct ValidationReport {
    /// Number of findings with [`Severity::Error`].
    errors: usize,
    /// Number of findings with [`Severity::Warning`].
    warnings: usize,
    /// Number of findings with [`Severity::Info`].
    infos: usize,
    /// Array containing [`ValidationError`].
    error_list: Vec<ValidationError>,
}

impl ValidationReport {
    /// Returns the number of [`Severity::Error`] findings.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.errors
    }

    /// Returns the number of [`Severity::Warning`] findings.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.warnings
    }

    /// Returns the number of [`Severity::Info`] findings.
    #[must_use]
    pub fn info_count(&self) -> usize {
        self.infos
    }

    /// Returns `true` if the report contains at least one [`Severity::Error`].
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.errors > 0
    }

    /// Returns a reference to the list of validation errors.
    #[must_use]
    pub fn errors(&self) -> &[ValidationError] {
        &self.error_list
    }

    /// Returns references to the validation errors sorted by file name.
    ///
    /// Errors with a file name come first (alphabetically), followed by
    /// errors without a file name.
    #[must_use]
    pub fn errors_sorted_by_file(&self) -> Vec<&ValidationError> {
        let mut sorted: Vec<&ValidationError> = self.error_list.iter().collect();
        sorted.sort_by(|a, b| match (&a.file_name, &b.file_name) {
            (Some(fa), Some(fb)) => fa.cmp(fb),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
        sorted
    }
}

impl From<Vec<ValidationError>> for ValidationReport {
    /// Builds a report by counting each [`ValidationError`] by its severity.
    fn from(error_list: Vec<ValidationError>) -> ValidationReport {
        let (mut errors, mut warnings, mut infos) = (0, 0, 0);
        for e in &error_list {
            match e.severity {
                Severity::Error => errors += 1,
                Severity::Warning => warnings += 1,
                Severity::Info => infos += 1,
            }
        }

        Self {
            errors,
            warnings,
            infos,
            error_list,
        }
    }
}
