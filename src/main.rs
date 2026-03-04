//! Binary entry point for headway.
//!
//! This file is intentionally minimal. It parses CLI arguments via [`clap`] and
//! delegates all logic to the [`headway`] library crate. Keeping the binary thin
//! ensures that all business logic is testable without running the CLI.

use clap::Parser;
use headway::Cli;

fn main() {
    let args = Cli::parse();

    println!("{args:?}");
    println!("Not implemented yet!");
}
