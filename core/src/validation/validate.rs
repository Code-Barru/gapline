//! Top-level validation entry point.
//!
//! Provides a single function that encapsulates the full validation pipeline:
//! feed loading → structural validation → parsing → field type validation → report.

use std::io::IsTerminal;
use std::path::Path;
use std::sync::{Arc, LazyLock};

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

use crate::config::Config;
use crate::dataset::Dataset;
use crate::parser::{FeedLoader, ParserError};
use crate::validation::ValidationReport;
use crate::validation::engine::ValidationEngine;

static SPINNER_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::with_template("{spinner:.cyan} {msg}")
        .expect("hard-coded spinner template is valid")
        .tick_chars("⣷⣯⣟⡿⢿⣻⣽⣾ ")
});

/// Runs the full validation pipeline on a GTFS feed.
///
/// 1. Opens the feed at `path`.
/// 2. Executes structural rules (sections 1-2).
/// 3. If no blocking errors, parses the feed into a `GtfsFeed`.
/// 4. Executes post-parsing rules (section 3+).
/// 5. Merges all findings into a single [`ValidationReport`].
///
/// # Errors
///
/// Returns [`ParserError`] if the feed cannot be opened.
pub fn validate(path: &Path, config: Arc<Config>) -> Result<ValidationReport, ParserError> {
    let mut source = FeedLoader::open(path)?;
    source.preload()?;
    let show_progress = config.output.show_progress;
    let config_for_semantic = Arc::clone(&config);
    let engine = ValidationEngine::new(config);

    let structural_report = engine.validate_structural(&source);

    if structural_report.has_errors() {
        return Ok(structural_report);
    }

    let spinner = ProgressBar::new_spinner();
    if show_progress && std::io::stderr().is_terminal() {
        spinner.set_style(SPINNER_STYLE.clone());
        spinner.set_message("Loading feed...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    } else {
        spinner.set_draw_target(ProgressDrawTarget::hidden());
    }

    let (dataset, parse_errors) = Dataset::from_source(&source);
    spinner.finish_and_clear();

    let semantic_report = dataset.validate_semantic(&config_for_semantic, &parse_errors);

    let mut all_errors: Vec<_> = structural_report.errors().to_vec();
    all_errors.extend(semantic_report.errors().to_vec());

    Ok(ValidationReport::from(all_errors))
}
