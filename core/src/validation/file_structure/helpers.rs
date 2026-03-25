//! Shared utilities for structural validation rules.

use std::io::BufRead;

use crate::parser::{FeedSource, GtfsFiles, ParserError};

/// Reads and returns the CSV header (first line) of a GTFS file as individual column names.
///
/// Strips a leading UTF-8 BOM if present. Returns `Ok(None)` if the file is
/// empty (0 bytes). Returns the raw column strings without trimming whitespace
/// (that is the responsibility of the `leading_or_trailing_whitespaces` rule).
///
/// # Errors
///
/// Propagates any [`ParserError`] from [`FeedSource::read_file`].
pub fn read_header(
    source: &FeedSource,
    file: GtfsFiles,
) -> Result<Option<Vec<String>>, ParserError> {
    let mut reader = source.read_file(file)?;
    let mut first_line = String::new();
    let bytes_read = reader.read_line(&mut first_line)?;

    if bytes_read == 0 {
        return Ok(None);
    }

    // Strip UTF-8 BOM if present.
    let line = first_line.strip_prefix('\u{FEFF}').unwrap_or(&first_line);

    // Strip trailing newline characters.
    let line = line.trim_end_matches(['\n', '\r']);

    let columns: Vec<String> = line.split(',').map(String::from).collect();
    Ok(Some(columns))
}
