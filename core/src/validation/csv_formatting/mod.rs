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
