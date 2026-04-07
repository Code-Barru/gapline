//! # headway-core
//!
//! Core business logic for headway - entirely CLI-agnostic, usable as a library.
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

/// Application configuration.
pub mod config;

/// GTFS data model: IDs, types, enums, records, feed.
pub mod models;

/// GTFS feed loading from ZIP archives and directories.
pub mod parser;

/// GTFS feed validation engine.
pub mod validation;

/// Referential integrity indexes.
pub mod integrity;

/// Geodesic distance helpers.
pub mod geo;

/// CRUD operations on GTFS feeds.
pub mod crud;

/// Feed writer — serialize a [`models::GtfsFeed`] to a GTFS ZIP archive.
pub mod writer;
