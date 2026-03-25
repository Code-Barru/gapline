//! GTFS specification constants for structural validation.
//!
//! Defines which files are required or recommended according to the
//! [GTFS Schedule Reference](https://gtfs.org/documentation/schedule/reference/).
//!
//! Per-file column definitions live on [`GtfsFiles::expected_columns`](crate::parser::GtfsFiles::expected_columns).

use crate::parser::GtfsFiles;

/// Files that MUST always be present in a valid GTFS feed.
pub const REQUIRED_FILES: &[GtfsFiles; 4] = &[
    GtfsFiles::Agency,
    GtfsFiles::Routes,
    GtfsFiles::Trips,
    GtfsFiles::StopTimes,
];

/// Files that are RECOMMENDED but not required.
pub const RECOMMENDED_FILES: &[GtfsFiles; 2] = &[GtfsFiles::FeedInfo, GtfsFiles::Shapes];
