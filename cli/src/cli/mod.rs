//! Command-line interface for headway.
//!
//! This module handles all user-facing concerns: argument parsing via
//! [`clap`](https://docs.rs/clap), subcommand routing, output formatting, and
//! interactive prompts. It depends on [`headway_core`] for business logic but the
//! reverse dependency is never allowed.
//!
//! ## Re-exports
//!
//! - [`Cli`] -- Top-level clap parser struct.
//! - `Commands` -- Enum of all available subcommands.
//! - `OutputFormat` -- Supported output formats (JSON, CSV, XML, text).
//! - `CrudTarget` -- GTFS files that support CRUD operations.

mod output;
mod parser;

pub use output::render_report;
pub use parser::{Cli, Commands, CrudTarget, OutputFormat};
