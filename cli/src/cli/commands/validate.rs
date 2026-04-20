//! `gapline validate` — runs the full validation pipeline.

use std::path::Path;
use std::process;
use std::sync::Arc;

use gapline_core::config::Config;

use super::super::exit;
use super::super::output::render_report;
use super::super::parser::OutputFormat;
use super::{resolve_feed, resolve_format, resolve_output};

pub fn run_validate(
    config: &Arc<Config>,
    feed: Option<&Path>,
    format: Option<OutputFormat>,
    output: Option<&Path>,
) {
    let feed = resolve_feed(feed, config);
    let fmt = resolve_format(format, config);
    let output = resolve_output(output, config);

    let report = match gapline_core::validation::validate(&feed, Arc::clone(config)) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            process::exit(exit::INPUT_ERROR);
        }
    };

    if let Err(e) = render_report(&report, fmt, &feed, output.as_deref(), config) {
        eprintln!("Error while rendering report: {e}");
        process::exit(exit::COMMAND_FAILED);
    }

    if report.has_errors() {
        process::exit(exit::COMMAND_FAILED);
    }
}
