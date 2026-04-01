//! Rules `invalid_quoting` (CA5) and `invalid_inner_quotes` (CA6) —
//! RFC 4180 quoting compliance.

use std::io::Read;

use crate::parser::FeedSource;
use crate::validation::utils::strip_bom;
use crate::validation::{Severity, StructuralValidationRule, ValidationError};

/// Checks RFC 4180 quoting: values containing commas or quotes must be quoted,
/// and inner quotes must be doubled (`""`).
pub struct InvalidQuotingRule;

/// State machine for CSV field parsing.
#[derive(Clone, Copy, PartialEq)]
enum State {
    /// At the start of a field (after comma or line start).
    FieldStart,
    /// Inside an unquoted field.
    InUnquoted,
    /// Inside a quoted field.
    InQuoted,
    /// Just saw a `"` inside a quoted field — could be closing quote or escaped quote.
    AfterQuote,
}

impl StructuralValidationRule for InvalidQuotingRule {
    fn rule_id(&self) -> &'static str {
        "invalid_quoting"
    }

    fn section(&self) -> &'static str {
        "2"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    #[allow(clippy::too_many_lines)]
    fn validate(&self, source: &FeedSource) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for file in source.file_names() {
            let Ok(mut reader) = source.read_file(file) else {
                continue;
            };

            let mut bytes = Vec::new();
            if reader.read_to_end(&mut bytes).is_err() {
                continue;
            }

            let data = strip_bom(&bytes);

            let Ok(content) = std::str::from_utf8(data) else {
                continue;
            };

            let file_name = file.to_string();
            let mut state = State::FieldStart;
            let mut line_num: usize = 1;
            let mut field_start_in_line = 0;
            let mut current_field = String::new();

            for ch in content.chars() {
                match state {
                    State::FieldStart => match ch {
                        '"' => {
                            state = State::InQuoted;
                            current_field.clear();
                        }
                        ',' | '\r' => {
                            current_field.clear();
                        }
                        '\n' => {
                            line_num += 1;
                            current_field.clear();
                        }
                        _ => {
                            state = State::InUnquoted;
                            current_field.clear();
                            current_field.push(ch);
                            field_start_in_line = line_num;
                        }
                    },
                    State::InUnquoted => match ch {
                        ',' | '\n' | '\r' => {
                            state = if ch == ',' {
                                State::FieldStart
                            } else {
                                if ch == '\n' {
                                    line_num += 1;
                                }
                                State::FieldStart
                            };
                            current_field.clear();
                        }
                        '"' => {
                            errors.push(
                                ValidationError::new(
                                    self.rule_id(),
                                    self.section(),
                                    self.severity(),
                                )
                                .message("Quote character in unquoted field")
                                .file(file_name.clone())
                                .line(line_num),
                            );
                            current_field.push(ch);
                        }
                        _ => {
                            current_field.push(ch);
                        }
                    },
                    State::InQuoted => match ch {
                        '"' => {
                            state = State::AfterQuote;
                        }
                        '\n' => {
                            line_num += 1;
                            current_field.push(ch);
                        }
                        _ => {
                            current_field.push(ch);
                        }
                    },
                    State::AfterQuote => match ch {
                        '"' => {
                            state = State::InQuoted;
                            current_field.push('"');
                        }
                        ',' | '\r' => {
                            state = State::FieldStart;
                            current_field.clear();
                        }
                        '\n' => {
                            state = State::FieldStart;
                            line_num += 1;
                            current_field.clear();
                        }
                        _ => {
                            errors.push(
                                ValidationError::new(
                                    "invalid_inner_quotes",
                                    self.section(),
                                    self.severity(),
                                )
                                .message(format!(
                                    "Character '{ch}' after closing quote; inner quotes must be doubled (\"\")"
                                ))
                                .file(file_name.clone())
                                .line(field_start_in_line.max(line_num)),
                            );
                            state = State::InUnquoted;
                            current_field.push(ch);
                        }
                    },
                }

                if state == State::InQuoted && current_field.is_empty() {
                    field_start_in_line = line_num;
                }
            }

            if state == State::InQuoted {
                errors.push(
                    ValidationError::new(self.rule_id(), self.section(), self.severity())
                        .message("Unclosed quoted field at end of file")
                        .file(file_name.clone())
                        .line(line_num),
                );
            }
        }

        errors
    }
}
