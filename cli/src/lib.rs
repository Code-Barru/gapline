//! # gapline (cli)
//!
//! Command-line interface layer for gapline.
//!
//! This crate provides the user-facing CLI (argument parsing, output formatting)
//! and depends on [`gapline_core`] for all business logic.

/// Command-line interface module.
pub mod cli;
/// HTTP feed download module.
pub mod http;
