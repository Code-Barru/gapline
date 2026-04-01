//! Converts `ParseError`s from the parsing phase into `ValidationError`s
//! with appropriate section 3 rule IDs.

use crate::parser::error::{ParseError, ParseErrorKind};
use crate::validation::{Severity, ValidationError};

/// Converts parser-level `ParseError`s into structured `ValidationError`s.
pub fn convert(errors: &[ParseError]) -> Vec<ValidationError> {
    errors.iter().map(convert_one).collect()
}

fn convert_one(e: &ParseError) -> ValidationError {
    let (rule_id, severity) = match e.kind {
        ParseErrorKind::InvalidInteger => ("invalid_integer", Severity::Error),
        ParseErrorKind::InvalidFloat => ("invalid_float", Severity::Error),
        ParseErrorKind::InvalidDate => ("invalid_date", Severity::Error),
        ParseErrorKind::InvalidTime => ("invalid_time", Severity::Error),
        ParseErrorKind::InvalidEnum => ("unexpected_enum_value", Severity::Warning),
        ParseErrorKind::MissingRequired => ("missing_required_field", Severity::Error),
    };

    ValidationError::new(rule_id, "3", severity)
        .file(&e.file_name)
        .line(e.line_number)
        .field(&e.field_name)
        .value(&e.value)
        .message(format!(
            "{} in field '{}': '{}'",
            e.kind, e.field_name, e.value
        ))
}
