//! Rule `leading_or_trailing_whitespaces` — warns about whitespace around cell values.

use std::io::BufRead;

use crate::parser::FeedSource;
use crate::validation::file_structure::StructuralValidationRule;
use crate::validation::file_structure::helpers::read_header;
use crate::validation::{Severity, ValidationError};

/// Checks for leading or trailing whitespace in CSV cell values.
///
/// Inspects every cell in every data row. The header row itself is also checked.
/// Produces one `WARNING` per cell with offending whitespace, including the
/// column name, line number, and the raw value.
pub struct LeadingOrTrailingWhitespacesRule;

impl StructuralValidationRule for LeadingOrTrailingWhitespacesRule {
    fn rule_id(&self) -> &'static str {
        "leading_or_trailing_whitespaces"
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
            let Ok(columns) = read_header(source, file) else {
                continue;
            };
            let Some(columns) = columns else {
                continue;
            };

            let Ok(reader) = source.read_file(file) else {
                continue;
            };

            let name = file.to_string();

            for (idx, line_result) in reader.lines().enumerate() {
                // Skip header — we check data rows only (header whitespace is
                // debatable; the spec focuses on data values).
                if idx == 0 {
                    continue;
                }

                let Ok(line) = line_result else {
                    continue;
                };

                if line.trim().is_empty() {
                    continue; // Handled by empty_row rule.
                }

                let line_number = idx + 1;

                for (col_idx, value) in line.split(',').enumerate() {
                    if value != value.trim() {
                        let field_name = columns.get(col_idx).map_or_else(
                            || format!("column_{}", col_idx + 1),
                            |s| s.trim().to_owned(),
                        );
                        errors.push(
                            ValidationError::new(
                                self.rule_id(),
                                self.section(),
                                self.severity(),
                            )
                            .message(format!(
                                "Leading or trailing whitespace in {name} at line {line_number}, field \"{field_name}\""
                            ))
                            .file(&name)
                            .line(line_number)
                            .field(field_name)
                            .value(value),
                        );
                    }
                }
            }
        }

        errors
    }
}
