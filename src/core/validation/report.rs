use crate::validation::{Severity, ValidationError};

pub struct ValidationReport {
    errors: usize,
    warnings: usize,
    infos: usize,
}

impl ValidationReport {
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.errors
    }

    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.warnings
    }

    #[must_use]
    pub fn info_count(&self) -> usize {
        self.infos
    }

    #[must_use]
    pub fn has_errors(&self) -> bool {
        if self.errors > 0 {
            return true;
        }
        false
    }
}

impl From<&Vec<ValidationError>> for ValidationReport {
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
