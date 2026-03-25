//! Binary entry point for headway
//!
//! This file is intentionally minimal. It parses CLI arguments via [`clap`] and
//! delegates all logic to the library crate.

use clap::Parser;
use headway::cli::{Cli, Commands};

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Validate {
            feed,
            format,
            output,
        } => {
            println!("{} {format:?} {output:?}", feed.display());
        }
        _ => {
            println!("Not implemented yet");
        }
    }
}
