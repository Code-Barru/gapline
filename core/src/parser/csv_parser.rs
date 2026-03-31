use std::collections::HashMap;
use std::io::BufRead;

use csv::{Reader, StringRecord};

const UTF8_BOM: &str = "\u{feff}";

pub struct CsvRecords<R> {
    reader: Reader<R>,
    headers: Vec<String>,
    record: StringRecord,
    line: usize,
}

impl<R: BufRead> CsvRecords<R> {
    pub fn headers(&self) -> &[String] {
        &self.headers
    }
}

impl<R: BufRead> Iterator for CsvRecords<R> {
    type Item = (usize, HashMap<String, String>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.read_record(&mut self.record).ok()? {
            self.line += 1;
            let row: HashMap<String, String> = self
                .headers
                .iter()
                .enumerate()
                .filter_map(|(i, col)| self.record.get(i).map(|v| (col.clone(), v.to_owned())))
                .collect();
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
            let s = if i == 0 {
                h.strip_prefix(UTF8_BOM).unwrap_or(h)
            } else {
                h
            };
            s.to_owned()
        })
        .collect();

    Ok(CsvRecords {
        reader: csv_reader,
        headers,
        record: StringRecord::new(),
        line: 1,
    })
}
