//! GTFS feed loading from ZIP archives and directories.
//!
//! This module provides [`FeedLoader`] as the single entry point for opening
//! a GTFS feed, and [`FeedSource`] as the abstraction for accessing the raw
//! file contents (names + bytes) without parsing CSV data.

/// Error types for feed loading operations.
pub mod error;
/// Feed loader entry point.
pub mod feed_loader;
/// Feed source abstraction over ZIP and directory feeds.
pub mod feed_source;

pub use error::ParserError;
pub use feed_loader::FeedLoader;
pub use feed_source::{FeedSource, GtfsFiles};
