use std::path::PathBuf;

use thiserror::Error;

use crate::parser::feed_source::GtfsFiles;

/// Errors that can occur when loading a GTFS feed from a ZIP archive or directory.
#[derive(Debug, Error)]
pub enum ParserError {
    /// The feed path does not exist on disk.
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// A requested GTFS file is not present in the loaded feed.
    #[error("GTFS file not found in feed: {0}")]
    GtfsFileNotFound(GtfsFiles),

    /// The ZIP archive is corrupted or unreadable.
    #[error("Failed to read zip archive: {0}")]
    ZipExtraction(#[from] zip::result::ZipError),

    /// The path is neither a `.zip` file nor a directory.
    #[error("Expected .zip archive or directory, got file: {}", .0.display())]
    NotAGtfsFeed(PathBuf),

    /// An I/O error occurred while reading the feed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
