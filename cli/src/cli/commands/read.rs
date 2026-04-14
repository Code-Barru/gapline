//! `headway read` — query records from a single GTFS file.

use std::path::Path;
use std::process;
use std::sync::Arc;

use headway_core::config::Config;
use headway_core::parser::FeedLoader;

use super::super::exit;
use super::super::output::render_read_results;
use super::super::parser::{CrudTarget, OutputFormat};
use super::{resolve_feed, resolve_format, resolve_output};

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

    let mut source = match FeedLoader::open(&feed) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            process::exit(exit::INPUT_ERROR);
        }
    };
    if let Err(e) = source.preload() {
        eprintln!("{e}");
        process::exit(exit::INPUT_ERROR);
    }
    let (feed_data, _parse_errors) = FeedLoader::load(&source);

    let query = match where_query {
        Some(q) => match headway_core::crud::query::parse(q) {
            Ok(parsed) => Some(parsed),
            Err(e) => {
                eprintln!("Invalid query: {e}");
                process::exit(exit::COMMAND_FAILED);
            }
        },
        None => None,
    };

    let result = match headway_core::crud::read::read_records(
        &feed_data,
        target.to_target(),
        query.as_ref(),
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            process::exit(exit::COMMAND_FAILED);
        }
    };

    if let Err(e) = render_read_results(&result, fmt, output.as_deref()) {
        eprintln!("Error while rendering results: {e}");
        process::exit(exit::COMMAND_FAILED);
    }
}
