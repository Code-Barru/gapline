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
/// use headway::validation::{ValidationError, ValidationReport, Severity};
///
/// let errors = vec![
///     ValidationError::new("rule_a", "1", Severity::Error)
///         .message("Something is wrong"),
///     ValidationError::new("rule_b", "8", Severity::Warning)
///         .message("Consider improving this"),
/// ];
///
/// let report = ValidationReport::from(&errors);
/// assert_eq!(report.error_count(), 1);
/// assert_eq!(report.warning_count(), 1);
/// assert!(report.has_errors());
/// ```
pub struct ValidationReport {
    /// Number of findings with [`Severity::Error`].
    errors: usize,
    /// Number of findings with [`Severity::Warning`].
    warnings: usize,
    /// Number of findings with [`Severity::Info`].
    infos: usize,
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
        if self.errors > 0 {
            return true;
        }
        false
    }
}

impl From<&Vec<ValidationError>> for ValidationReport {
    /// Builds a report by counting each [`ValidationError`] by its severity.
    fn from(errors: &Vec<ValidationError>) -> ValidationReport {
        let error_count = errors
            .iter()
            .filter(|e| e.severity == Severity::Error)
            .count();
        let warning_count = errors
            .iter()
            .filter(|e| e.severity == Severity::Warning)
            .count();
        let info_count = errors
            .iter()
            .filter(|e| e.severity == Severity::Info)
            .count();

        Self {
            errors: error_count,
            warnings: warning_count,
            infos: info_count,
        }
    }
}
