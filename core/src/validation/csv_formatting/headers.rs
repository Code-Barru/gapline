//! Rule `missing_header` - detects files whose first line looks like data
//! rather than column headers.

use crate::parser::FeedSource;
use crate::validation::file_structure::helpers;
use crate::validation::{Severity, StructuralValidationRule, ValidationError};

/// Checks that the first line of each file contains header names, not data.
///
/// Heuristic: if every field in the first line parses as a number, the file
/// likely has no header row.
pub struct MissingHeaderRule;

impl StructuralValidationRule for MissingHeaderRule {
    fn rule_id(&self) -> &'static str {
        "missing_header"
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
            let Ok(columns) = helpers::read_header(source, file) else {
                continue;
            };

            // If the header row is empty or has a single empty column, skip
            // (empty_file rule in section 1 handles that).
            if columns.is_empty() || (columns.len() == 1 && columns[0].is_empty()) {
                continue;
            }

            // Heuristic: all fields are numeric → likely missing header.
            let all_numeric = columns
                .iter()
                .all(|c| !c.trim().is_empty() && c.trim().parse::<f64>().is_ok());

            if all_numeric {
                errors.push(
                    ValidationError::new(self.rule_id(), self.section(), self.severity())
                        .message(
                            "First line appears to contain data instead of column headers \
                             (all fields are numeric)",
                        )
                        .file(file.to_string())
                        .line(1),
                );
            }
        }

        errors
    }
}
