use std::collections::HashMap;
use std::io::BufRead;

use csv::{Reader, StringRecord};

use crate::validation::utils::strip_bom_str;

/// A borrowed view into the current CSV row.
///
/// Provides O(1) field access by column name without any allocation.
/// The returned `&str` values borrow directly from the underlying
/// [`StringRecord`] buffer that is reused across rows.
pub struct CsvRow<'a> {
    column_index: &'a HashMap<String, usize>,
    record: &'a StringRecord,
}

impl<'a> CsvRow<'a> {
    /// Returns the value for the given column name, or `None` if the column
    /// does not exist or the value is empty.
    #[inline]
    #[must_use]
    pub fn get(&self, column: &str) -> Option<&'a str> {
        let &idx = self.column_index.get(column)?;
        self.record.get(idx).filter(|s| !s.is_empty())
    }
}

pub struct CsvRecords<R> {
    reader: Reader<R>,
    headers: Vec<String>,
    column_index: HashMap<String, usize>,
    record: StringRecord,
    line: usize,
}

impl<R: BufRead> CsvRecords<R> {
    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    /// Advances to the next row and calls `f` with the line number and a
    /// borrowed [`CsvRow`]. Returns `false` when there are no more rows.
    pub fn next_row(&mut self) -> Option<(usize, CsvRow<'_>)> {
        if self.reader.read_record(&mut self.record).ok()? {
            self.line += 1;
            let row = CsvRow {
                column_index: &self.column_index,
                record: &self.record,
            };
            Some((self.line, row))
        } else {
            None
        }
    }
}

/// # Errors
///
/// Returns `csv::Error` if headers cannot be read.
pub fn parse_csv<R: BufRead>(reader: R) -> Result<CsvRecords<R>, csv::Error> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .flexible(true)
        .quoting(true)
        .from_reader(reader);

    let headers: Vec<String> = csv_reader
        .headers()?
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let s = if i == 0 { strip_bom_str(h) } else { h };
            s.to_owned()
        })
        .collect();

    let column_index: HashMap<String, usize> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| (h.clone(), i))
        .collect();

    Ok(CsvRecords {
        reader: csv_reader,
        headers,
        column_index,
        record: StringRecord::new(),
        line: 1,
    })
}
