//! Rule `superfluous_whitespace` (CA9) — warns about leading or trailing spaces
//! around field values adjacent to the delimiter.

use std::io::Read;

use crate::parser::FeedSource;
use crate::validation::{Severity, StructuralValidationRule, ValidationError};

/// Warns when fields have leading or trailing spaces next to the delimiter.
///
/// This is the only WARNING-level rule in section 2.
pub struct SuperfluousWhitespaceRule;

impl StructuralValidationRule for SuperfluousWhitespaceRule {
    fn rule_id(&self) -> &'static str {
        "superfluous_whitespace"
    }

    fn section(&self) -> &'static str {
        "2"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
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

            let data = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
                &bytes[3..]
            } else {
                &bytes
            };

            let Ok(content) = std::str::from_utf8(data) else {
                continue;
            };

            let file_name = file.to_string();

            for (line_idx, line) in content.lines().enumerate() {
                let line_num = line_idx + 1;

                // Simple split by comma, skipping quoted fields.
                let fields = split_respecting_quotes(line);

                for field in &fields {
                    // Only check unquoted fields or the raw representation.
                    // A field that starts/ends with space (and is not quoted) is suspect.
                    if !field.starts_with('"') && (field.starts_with(' ') || field.ends_with(' ')) {
                        errors.push(
                            ValidationError::new(self.rule_id(), self.section(), self.severity())
                                .message("Superfluous whitespace around field value")
                                .file(file_name.clone())
                                .line(line_num)
                                .value(field.to_string()),
                        );
                    }
                }
            }
        }

        errors
    }
}

/// Splits a CSV line by comma while respecting quoted fields.
fn split_respecting_quotes(line: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;

    for (i, ch) in line.char_indices() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                fields.push(&line[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    fields.push(&line[start..]);
    fields
}
