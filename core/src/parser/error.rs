use std::fmt;
use std::path::PathBuf;

use thiserror::Error;

use crate::parser::feed_source::GtfsFiles;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("GTFS file not found in feed: {0}")]
    GtfsFileNotFound(GtfsFiles),

    #[error("Failed to read zip archive: {0}")]
    ZipExtraction(#[from] zip::result::ZipError),

    #[error("Expected .zip archive or directory, got file: {}", .0.display())]
    NotAGtfsFeed(PathBuf),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV error in {file}: {source}")]
    Csv { file: String, source: csv::Error },
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub file_name: String,
    pub line_number: usize,
    pub field_name: String,
    pub value: String,
    pub kind: ParseErrorKind,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: field '{}' (value '{}'): {}",
            self.file_name, self.line_number, self.field_name, self.value, self.kind
        )
    }
}

#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    InvalidInteger,
    InvalidFloat,
    InvalidDate,
    InvalidTime,
    InvalidEnum,
    MissingRequired,
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInteger => write!(f, "invalid integer"),
            Self::InvalidFloat => write!(f, "invalid float"),
            Self::InvalidDate => write!(f, "invalid date"),
            Self::InvalidTime => write!(f, "invalid time"),
            Self::InvalidEnum => write!(f, "invalid enum value"),
            Self::MissingRequired => write!(f, "missing required field"),
        }
    }
}
