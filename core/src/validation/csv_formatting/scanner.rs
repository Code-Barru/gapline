//! Single-pass CSV format scanner.
//!
//! Merges the logic of 6 validation rules into one character-by-character pass
//! per file, eliminating redundant I/O and UTF-8 decoding:
//!
//! - `invalid_encoding` (CA1)
//! - `invalid_delimiter` (CA3/CA4)
//! - `invalid_quoting` / `invalid_inner_quotes` (CA5/CA6)
//! - `control_character` / `forbidden_content` (CA7/CA8)
//! - `superfluous_whitespace` (CA9)
//! - `new_line_in_value` (section 1)

use std::sync::LazyLock;

use regex::Regex;

use crate::parser::feed_source::GtfsFiles;
use crate::validation::utils::strip_bom;
use crate::validation::{Severity, ValidationError};

static HTML_TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<[a-zA-Z/][^>]*>").expect("invalid regex"));
static HTML_COMMENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<!--.*?-->").expect("invalid regex"));
static LITERAL_ESCAPE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\[ntr]").expect("invalid regex"));

/// Quote-tracking state machine (RFC 4180).
#[derive(Clone, Copy, PartialEq)]
enum State {
    FieldStart,
    InUnquoted,
    InQuoted,
    AfterQuote,
}

/// Scans the raw bytes of a single GTFS CSV file, running all section-2
/// formatting checks plus `new_line_in_value` in a single pass.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn scan(file: GtfsFiles, raw_bytes: &[u8]) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let file_name = file.to_string();

    // --- Encoding check ---
    let data = strip_bom(raw_bytes);
    let content = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(e) => {
            let offset = e.valid_up_to();
            #[allow(clippy::naive_bytecount)]
            let line = data[..offset].iter().filter(|&&b| b == b'\n').count() + 1;
            errors.push(
                ValidationError::new("invalid_encoding", "2", Severity::Error)
                    .message(format!(
                        "File is not valid UTF-8 (invalid byte at offset {offset})"
                    ))
                    .file(&file_name)
                    .line(line),
            );
            return errors;
        }
    };

    if content.is_empty() {
        return errors;
    }

    // --- Delimiter sniffing (first line only) ---
    if let Some(first_line) = content.lines().next() {
        let commas = first_line.matches(',').count();
        let semicolons = first_line.matches(';').count();
        let tabs = first_line.matches('\t').count();

        if semicolons > 0 && semicolons >= commas {
            errors.push(
                ValidationError::new("invalid_delimiter", "2", Severity::Error)
                    .message("Semicolon delimiter detected; comma is required")
                    .file(&file_name)
                    .line(1)
                    .value(";"),
            );
        } else if tabs > 0 && tabs >= commas {
            errors.push(
                ValidationError::new("invalid_delimiter", "2", Severity::Error)
                    .message("Tab delimiter detected; comma is required")
                    .file(&file_name)
                    .line(1)
                    .value("\\t"),
            );
        }
    }

    // --- Single-pass character scan ---
    let mut state = State::FieldStart;
    let mut line_num: usize = 1;
    let mut field_start_line: usize = 1;
    let mut quote_open_line: usize = 0;

    // Per-field tracking for whitespace
    let mut field_leading_space = false;
    let mut field_trailing_space = false;
    let mut field_is_quoted = false;

    // Per-line tracking for content checks (control chars, regex)
    let mut line_start_byte: usize = 0;

    // Bare CR detection (delimiter rule)
    let mut found_bare_cr = false;

    let bytes = content.as_bytes();

    for (byte_idx, ch) in content.char_indices() {
        match state {
            State::FieldStart => {
                field_leading_space = false;
                field_trailing_space = false;
                field_is_quoted = false;

                match ch {
                    '"' => {
                        state = State::InQuoted;
                        field_is_quoted = true;
                        field_start_line = line_num;
                        quote_open_line = line_num;
                    }
                    ',' | '\r' => {
                        // Empty field or CR — nothing to check
                    }
                    '\n' => {
                        // End of line — run line-level content checks
                        run_line_content_checks(
                            content,
                            line_start_byte,
                            byte_idx,
                            line_num,
                            &file_name,
                            &mut errors,
                        );
                        line_num += 1;
                        line_start_byte = byte_idx + 1;
                    }
                    _ => {
                        state = State::InUnquoted;
                        field_start_line = line_num;
                        field_leading_space = ch == ' ';
                        field_trailing_space = ch == ' ';
                    }
                }
            }
            State::InUnquoted => match ch {
                ',' => {
                    emit_whitespace_warning(
                        field_leading_space,
                        field_trailing_space,
                        field_is_quoted,
                        &file_name,
                        line_num,
                        &mut errors,
                    );
                    state = State::FieldStart;
                }
                '\r' => {
                    // Possible end of field, wait for \n
                }
                '\n' => {
                    emit_whitespace_warning(
                        field_leading_space,
                        field_trailing_space,
                        field_is_quoted,
                        &file_name,
                        line_num,
                        &mut errors,
                    );
                    run_line_content_checks(
                        content,
                        line_start_byte,
                        byte_idx,
                        line_num,
                        &file_name,
                        &mut errors,
                    );
                    state = State::FieldStart;
                    line_num += 1;
                    line_start_byte = byte_idx + 1;
                }
                '"' => {
                    errors.push(
                        ValidationError::new("invalid_quoting", "2", Severity::Error)
                            .message("Quote character in unquoted field")
                            .file(&file_name)
                            .line(line_num),
                    );
                    field_trailing_space = false;
                }
                _ => {
                    field_trailing_space = ch == ' ';
                }
            },
            State::InQuoted => match ch {
                '"' => {
                    state = State::AfterQuote;
                }
                '\n' => {
                    // Newline inside quoted value
                    errors.push(
                        ValidationError::new("new_line_in_value", "1", Severity::Error)
                            .message(format!(
                                "Newline found inside quoted value in {file_name} (quote opened at line {quote_open_line})"
                            ))
                            .file(&file_name)
                            .line(quote_open_line),
                    );
                    // Don't run line-level content checks for lines inside quoted values
                    line_num += 1;
                    line_start_byte = byte_idx + 1;
                }
                _ => {}
            },
            State::AfterQuote => match ch {
                '"' => {
                    // Escaped quote ""
                    state = State::InQuoted;
                }
                ',' | '\r' => {
                    state = State::FieldStart;
                }
                '\n' => {
                    run_line_content_checks(
                        content,
                        line_start_byte,
                        byte_idx,
                        line_num,
                        &file_name,
                        &mut errors,
                    );
                    state = State::FieldStart;
                    line_num += 1;
                    line_start_byte = byte_idx + 1;
                }
                _ => {
                    errors.push(
                        ValidationError::new("invalid_inner_quotes", "2", Severity::Error)
                            .message(format!(
                                "Character '{ch}' after closing quote; inner quotes must be doubled (\"\")"
                            ))
                            .file(&file_name)
                            .line(field_start_line.max(line_num)),
                    );
                    state = State::InUnquoted;
                    field_trailing_space = ch == ' ';
                }
            },
        }

        // Bare CR detection (not inside quotes)
        if !found_bare_cr
            && state != State::InQuoted
            && ch != '\r'
            && byte_idx > 0
            && bytes[byte_idx - 1] == b'\r'
            && ch != '\n'
        {
            #[allow(clippy::naive_bytecount)]
            let cr_line = bytes[..byte_idx - 1]
                .iter()
                .filter(|&&b| b == b'\n')
                .count()
                + 1;
            errors.push(
                ValidationError::new("invalid_delimiter", "2", Severity::Error)
                    .message("Bare carriage return (CR) line ending; use CRLF or LF")
                    .file(&file_name)
                    .line(cr_line)
                    .value("\\r"),
            );
            found_bare_cr = true;
        }
    }

    // End-of-file checks
    if state == State::InQuoted {
        errors.push(
            ValidationError::new("invalid_quoting", "2", Severity::Error)
                .message("Unclosed quoted field at end of file")
                .file(&file_name)
                .line(line_num),
        );
    }

    // Check last line content (if file doesn't end with \n)
    if line_start_byte < content.len() {
        run_line_content_checks(
            content,
            line_start_byte,
            content.len(),
            line_num,
            &file_name,
            &mut errors,
        );
    }

    // Trailing bare CR
    if !found_bare_cr && data.last() == Some(&b'\r') {
        #[allow(clippy::naive_bytecount)]
        let line = data.iter().filter(|&&b| b == b'\n').count() + 1;
        errors.push(
            ValidationError::new("invalid_delimiter", "2", Severity::Error)
                .message("File ends with a bare carriage return (CR); use CRLF or LF")
                .file(&file_name)
                .line(line),
        );
    }

    errors
}

fn emit_whitespace_warning(
    leading: bool,
    trailing: bool,
    is_quoted: bool,
    file_name: &str,
    line_num: usize,
    errors: &mut Vec<ValidationError>,
) {
    if !is_quoted && (leading || trailing) {
        errors.push(
            ValidationError::new("superfluous_whitespace", "2", Severity::Warning)
                .message("Superfluous whitespace around field value")
                .file(file_name)
                .line(line_num),
        );
    }
}

/// Runs per-line content checks: control characters, HTML tags, HTML comments,
/// literal escape sequences.
fn run_line_content_checks(
    content: &str,
    start: usize,
    end: usize,
    line_num: usize,
    file_name: &str,
    errors: &mut Vec<ValidationError>,
) {
    let line = &content[start..end];
    let line_trimmed = line.trim_end_matches('\r');

    // Control character checks (byte-level)
    let line_bytes = line_trimmed.as_bytes();

    // Tab check
    if line_bytes.contains(&b'\t') {
        errors.push(
            ValidationError::new("control_character", "2", Severity::Error)
                .message("Tab character (0x09) found in value")
                .file(file_name)
                .line(line_num),
        );
    }

    // Bare CR within line
    if line_trimmed.contains('\r') {
        errors.push(
            ValidationError::new("control_character", "2", Severity::Error)
                .message("Bare carriage return (CR) found within value")
                .file(file_name)
                .line(line_num),
        );
    }

    // Other control characters (< 0x20 excluding \t, \r, \n)
    for &b in line_bytes {
        if b < 0x20 && b != b'\t' && b != b'\r' && b != b'\n' {
            errors.push(
                ValidationError::new("control_character", "2", Severity::Error)
                    .message(format!("Control character (0x{b:02X}) found in value"))
                    .file(file_name)
                    .line(line_num),
            );
            break;
        }
    }

    // Regex-based content checks (HTML tags, comments, escape sequences)
    if let Some(m) = HTML_TAG_RE.find(line_trimmed) {
        errors.push(
            ValidationError::new("forbidden_content", "2", Severity::Error)
                .message("HTML tag found in value")
                .file(file_name)
                .line(line_num)
                .value(m.as_str().to_string()),
        );
    }

    if let Some(m) = HTML_COMMENT_RE.find(line_trimmed) {
        errors.push(
            ValidationError::new("forbidden_content", "2", Severity::Error)
                .message("HTML comment found in value")
                .file(file_name)
                .line(line_num)
                .value(m.as_str().to_string()),
        );
    }

    if let Some(m) = LITERAL_ESCAPE_RE.find(line_trimmed) {
        errors.push(
            ValidationError::new("forbidden_content", "2", Severity::Error)
                .message("Literal escape sequence found in value")
                .file(file_name)
                .line(line_num)
                .value(m.as_str().to_string()),
        );
    }
}
