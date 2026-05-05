//! Rule `too_many_rows` - detects files exceeding a configurable row limit.

use std::io::BufRead;

use crate::parser::FeedSource;
use crate::validation::file_structure::StructuralValidationRule;
use crate::validation::{Severity, ValidationError};

/// Checks that no file exceeds a maximum number of data rows.
///
/// The threshold is configurable. In MVP, the default is `None` (no limit),
/// effectively disabling this rule until a configuration value is set.
pub struct TooManyRowsRule {
    /// Maximum allowed data rows (excluding header). `None` disables the check.
    max_rows: Option<usize>,
}

impl TooManyRowsRule {
    /// Creates a new rule with the given row limit.
    #[must_use]
    pub fn new(max_rows: Option<usize>) -> Self {
        Self { max_rows }
    }
}

impl StructuralValidationRule for TooManyRowsRule {
    fn rule_id(&self) -> &'static str {
        "too_many_rows"
    }

    fn section(&self) -> &'static str {
        "1"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, source: &FeedSource) -> Vec<ValidationError> {
        let Some(max) = self.max_rows else {
            return vec![];
        };

        let mut errors = Vec::new();

        for file in source.file_names() {
            let Ok(reader) = source.read_file(file) else {
                continue;
            };

            let total_lines = reader.lines().count();
            let data_rows = total_lines.saturating_sub(1);

            if data_rows > max {
                let name = file.to_string();
                errors.push(
                    ValidationError::new(self.rule_id(), self.section(), self.severity())
                        .message(format!(
                            "File {name} has {data_rows} data rows, exceeding the limit of {max}"
                        ))
                        .file(name)
                        .value(format!("{data_rows}")),
                );
            }
        }

        errors
    }
}
