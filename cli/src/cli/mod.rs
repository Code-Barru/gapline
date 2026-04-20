//! Command-line interface for gapline.
//!
//! This module handles all user-facing concerns: argument parsing via
//! [`clap`](https://docs.rs/clap), subcommand routing, output formatting, and
//! interactive prompts. It depends on [`gapline_core`] for business logic but the
//! reverse dependency is never allowed.
//!
//! ## Re-exports
//!
//! - [`Cli`] -- Top-level clap parser struct.
//! - `Commands` -- Enum of all available subcommands.
//! - `OutputFormat` -- Supported output formats (JSON, CSV, XML, text).
//! - `CrudTarget` -- GTFS files that support CRUD operations.

pub mod bootstrap;
pub mod commands;
mod completion_install;
pub mod exit;
mod output;
mod parser;
pub mod runner;

pub use completion_install::{InstallError, InstallReport, install_completion};
pub use output::{RuleEntry, Stage, render_read_results, render_report, render_rules_list};
pub use parser::{Cli, Commands, CrudTarget, OutputFormat, RulesCommand, SeverityArg};
