//! # Headway
//!
//! A high-performance, all-in-one CLI tool for manipulating and validating
//! [GTFS](https://gtfs.org/) (General Transit Feed Specification) files.
//!
//! Headway replaces the fragmented ecosystem of Java validators, Python libraries,
//! and ad-hoc scripts with a single, fast, local-first Rust binary
//!
//! ## Crate Structure
//!
//! - [`cli`] -- Command-line interface layer (argument parsing, output formatting,
//!   user interaction). Depends on [`core`] but never the reverse.
//! - [`core`] -- Business logic layer (validation engine, data model, future CRUD
//!   and configuration). 100% CLI-agnostic and usable as a library.
//!
//! ## Quick Example
//!
//! ```no_run
//! use headway::validation::{ValidationError, Severity};
//!
//! let error = ValidationError::new("missing_required_file", "1", Severity::Error)
//!     .message("Required file agency.txt is missing")
//!     .file("agency.txt");
//! ```

#![warn(missing_docs)]

/// Command-line interface layer.
pub mod cli;

/// Core business logic layer.
pub mod core;

/// Re-exported CLI entry point for argument parsing.
pub use cli::Cli;

/// Re-exported validation subsystem.
pub use core::validation;
