//! Binary entry point for headway.
//!
//! This file is intentionally minimal. It parses CLI arguments via [`clap`] and
//! delegates all logic to the library crate.

use clap::Parser;
use headway::cli::Cli;

fn main() {
    let args = Cli::parse();

    println!("{args:?}");
}
