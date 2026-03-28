//! Rule `invalid_delimiter` — ensures the CSV delimiter is a comma and line
//! endings are CRLF or LF (not bare CR).

use std::io::Read;

use crate::parser::FeedSource;
use crate::validation::{Severity, StructuralValidationRule, ValidationError};

/// Checks that every file uses comma as the delimiter and valid line endings.
pub struct InvalidDelimiterRule;

impl StructuralValidationRule for InvalidDelimiterRule {
    fn rule_id(&self) -> &'static str {
        "invalid_delimiter"
    }

    fn section(&self) -> &'static str {
        "2"
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

            let mut bytes = Vec::new();
            if reader.read_to_end(&mut bytes).is_err() {
                continue;
            }

            // Strip BOM.
            let data = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
                &bytes[3..]
            } else {
                &bytes
            };

            // Must be valid UTF-8 for delimiter analysis (encoding rule catches this).
            let Ok(content) = std::str::from_utf8(data) else {
                continue;
            };

            // Check delimiter on the first line.
            if let Some(first_line) = content.lines().next() {
                let commas = first_line.matches(',').count();
                let semicolons = first_line.matches(';').count();
                let tabs = first_line.matches('\t').count();

                if semicolons > 0 && semicolons >= commas {
                    errors.push(
                        ValidationError::new(self.rule_id(), self.section(), self.severity())
                            .message("Semicolon delimiter detected; comma is required")
                            .file(file.to_string())
                            .line(1)
                            .value(";"),
                    );
                } else if tabs > 0 && tabs >= commas {
                    errors.push(
                        ValidationError::new(self.rule_id(), self.section(), self.severity())
                            .message("Tab delimiter detected; comma is required")
                            .file(file.to_string())
                            .line(1)
                            .value("\\t"),
                    );
                }
            }

            // Check for bare CR line endings (CR not followed by LF).
            for (i, window) in data.windows(2).enumerate() {
                if window[0] == b'\r' && window[1] != b'\n' {
                    #[allow(clippy::naive_bytecount)]
                    let line = data[..i].iter().filter(|&&b| b == b'\n').count() + 1;
                    errors.push(
                        ValidationError::new(self.rule_id(), self.section(), self.severity())
                            .message("Bare carriage return (CR) line ending; use CRLF or LF")
                            .file(file.to_string())
                            .line(line)
                            .value("\\r"),
                    );
                    // Report only the first occurrence per file.
                    break;
                }
            }
            // Also check if the last byte is a lone CR.
            if data.last() == Some(&b'\r') {
                #[allow(clippy::naive_bytecount)]
                let line = data.iter().filter(|&&b| b == b'\n').count() + 1;
                errors.push(
                    ValidationError::new(self.rule_id(), self.section(), self.severity())
                        .message("File ends with a bare carriage return (CR); use CRLF or LF")
                        .file(file.to_string())
                        .line(line),
                );
            }
        }

        errors
    }
}
