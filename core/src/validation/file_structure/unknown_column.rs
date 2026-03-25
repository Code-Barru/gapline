//! Rule `unknown_column` — reports columns not recognized by the GTFS spec for a given file.

use crate::parser::FeedSource;
use crate::validation::file_structure::StructuralValidationRule;
use crate::validation::file_structure::helpers::read_header;
use crate::validation::{Severity, ValidationError};

/// Reports columns in a GTFS file header that are not defined in the
/// GTFS Schedule Reference for that file.
///
/// Uses [`GtfsFiles::expected_columns`] to determine the recognized set.
/// Produces one `INFO` per unknown column.
pub struct UnknownColumnRule;

impl StructuralValidationRule for UnknownColumnRule {
    fn rule_id(&self) -> &'static str {
        "unknown_column"
    }

    fn section(&self) -> &'static str {
        "1"
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn validate(&self, source: &FeedSource) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for file in source.file_names() {
            let Ok(Some(columns)) = read_header(source, file) else {
                continue;
            };

            let expected = file.expected_columns();
            let name = file.to_string();

            for col in &columns {
                let trimmed = col.trim();
                if trimmed.is_empty() {
                    continue; // Handled by empty_column_name rule.
                }
                if !expected.contains(&trimmed) {
                    errors.push(
                        ValidationError::new(self.rule_id(), self.section(), self.severity())
                            .message(format!("Unknown column \"{trimmed}\" in {name}"))
                            .file(&name)
                            .line(1)
                            .field(trimmed),
                    );
                }
            }
        }

        errors
    }
}
