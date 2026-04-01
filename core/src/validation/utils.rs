//! Shared utilities for validation rules.

/// UTF-8 BOM as a byte sequence.
const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// Strips a leading UTF-8 BOM from raw bytes, if present.
#[must_use]
pub fn strip_bom(bytes: &[u8]) -> &[u8] {
    bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes)
}

/// Strips a leading UTF-8 BOM from a string, if present.
#[must_use]
pub fn strip_bom_str(s: &str) -> &str {
    s.strip_prefix('\u{FEFF}').unwrap_or(s)
}
