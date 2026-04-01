//! Rule `new_line_in_value` — detects unescaped newlines inside quoted CSV values.

use std::io::Read;

use crate::parser::FeedSource;
use crate::validation::file_structure::StructuralValidationRule;
use crate::validation::{Severity, ValidationError};

/// Detects newline characters inside quoted CSV values (unclosed quotes).
///
/// Uses a simple state machine that tracks whether we are inside a quoted field.
/// When a newline is encountered while inside quotes, an error is emitted for
/// the line where the opening quote was found.
pub struct NewLineInValueRule;

impl StructuralValidationRule for NewLineInValueRule {
    fn rule_id(&self) -> &'static str {
        "new_line_in_value"
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
            let Ok(mut reader) = source.read_file(file) else {
                continue;
            };

            let mut content = String::new();
            if reader.read_to_string(&mut content).is_err() {
                continue;
            }

            let name = file.to_string();
            let mut in_quotes = false;
            let mut line_number: usize = 1;
            let mut quote_open_line: usize = 0;

            let chars: Vec<char> = content.chars().collect();
            let len = chars.len();
            let mut i = 0;

            while i < len {
                let ch = chars[i];

                if ch == '"' {
                    if in_quotes {
                        if i + 1 < len && chars[i + 1] == '"' {
                            i += 2;
                            continue;
                        }
                        in_quotes = false;
                    } else {
                        in_quotes = true;
                        quote_open_line = line_number;
                    }
                } else if ch == '\n' {
                    if in_quotes {
                        errors.push(
                            ValidationError::new(self.rule_id(), self.section(), self.severity())
                                .message(format!(
                                    "Newline found inside quoted value in {name} (quote opened at line {quote_open_line})"
                                ))
                                .file(&name)
                                .line(quote_open_line),
                        );
                        in_quotes = false;
                    }
                    line_number += 1;
                } else if ch == '\r' {
                    // Don't increment line_number for \r, only for \n.
                }

                i += 1;
            }
        }

        errors
    }
}
