//! Binary entry point for headway.
//!
//! Parses CLI arguments via [`clap`] and delegates to the appropriate handler
//! in [`headway::cli::commands`].

use clap::Parser;

use headway::cli::{Cli, Commands, commands};

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Validate {
            feed,
            format,
            output,
        } => commands::run_validate(feed, *format, output.as_deref()),
        Commands::Read {
            feed,
            where_query,
            target,
            format,
            output,
        } => commands::run_read(
            feed,
            where_query.as_ref(),
            *target,
            *format,
            output.as_deref(),
        ),
        Commands::Create {
            feed,
            set,
            target,
            confirm,
            output,
        } => commands::run_create(feed, set, *target, *confirm, output.as_deref()),
        _ => {
            println!("Not implemented yet");
        }
    }
}
