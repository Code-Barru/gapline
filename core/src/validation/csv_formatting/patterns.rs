//! Shared regex patterns for CSV content rules (HTML tags/comments, literal
//! escape sequences). Used by both the single-pass `scanner` and the standalone
//! `content` rule.

use std::sync::LazyLock;

use regex::Regex;

pub static HTML_TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<[a-zA-Z/][^>]*>").expect("invalid regex"));

pub static HTML_COMMENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<!--.*?-->").expect("invalid regex"));

/// Matches literal backslash followed by n, t, or r (the text `\n`, not the byte 0x0A).
pub static LITERAL_ESCAPE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\[ntr]").expect("invalid regex"));
