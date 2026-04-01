//! Top-level validation entry point.
//!
//! Provides a single function that encapsulates the full validation pipeline:
//! feed loading → structural validation → parsing → field type validation → report.

use std::path::Path;
use std::sync::Arc;

use crate::config::Config;
use crate::parser::{FeedLoader, ParserError};
use crate::validation::ValidationReport;
use crate::validation::engine::ValidationEngine;

/// Runs the full validation pipeline on a GTFS feed.
///
/// 1. Opens the feed at `path`.
/// 2. Executes structural rules (sections 1-2) via [`ValidationEngine::validate`].
/// 3. If no blocking errors, parses the feed into a [`GtfsFeed`].
/// 4. Executes post-parsing rules (section 3+) via [`ValidationEngine::validate_feed`].
/// 5. Merges all findings into a single [`ValidationReport`].
///
/// # Errors
///
/// Returns [`ParserError`] if the feed cannot be opened.
pub fn validate(path: &Path, config: Arc<Config>) -> Result<ValidationReport, ParserError> {
    let source = FeedLoader::open(path)?;
    let engine = ValidationEngine::new(config);

    // Phase 1: structural validation (sections 1-2)
    let structural_report = engine.validate_structural(&source);

    // If structural validation found errors, stop here — the feed may not be parsable.
    if structural_report.has_errors() {
        return Ok(structural_report);
    }

    // Phase 2: parse the feed
    let (feed, parse_errors) = FeedLoader::load(&source);

    // Phase 3: field type validation (section 3+)
    let feed_report = engine.validate_feed(&feed, &parse_errors);

    // Merge both reports
    let mut all_errors: Vec<_> = structural_report.errors().to_vec();
    all_errors.extend(feed_report.errors().to_vec());

    Ok(ValidationReport::from(all_errors))
}
