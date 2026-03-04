//! Core business logic for headway.
//!
//! This module contains all domain logic and is entirely CLI-agnostic. It can be
//! used as a library independently of the command-line interface.
//!
//! ## Submodules
//!
//! - [`validation`] -- Validation engine with trait-based rules, error reporting,
//!   and summary reports.
//!
//! Future submodules (not yet implemented): `parser` (GTFS CSV/ZIP loading),
//! `model` (GTFS data structures with type-safe IDs), `integrity` (referential
//! integrity graph via petgraph), `crud` (create/read/update/delete operations),
//! and `config` (TOML configuration hierarchy).

/// GTFS feed validation engine.
pub mod validation;
