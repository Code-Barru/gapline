//! Rule `invalid_encoding` — rejects files not encoded in valid UTF-8.
//!
//! A UTF-8 BOM (`\xEF\xBB\xBF`) at the start of a file is accepted and ignored.
//! Any other encoding (Latin-1, UTF-16, etc.) produces an ERROR.

use std::io::Read;

use crate::parser::FeedSource;
use crate::validation::utils::strip_bom;
use crate::validation::{Severity, StructuralValidationRule, ValidationError};

/// Checks that every file in the feed is valid UTF-8.
pub struct InvalidEncodingRule;

impl StructuralValidationRule for InvalidEncodingRule {
    fn rule_id(&self) -> &'static str {
        "invalid_encoding"
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

            let data = strip_bom(&bytes);

            if let Err(e) = std::str::from_utf8(data) {
                let offset = e.valid_up_to();
                #[allow(clippy::naive_bytecount)]
                let line = data[..offset].iter().filter(|&&b| b == b'\n').count() + 1;

                errors.push(
                    ValidationError::new(self.rule_id(), self.section(), self.severity())
                        .message(format!(
                            "File is not valid UTF-8 (invalid byte at offset {offset})"
                        ))
                        .file(file.to_string())
                        .line(line),
                );
            }
        }

        errors
    }
}
