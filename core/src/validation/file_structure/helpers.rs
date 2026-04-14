//! Shared utilities for structural validation rules.

use std::io::BufRead;

use crate::parser::{FeedSource, GtfsFiles, ParserError};
use crate::validation::utils::strip_bom_str;

/// Reads and returns the CSV header (first line) of a GTFS file as individual column names.
///
/// Strips a leading UTF-8 BOM if present. Returns an empty `Vec` if the file
/// is empty (0 bytes). Returns the raw column strings without trimming
/// whitespace (that is the responsibility of the `superfluous_whitespace` rule).
///
/// # Errors
///
/// Propagates any [`ParserError`] from [`FeedSource::read_file`].
pub fn read_header(source: &FeedSource, file: GtfsFiles) -> Result<Vec<String>, ParserError> {
    let mut reader = source.read_file(file)?;
    let mut first_line = String::new();
    let bytes_read = reader.read_line(&mut first_line)?;

    if bytes_read == 0 {
        return Ok(Vec::new());
    }

    let line = strip_bom_str(&first_line);
    let line = line.trim_end_matches(['\n', '\r']);

    Ok(line.split(',').map(String::from).collect())
}
