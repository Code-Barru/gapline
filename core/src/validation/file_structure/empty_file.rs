//! Rule `empty_file` — detects GTFS files that are empty or contain only a header.

use std::io::BufRead;

use crate::parser::FeedSource;
use crate::validation::file_structure::StructuralValidationRule;
use crate::validation::{Severity, ValidationError};

/// Checks that every known GTFS file in the feed has at least a header **and**
/// one data row.
///
/// Two cases trigger an error:
/// - The file is completely empty (0 bytes).
/// - The file has a header line but zero data lines.
pub struct EmptyFileRule;

impl StructuralValidationRule for EmptyFileRule {
    fn rule_id(&self) -> &'static str {
        "empty_file"
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

            let mut first_line = String::new();
            let Ok(bytes_read) = reader.read_line(&mut first_line) else {
                continue;
            };

            let name = file.to_string();

            // Case 1: completely empty file (0 bytes).
            if bytes_read == 0 {
                errors.push(
                    ValidationError::new(self.rule_id(), self.section(), self.severity())
                        .message(format!("File {name} is empty (0 bytes)"))
                        .file(name),
                );
                continue;
            }

            // Case 2: header present but no data rows.
            let mut second_line = String::new();
            let has_data = reader.read_line(&mut second_line).is_ok_and(|n| n > 0);

            if !has_data {
                errors.push(
                    ValidationError::new(self.rule_id(), self.section(), self.severity())
                        .message(format!("File {name} has a header but no data rows"))
                        .file(name),
                );
            }
        }

        errors
    }
}
