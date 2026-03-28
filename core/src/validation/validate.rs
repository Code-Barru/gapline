//! Top-level validation entry point.
//!
//! Provides a single function that encapsulates the full structural validation
//! pipeline: feed loading → engine execution → report.

use std::path::Path;
use std::sync::Arc;

use crate::config::Config;
use crate::parser::{FeedLoader, ParserError};
use crate::validation::ValidationReport;
use crate::validation::engine::ValidationEngine;

/// Runs the full structural validation pipeline on a GTFS feed.
///
/// Opens the feed at `path`, executes all registered section 1 + 2 rules via
/// [`ValidationEngine`], and returns the aggregated [`ValidationReport`].
///
/// # Errors
///
/// Returns [`ParserError`] if the feed cannot be opened (file not found,
/// corrupted ZIP, not a GTFS feed, etc.).
///
/// # Example
///
/// ```no_run
/// use std::sync::Arc;
/// use headway_core::config::Config;
/// use headway_core::validation::validate;
///
/// let config = Arc::new(Config::default());
/// let report = validate(std::path::Path::new("feed.zip"), config).unwrap();
/// println!("{} errors", report.error_count());
/// ```
pub fn validate(path: &Path, config: Arc<Config>) -> Result<ValidationReport, ParserError> {
    let source = FeedLoader::open(path)?;
    let engine = ValidationEngine::new(config);
    Ok(engine.validate(&source))
}
