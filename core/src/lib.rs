//! # headway-core
//!
//! Core business logic for headway — entirely CLI-agnostic, usable as a library.
//!
//! ## Modules
//!
//! - [`parser`] — GTFS feed loading (ZIP / directory), raw file access.
//! - [`validation`] — Validation engine: trait-based rules, errors, reports.
//!
//! ## Quick Example
//!
//! ```no_run
//! use headway_core::validation::{ValidationError, Severity};
//!
//! let error = ValidationError::new("missing_required_file", "1", Severity::Error)
//!     .message("Required file agency.txt is missing")
//!     .file("agency.txt");
//! ```

/// GTFS feed loading from ZIP archives and directories.
pub mod parser;

/// GTFS feed validation engine.
pub mod validation;
