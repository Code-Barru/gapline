use thiserror::Error;

use crate::crud::query::{Filterable, Query, QueryError};
use crate::models::GtfsFeed;

/// Identifies which GTFS file to read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GtfsTarget {
    Agency,
    Stops,
    Routes,
    Trips,
    StopTimes,
    Calendar,
    CalendarDates,
    Shapes,
    Frequencies,
    Transfers,
    Pathways,
    Levels,
    FeedInfo,
    FareAttributes,
    FareRules,
    Translations,
    Attributions,
}

impl GtfsTarget {
    /// Returns the GTFS file name for this target (e.g. `"stops.txt"`).
    #[must_use]
    pub fn file_name(self) -> &'static str {
        match self {
            Self::Agency => "agency.txt",
            Self::Stops => "stops.txt",
            Self::Routes => "routes.txt",
            Self::Trips => "trips.txt",
            Self::StopTimes => "stop_times.txt",
            Self::Calendar => "calendar.txt",
            Self::CalendarDates => "calendar_dates.txt",
            Self::Shapes => "shapes.txt",
            Self::Frequencies => "frequencies.txt",
            Self::Transfers => "transfers.txt",
            Self::Pathways => "pathways.txt",
            Self::Levels => "levels.txt",
            Self::FeedInfo => "feed_info.txt",
            Self::FareAttributes => "fare_attributes.txt",
            Self::FareRules => "fare_rules.txt",
            Self::Translations => "translations.txt",
            Self::Attributions => "attributions.txt",
        }
    }
}

/// Result of a read operation: column headers + rows of field values.
#[derive(Debug)]
pub struct ReadResult {
    /// Column names (from `Filterable::valid_fields`).
    pub headers: Vec<&'static str>,
    /// One row per matching record; each cell is `None` when the field is unset.
    pub rows: Vec<Vec<Option<String>>>,
    /// GTFS file name (e.g. `"stops.txt"`).
    pub file_name: &'static str,
}

/// Errors that can occur during a read operation.
#[derive(Debug, Error)]
pub enum ReadError {
    #[error("{0}")]
    QueryError(#[from] QueryError),
}

/// Reads and optionally filters records from a GTFS feed.
///
/// # Errors
///
/// Returns [`ReadError::QueryError`] if the query references unknown fields for
/// the given target.
pub fn read_records(
    feed: &GtfsFeed,
    target: GtfsTarget,
    query: Option<&Query>,
) -> Result<ReadResult, ReadError> {
    let mut result = match target {
        GtfsTarget::Agency => collect_records(&feed.agencies, query)?,
        GtfsTarget::Stops => collect_records(&feed.stops, query)?,
        GtfsTarget::Routes => collect_records(&feed.routes, query)?,
        GtfsTarget::Trips => collect_records(&feed.trips, query)?,
        GtfsTarget::StopTimes => collect_records(&feed.stop_times, query)?,
        GtfsTarget::Calendar => collect_records(&feed.calendars, query)?,
        GtfsTarget::CalendarDates => collect_records(&feed.calendar_dates, query)?,
        GtfsTarget::Shapes => collect_records(&feed.shapes, query)?,
        GtfsTarget::Frequencies => collect_records(&feed.frequencies, query)?,
        GtfsTarget::Transfers => collect_records(&feed.transfers, query)?,
        GtfsTarget::Pathways => collect_records(&feed.pathways, query)?,
        GtfsTarget::Levels => collect_records(&feed.levels, query)?,
        GtfsTarget::FeedInfo => collect_records(feed.feed_info.as_slice(), query)?,
        GtfsTarget::FareAttributes => collect_records(&feed.fare_attributes, query)?,
        GtfsTarget::FareRules => collect_records(&feed.fare_rules, query)?,
        GtfsTarget::Translations => collect_records(&feed.translations, query)?,
        GtfsTarget::Attributions => collect_records(&feed.attributions, query)?,
    };
    result.file_name = target.file_name();

    Ok(result)
}

/// Collects records into rows of field values, applying an optional filter.
fn collect_records<T: Filterable>(
    records: &[T],
    query: Option<&Query>,
) -> Result<ReadResult, ReadError> {
    let headers = T::valid_fields();

    if let Some(q) = query {
        q.validate_fields::<T>()?;
    }

    let rows = records
        .iter()
        .filter(|r| query.is_none_or(|q| q.matches(*r)))
        .map(|r| headers.iter().map(|h| r.field_value(h)).collect())
        .collect();

    Ok(ReadResult {
        headers: headers.to_vec(),
        rows,
        file_name: "",
    })
}
