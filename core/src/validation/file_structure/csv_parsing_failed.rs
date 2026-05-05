//! Rule `csv_parsing_failed` - catch-all for CSV parsing errors (encoding, malformed data).

use std::io::Read;

use crate::parser::FeedSource;
use crate::validation::file_structure::StructuralValidationRule;
use crate::validation::{Severity, ValidationError};

/// Catch-all rule that attempts to read each file as valid UTF-8 and detect
/// fundamental CSV parsing failures not covered by more specific rules.
///
/// Currently checks for invalid UTF-8 encoding. Future iterations may add
/// additional checks (e.g. RFC 4180 violations).
pub struct CsvParsingFailedRule;

impl StructuralValidationRule for CsvParsingFailedRule {
    fn rule_id(&self) -> &'static str {
        "csv_parsing_failed"
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

            let name = file.to_string();

            // Try reading the entire file as UTF-8.
            let mut content = String::new();
            if let Err(e) = reader.read_to_string(&mut content) {
                errors.push(
                    ValidationError::new(self.rule_id(), self.section(), self.severity())
                        .message(format!("Failed to parse {name} as CSV: {e}"))
                        .file(name),
                );
            }
        }

        errors
    }
}
