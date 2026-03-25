//! Rule `invalid_input_files_in_subfolder` — detects GTFS files nested in subdirectories.

use crate::parser::FeedSource;
use crate::validation::file_structure::StructuralValidationRule;
use crate::validation::{Severity, ValidationError};

/// Checks that no recognized GTFS file names appear inside a subdirectory
/// within the archive.
///
/// For example, `gtfs/agency.txt` instead of `agency.txt` at the root level.
/// Uses [`FeedSource::raw_entry_names`] to inspect the original archive paths.
pub struct InvalidInputFilesInSubfolderRule;

impl StructuralValidationRule for InvalidInputFilesInSubfolderRule {
    fn rule_id(&self) -> &'static str {
        "invalid_input_files_in_subfolder"
    }

    fn section(&self) -> &'static str {
        "1"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, source: &FeedSource) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for raw_name in source.raw_entry_names() {
            if raw_name.contains('/') {
                errors.push(
                    ValidationError::new(self.rule_id(), self.section(), self.severity())
                        .message(format!("GTFS file found in subdirectory: {raw_name}"))
                        .file(raw_name),
                );
            }
        }

        errors
    }
}
