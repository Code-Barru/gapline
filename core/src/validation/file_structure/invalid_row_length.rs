//! Rule `invalid_row_length` — detects rows with a different number of fields than the header.

use std::io::BufRead;

use crate::parser::FeedSource;
use crate::validation::file_structure::StructuralValidationRule;
use crate::validation::file_structure::helpers::read_header;
use crate::validation::{Severity, ValidationError};

/// Checks that every data row has the same number of comma-separated fields as
/// the header row.
///
/// Produces one `ERROR` per mismatched row, including the line number and the
/// actual field count found.
pub struct InvalidRowLengthRule;

impl StructuralValidationRule for InvalidRowLengthRule {
    fn rule_id(&self) -> &'static str {
        "invalid_row_length"
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
            let expected = columns.len();
            let name = file.to_string();

            let Ok(reader) = source.read_file(file) else {
                continue;
            };

            for (idx, line_result) in reader.lines().enumerate() {
                if idx == 0 {
                    continue;
                }

                let Ok(line) = line_result else {
                    continue;
                };

                if line.trim().is_empty() {
                    continue;
                }

                let actual = line.split(',').count();
                if actual != expected {
                    let line_number = idx + 1; // 1-indexed
                    errors.push(
                        ValidationError::new(self.rule_id(), self.section(), self.severity())
                            .message(format!(
                                "Row has {actual} fields but header has {expected} columns in {name}"
                            ))
                            .file(&name)
                            .line(line_number)
                            .value(format!("{actual}")),
                    );
                }
            }
        }

        errors
    }
}
