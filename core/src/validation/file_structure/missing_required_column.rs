//! Rule `missing_required_column` - reports required columns absent from a GTFS file header.

use crate::parser::FeedSource;
use crate::validation::file_structure::helpers::read_header;
use crate::validation::{Severity, StructuralValidationRule, ValidationError};

/// Reports required columns that are missing from a GTFS file header.
///
/// Uses [`GtfsFiles::required_columns`] to determine the required set for each file
/// and compares it against the actual CSV header. Produces one `ERROR` per missing
/// required column.
pub struct MissingRequiredColumnRule;

impl StructuralValidationRule for MissingRequiredColumnRule {
    fn rule_id(&self) -> &'static str {
        "missing_required_column"
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

            let required = file.required_columns();
            let name = file.to_string();

            let present: Vec<&str> = columns.iter().map(|c| c.trim()).collect();

            for &col in required {
                if !present.contains(&col) {
                    errors.push(
                        ValidationError::new(self.rule_id(), self.section(), self.severity())
                            .message(format!("Required column \"{col}\" is missing from {name}"))
                            .file(&name)
                            .line(1)
                            .field(col),
                    );
                }
            }
        }

        errors
    }
}
