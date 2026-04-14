//! CSV formatting validation rules (section 2).
//!
//! These rules check low-level CSV compliance **before** data is parsed into
//! memory: UTF-8 encoding, comma delimiter, RFC 4180 quoting, forbidden content,
//! etc. Together with section 1 (`file_structure`), they form the structural gate
//! that must pass without any `ERROR` for feed loading to proceed.

mod case_sensitivity;
mod content;
mod delimiter;
mod encoding;
mod headers;
mod quoting;
pub mod scanner;
mod whitespace;

pub use case_sensitivity::CaseSensitiveRule;
pub use content::InvalidContentRule;
pub use delimiter::InvalidDelimiterRule;
pub use encoding::InvalidEncodingRule;
pub use headers::MissingHeaderRule;
pub use quoting::InvalidQuotingRule;
pub use whitespace::SuperfluousWhitespaceRule;

use crate::validation::StructuralValidationRule;

/// Returns every pre-parsing rule owned by this module. The 6 content-scanning
/// rules (`encoding`, `delimiter`, `quoting`, `content`, `whitespace`,
/// `new_line_in_value`) are handled by [`scanner::scan`] in a single pass and
/// are not listed here.
#[must_use]
pub fn pre_rules() -> Vec<Box<dyn StructuralValidationRule>> {
    vec![Box::new(MissingHeaderRule), Box::new(CaseSensitiveRule)]
}
