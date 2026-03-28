//! Rules `control_character` (CA7) and `forbidden_content` (CA8) —
//! rejects control characters, HTML tags, HTML comments, and literal escape
//! sequences in field values.

use std::io::Read;
use std::sync::LazyLock;

use regex::Regex;

use crate::parser::FeedSource;
use crate::validation::{Severity, StructuralValidationRule, ValidationError};

static HTML_TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<[a-zA-Z/][^>]*>").expect("invalid regex"));
static HTML_COMMENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<!--.*?-->").expect("invalid regex"));
/// Matches literal backslash followed by n, t, or r (the text `\n`, not the byte 0x0A).
static LITERAL_ESCAPE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\[ntr]").expect("invalid regex"));

/// Checks for control characters (CA7) and forbidden content (CA8).
pub struct InvalidContentRule;

impl StructuralValidationRule for InvalidContentRule {
    fn rule_id(&self) -> &'static str {
        "control_character"
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

            let data = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
                &bytes[3..]
            } else {
                &bytes
            };

            let Ok(content) = std::str::from_utf8(data) else {
                continue;
            };

            let file_name = file.to_string();

            // Process line by line for CA7 (control characters).
            // We work on raw bytes to detect control chars properly.
            let mut line_num: usize = 0;
            let mut in_quoted = false;

            for line in content.split('\n') {
                line_num += 1;
                let line_bytes = line.as_bytes();

                // Track quoted state across the line for tab detection context.
                // (Tabs are always invalid regardless of quoting context since delimiter is comma.)
                for &b in line_bytes {
                    match b {
                        b'"' => in_quoted = !in_quoted,
                        b'\t' => {
                            errors.push(
                                ValidationError::new(
                                    "control_character",
                                    self.section(),
                                    self.severity(),
                                )
                                .message("Tab character (0x09) found in value")
                                .file(file_name.clone())
                                .line(line_num),
                            );
                            // One error per line for tabs.
                            break;
                        }
                        _ => {}
                    }
                }

                // Check for bare CR within the line (not CRLF — the \n split already
                // consumed LF, so any remaining \r is either part of CRLF at end
                // or a bare CR in the middle).
                let line_trimmed = line.trim_end_matches('\r');
                if line_trimmed.contains('\r') {
                    errors.push(
                        ValidationError::new("control_character", self.section(), self.severity())
                            .message("Bare carriage return (CR) found within value")
                            .file(file_name.clone())
                            .line(line_num),
                    );
                }

                // Check for other control characters (< 0x20, excluding \t \r \n).
                for &b in line_bytes {
                    if b < 0x20 && b != b'\t' && b != b'\r' && b != b'\n' {
                        errors.push(
                            ValidationError::new(
                                "control_character",
                                self.section(),
                                self.severity(),
                            )
                            .message(format!("Control character (0x{b:02X}) found in value"))
                            .file(file_name.clone())
                            .line(line_num),
                        );
                        break; // One error per line.
                    }
                }

                // CA8 — Forbidden content: HTML tags.
                if let Some(m) = HTML_TAG_RE.find(line) {
                    errors.push(
                        ValidationError::new("forbidden_content", self.section(), self.severity())
                            .message("HTML tag found in value")
                            .file(file_name.clone())
                            .line(line_num)
                            .value(m.as_str().to_string()),
                    );
                }

                // CA8 — HTML comments.
                if let Some(m) = HTML_COMMENT_RE.find(line) {
                    errors.push(
                        ValidationError::new("forbidden_content", self.section(), self.severity())
                            .message("HTML comment found in value")
                            .file(file_name.clone())
                            .line(line_num)
                            .value(m.as_str().to_string()),
                    );
                }

                // CA8 — Literal escape sequences (\n, \t, \r as text).
                if let Some(m) = LITERAL_ESCAPE_RE.find(line) {
                    errors.push(
                        ValidationError::new("forbidden_content", self.section(), self.severity())
                            .message("Literal escape sequence found in value")
                            .file(file_name.clone())
                            .line(line_num)
                            .value(m.as_str().to_string()),
                    );
                }

                // Reset quoted state at line boundary (simplified — full quoting
                // is validated by the quoting rule).
                in_quoted = false;
            }
        }

        errors
    }
}
