//! Binary entry point for headway.
//!
//! Parses CLI arguments via [`clap`] and delegates to the appropriate handler.
//! The `validate` command runs the full structural validation pipeline via
//! [`headway_core::validation::validate`], then formats and outputs the report.

use std::process;
use std::sync::Arc;

use clap::Parser;

use headway::cli::{Cli, Commands, OutputFormat, render_read_results, render_report};
use headway_core::config::Config;
use headway_core::parser::FeedLoader;

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Validate {
            feed,
            format,
            output,
        } => {
            let config = Arc::new(Config::default());
            let report = match headway_core::validation::validate(feed, config) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{e}");
                    process::exit(1);
                }
            };

            let fmt = format.unwrap_or(OutputFormat::Text);
            if let Err(e) = render_report(&report, fmt, output.as_deref()) {
                eprintln!("Error while rendering report: {e}");
                process::exit(1);
            }

            if report.has_errors() {
                process::exit(1);
            }
        }
        Commands::Read {
            feed,
            where_query,
            target,
            format,
            output,
        } => {
            let source = match FeedLoader::open(feed) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e}");
                    process::exit(1);
                }
            };
            let (feed_data, _parse_errors) = FeedLoader::load(&source);

            let query = match where_query {
                Some(q) => match headway_core::crud::query::parse(q) {
                    Ok(parsed) => Some(parsed),
                    Err(e) => {
                        eprintln!("Invalid query: {e}");
                        process::exit(1);
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
                    process::exit(1);
                }
            };

            let fmt = format.unwrap_or(OutputFormat::Text);
            if let Err(e) = render_read_results(&result, fmt, output.as_deref()) {
                eprintln!("Error while rendering results: {e}");
                process::exit(1);
            }
        }
        _ => {
            println!("Not implemented yet");
        }
    }
}
