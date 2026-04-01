//! Rule `empty_row` — warns about data rows that contain only whitespace.

use std::io::BufRead;

use crate::parser::FeedSource;
use crate::validation::file_structure::StructuralValidationRule;
use crate::validation::{Severity, ValidationError};

/// Checks for data rows that are empty or contain only whitespace characters.
///
/// The header row (line 1) is excluded. Produces one `WARNING` per empty row.
pub struct EmptyRowRule;

impl StructuralValidationRule for EmptyRowRule {
    fn rule_id(&self) -> &'static str {
        "empty_row"
    }

    fn section(&self) -> &'static str {
        "1"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, source: &FeedSource) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for file in source.file_names() {
            let Ok(reader) = source.read_file(file) else {
                continue;
            };

            let name = file.to_string();

            for (idx, line_result) in reader.lines().enumerate() {
                if idx == 0 {
                    continue;
                }

                let Ok(line) = line_result else {
                    continue;
                };

                if line.trim().is_empty() {
                    let line_number = idx + 1; // 1-indexed
                    errors.push(
                        ValidationError::new(self.rule_id(), self.section(), self.severity())
                            .message(format!("Empty row in {name} at line {line_number}"))
                            .file(&name)
                            .line(line_number),
                    );
                }
            }
        }

        errors
    }
}
