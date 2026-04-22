//! `gapline read` — query records from a single GTFS file.

use std::path::Path;
use std::process;
use std::sync::Arc;

use gapline_core::config::Config;

use super::super::exit;
use super::super::output::render_read_results;
use super::super::parser::{CrudTarget, OutputFormat};
use super::{
    load_dataset_or_exit, parse_query_or_exit, resolve_feed, resolve_format, resolve_output,
    warn_parse_errors,
};

pub fn run_read(
    config: &Arc<Config>,
    feed: Option<&Path>,
    where_query: Option<&String>,
    target: CrudTarget,
    format: Option<OutputFormat>,
    output: Option<&Path>,
) {
    let feed = resolve_feed(feed, config);
    let fmt = resolve_format(format, config);
    let output = resolve_output(output, config);

    let (ds, parse_errors) = load_dataset_or_exit(&feed);
    warn_parse_errors(&parse_errors);

    let query = where_query.map(|q| parse_query_or_exit(q));

    let result = match ds.read(target.to_target(), query.as_ref()) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("{e}");
            process::exit(exit::COMMAND_FAILED);
        }
    };

    if let Err(e) = render_read_results(&result, fmt, output.as_deref()) {
        tracing::error!("Error while rendering results: {e}");
        process::exit(exit::COMMAND_FAILED);
    }
}
