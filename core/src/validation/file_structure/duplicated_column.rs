//! Rule `duplicated_column` — detects duplicate column names in CSV headers.

use std::collections::HashSet;

use crate::parser::FeedSource;
use crate::validation::file_structure::StructuralValidationRule;
use crate::validation::file_structure::helpers::read_header;
use crate::validation::{Severity, ValidationError};

/// Checks that no CSV header contains the same column name more than once.
///
/// Produces one `ERROR` per duplicated column, with the column name in `field`.
pub struct DuplicatedColumnRule;

impl StructuralValidationRule for DuplicatedColumnRule {
    fn rule_id(&self) -> &'static str {
        "duplicated_column"
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
            let Ok(Some(columns)) = read_header(source, file) else {
                continue;
            };

            let name = file.to_string();
            let mut seen = HashSet::new();

            for col in &columns {
                let trimmed = col.trim();
                if trimmed.is_empty() {
                    continue; // Handled by empty_column_name rule.
                }
                if !seen.insert(trimmed.to_owned()) {
                    errors.push(
                        ValidationError::new(self.rule_id(), self.section(), self.severity())
                            .message(format!(
                                "Duplicated column \"{trimmed}\" in header of {name}"
                            ))
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
