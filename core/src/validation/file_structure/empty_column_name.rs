//! Rule `empty_column_name` — detects empty column names in CSV headers.

use crate::parser::FeedSource;
use crate::validation::file_structure::StructuralValidationRule;
use crate::validation::file_structure::helpers::read_header;
use crate::validation::{Severity, ValidationError};

/// Checks that no CSV header contains an empty (or whitespace-only) column name.
///
/// For example, the header `"stop_id,,stop_name"` has an empty column at
/// position 2. Produces one `ERROR` per empty column found.
pub struct EmptyColumnNameRule;

impl StructuralValidationRule for EmptyColumnNameRule {
    fn rule_id(&self) -> &'static str {
        "empty_column_name"
    }

    fn section(&self) -> &'static str {
        "1"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, source: &FeedSource) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for file in source.file_names() {
            let Ok(columns) = read_header(source, file) else {
                continue;
            };

            let name = file.to_string();

            for (idx, col) in columns.iter().enumerate() {
                if col.trim().is_empty() {
                    errors.push(
                        ValidationError::new(self.rule_id(), self.section(), self.severity())
                            .message(format!(
                                "Header of {name} contains an empty column name at position {}",
                                idx + 1
                            ))
                            .file(&name)
                            .line(1),
                    );
                }
            }
        }

        errors
    }
}
