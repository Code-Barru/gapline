/// Shared types and helpers for CRUD operations.
pub mod common;

/// Record creation for GTFS feeds.
pub mod create;

/// Record deletion for GTFS feeds.
pub mod delete;

/// Mini query language for filtering GTFS records.
pub mod query;

/// Read operations on GTFS feeds.
pub mod read;

/// Field-level mutation functions for GTFS records.
pub mod setters;

/// Record update for GTFS feeds.
pub mod update;
