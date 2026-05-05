//! Section 3 - Field Type Validation.
//!
//! Validates field values in the parsed `GtfsFeed`: URLs, timezones, colors,
//! language codes, currencies, emails, phones, numeric ranges, and text quality.

pub mod field_type_rules;
pub mod numeric_rules;
pub mod parse_error_converter;
pub mod text_rules;

use crate::validation::engine::ValidationEngine;

/// Registers all section 3 (Field Type Validation) rules with the engine.
pub fn register_rules(engine: &mut ValidationEngine) {
    engine.register_rule(Box::new(field_type_rules::FieldTypeValidator));
    engine.register_rule(Box::new(numeric_rules::NumericRangeValidator));
    engine.register_rule(Box::new(text_rules::TextValidator));
}
