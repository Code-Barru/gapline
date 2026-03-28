//! Binary entry point for headway.
//!
//! Parses CLI arguments via [`clap`] and delegates to the appropriate handler.
//! The `validate` command runs the full structural validation pipeline via
//! [`headway_core::validation::validate`], then formats and outputs the report.

use std::process;
use std::sync::Arc;

use clap::Parser;

use headway::cli::{Cli, Commands, OutputFormat, render_report};
use headway_core::config::Config;

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
                eprintln!("{e}");
                process::exit(1);
            }

            if report.has_errors() {
                process::exit(1);
            }
        }
        _ => {
            println!("Not implemented yet");
        }
    }
}
