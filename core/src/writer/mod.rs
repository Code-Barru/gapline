//! Feed writer — writes modified GTFS data back to disk.
//!
//! Two strategies are supported:
//! - **ZIP**: copies unmodified files from the source, re-serializes only the changed file.
//! - **Directory**: writes only the modified `.txt` file into the directory.
//!
//! Uses the [`Filterable`] trait to generically convert records into CSV rows.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::crud::query::Filterable;
use crate::crud::read::GtfsTarget;
use crate::models::GtfsFeed;
use crate::parser::feed_source::{FeedSource, GtfsFiles};

/// Errors that can occur while writing a feed.
#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

/// Writes a complete GTFS feed as a ZIP archive at the given path.
///
/// Only collections that contain at least one record are included.
///
/// # Errors
///
/// Returns [`WriteError`] if file creation, CSV serialization, or ZIP writing fails.
pub fn write_feed(feed: &GtfsFeed, path: &Path) -> Result<(), WriteError> {
    let file = File::create(path)?;
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default();

    if !feed.agencies.is_empty() {
        write_records_to_zip(&feed.agencies, "agency.txt", &mut zip, opts)?;
    }
    if !feed.stops.is_empty() {
        write_records_to_zip(&feed.stops, "stops.txt", &mut zip, opts)?;
    }
    if !feed.routes.is_empty() {
        write_records_to_zip(&feed.routes, "routes.txt", &mut zip, opts)?;
    }
    if !feed.trips.is_empty() {
        write_records_to_zip(&feed.trips, "trips.txt", &mut zip, opts)?;
    }
    if !feed.stop_times.is_empty() {
        write_records_to_zip(&feed.stop_times, "stop_times.txt", &mut zip, opts)?;
    }
    if !feed.calendars.is_empty() {
        write_records_to_zip(&feed.calendars, "calendar.txt", &mut zip, opts)?;
    }
    if !feed.calendar_dates.is_empty() {
        write_records_to_zip(&feed.calendar_dates, "calendar_dates.txt", &mut zip, opts)?;
    }
    if !feed.shapes.is_empty() {
        write_records_to_zip(&feed.shapes, "shapes.txt", &mut zip, opts)?;
    }
    if !feed.frequencies.is_empty() {
        write_records_to_zip(&feed.frequencies, "frequencies.txt", &mut zip, opts)?;
    }
    if !feed.transfers.is_empty() {
        write_records_to_zip(&feed.transfers, "transfers.txt", &mut zip, opts)?;
    }
    if !feed.pathways.is_empty() {
        write_records_to_zip(&feed.pathways, "pathways.txt", &mut zip, opts)?;
    }
    if !feed.levels.is_empty() {
        write_records_to_zip(&feed.levels, "levels.txt", &mut zip, opts)?;
    }
    if let Some(fi) = &feed.feed_info {
        write_records_to_zip(std::slice::from_ref(fi), "feed_info.txt", &mut zip, opts)?;
    }
    if !feed.fare_attributes.is_empty() {
        write_records_to_zip(&feed.fare_attributes, "fare_attributes.txt", &mut zip, opts)?;
    }
    if !feed.fare_rules.is_empty() {
        write_records_to_zip(&feed.fare_rules, "fare_rules.txt", &mut zip, opts)?;
    }
    if !feed.translations.is_empty() {
        write_records_to_zip(&feed.translations, "translations.txt", &mut zip, opts)?;
    }
    if !feed.attributions.is_empty() {
        write_records_to_zip(&feed.attributions, "attributions.txt", &mut zip, opts)?;
    }

    zip.finish()?;
    Ok(())
}

/// Writes a modified feed, re-serializing only the changed target file.
///
/// - **ZIP source**: copies all unmodified files from the source as raw bytes,
///   then writes the modified target from the in-memory feed. O(copy) instead of
///   O(parse + serialize) for unchanged files.
/// - **Directory source**: writes only the modified `.txt` file into the directory.
///
/// # Errors
///
/// Returns [`WriteError`] on I/O, CSV, or ZIP failures.
#[allow(clippy::too_many_lines)]
pub fn write_modified(
    feed: &GtfsFeed,
    source: &FeedSource,
    target: GtfsTarget,
    output: &Path,
) -> Result<(), WriteError> {
    match source {
        FeedSource::Directory { path, .. } => {
            let dir = if output.is_dir() {
                output
            } else {
                // If -o not given, main.rs passes the source dir
                path.as_path()
            };
            let file_path = dir.join(target.file_name());
            write_target_to_file(feed, target, &file_path)
        }
        FeedSource::Zip { files, .. } => {
            let changed_gtfs = target_to_gtfs_file(target);
            let out = File::create(output)?;
            let mut zip = ZipWriter::new(out);
            let opts = SimpleFileOptions::default();

            // Copy unmodified files as raw bytes (fast — no re-parsing)
            for (&gtfs_file, bytes) in files {
                if gtfs_file == changed_gtfs {
                    continue;
                }
                let name = gtfs_file.to_string();
                zip.start_file(&name, opts)?;
                zip.write_all(bytes)?;
            }

            // Write the modified target from the in-memory feed
            write_target_to_zip(feed, target, &mut zip, opts)?;

            zip.finish()?;
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Internal: write a single target's records
// ---------------------------------------------------------------------------

/// Writes one target file to a standalone `.txt` file on disk.
fn write_target_to_file(
    feed: &GtfsFeed,
    target: GtfsTarget,
    path: &Path,
) -> Result<(), WriteError> {
    let file = File::create(path)?;
    let mut w = std::io::BufWriter::new(file);
    match target {
        GtfsTarget::Agency => serialize_records(&feed.agencies, &mut w),
        GtfsTarget::Stops => serialize_records(&feed.stops, &mut w),
        GtfsTarget::Routes => serialize_records(&feed.routes, &mut w),
        GtfsTarget::Trips => serialize_records(&feed.trips, &mut w),
        GtfsTarget::StopTimes => serialize_records(&feed.stop_times, &mut w),
        GtfsTarget::Calendar => serialize_records(&feed.calendars, &mut w),
        GtfsTarget::CalendarDates => serialize_records(&feed.calendar_dates, &mut w),
        GtfsTarget::Shapes => serialize_records(&feed.shapes, &mut w),
        GtfsTarget::Frequencies => serialize_records(&feed.frequencies, &mut w),
        GtfsTarget::Transfers => serialize_records(&feed.transfers, &mut w),
        GtfsTarget::Pathways => serialize_records(&feed.pathways, &mut w),
        GtfsTarget::Levels => serialize_records(&feed.levels, &mut w),
        GtfsTarget::FeedInfo => {
            if let Some(fi) = &feed.feed_info {
                serialize_records(std::slice::from_ref(fi), &mut w)
            } else {
                Ok(())
            }
        }
        GtfsTarget::FareAttributes => serialize_records(&feed.fare_attributes, &mut w),
        GtfsTarget::FareRules => serialize_records(&feed.fare_rules, &mut w),
        GtfsTarget::Translations => serialize_records(&feed.translations, &mut w),
        GtfsTarget::Attributions => serialize_records(&feed.attributions, &mut w),
    }
}

/// Writes one target file into an open ZIP archive.
fn write_target_to_zip(
    feed: &GtfsFeed,
    target: GtfsTarget,
    zip: &mut ZipWriter<File>,
    opts: SimpleFileOptions,
) -> Result<(), WriteError> {
    let name = target.file_name();
    zip.start_file(name, opts)?;
    match target {
        GtfsTarget::Agency => serialize_records(&feed.agencies, zip),
        GtfsTarget::Stops => serialize_records(&feed.stops, zip),
        GtfsTarget::Routes => serialize_records(&feed.routes, zip),
        GtfsTarget::Trips => serialize_records(&feed.trips, zip),
        GtfsTarget::StopTimes => serialize_records(&feed.stop_times, zip),
        GtfsTarget::Calendar => serialize_records(&feed.calendars, zip),
        GtfsTarget::CalendarDates => serialize_records(&feed.calendar_dates, zip),
        GtfsTarget::Shapes => serialize_records(&feed.shapes, zip),
        GtfsTarget::Frequencies => serialize_records(&feed.frequencies, zip),
        GtfsTarget::Transfers => serialize_records(&feed.transfers, zip),
        GtfsTarget::Pathways => serialize_records(&feed.pathways, zip),
        GtfsTarget::Levels => serialize_records(&feed.levels, zip),
        GtfsTarget::FeedInfo => {
            if let Some(fi) = &feed.feed_info {
                serialize_records(std::slice::from_ref(fi), zip)
            } else {
                Ok(())
            }
        }
        GtfsTarget::FareAttributes => serialize_records(&feed.fare_attributes, zip),
        GtfsTarget::FareRules => serialize_records(&feed.fare_rules, zip),
        GtfsTarget::Translations => serialize_records(&feed.translations, zip),
        GtfsTarget::Attributions => serialize_records(&feed.attributions, zip),
    }
}

// ---------------------------------------------------------------------------
// Internal: generic CSV serialization
// ---------------------------------------------------------------------------

/// Serializes records as CSV into any writer (ZIP entry, file, buffer).
fn serialize_records<T: Filterable>(records: &[T], w: &mut impl Write) -> Result<(), WriteError> {
    let headers = T::valid_fields();

    // Header
    let header_line = headers.join(",");
    w.write_all(header_line.as_bytes())?;
    w.write_all(b"\n")?;

    // Rows
    for record in records {
        let mut first = true;
        for h in headers {
            if !first {
                w.write_all(b",")?;
            }
            first = false;
            let val = record.field_value(h).unwrap_or_default();
            if val.contains(',') || val.contains('"') || val.contains('\n') {
                w.write_all(b"\"")?;
                w.write_all(val.replace('"', "\"\"").as_bytes())?;
                w.write_all(b"\"")?;
            } else {
                w.write_all(val.as_bytes())?;
            }
        }
        w.write_all(b"\n")?;
    }

    Ok(())
}

/// Writes a single GTFS file (header + rows) into the ZIP archive.
/// Used by [`write_feed`] for full-feed writes.
fn write_records_to_zip<T: Filterable>(
    records: &[T],
    file_name: &str,
    zip: &mut ZipWriter<File>,
    opts: SimpleFileOptions,
) -> Result<(), WriteError> {
    zip.start_file(file_name, opts)?;
    serialize_records(records, zip)
}

fn target_to_gtfs_file(target: GtfsTarget) -> GtfsFiles {
    match target {
        GtfsTarget::Agency => GtfsFiles::Agency,
        GtfsTarget::Stops => GtfsFiles::Stops,
        GtfsTarget::Routes => GtfsFiles::Routes,
        GtfsTarget::Trips => GtfsFiles::Trips,
        GtfsTarget::StopTimes => GtfsFiles::StopTimes,
        GtfsTarget::Calendar => GtfsFiles::Calendar,
        GtfsTarget::CalendarDates => GtfsFiles::CalendarDates,
        GtfsTarget::Shapes => GtfsFiles::Shapes,
        GtfsTarget::Frequencies => GtfsFiles::Frequencies,
        GtfsTarget::Transfers => GtfsFiles::Transfers,
        GtfsTarget::Pathways => GtfsFiles::Pathways,
        GtfsTarget::Levels => GtfsFiles::Levels,
        GtfsTarget::FeedInfo => GtfsFiles::FeedInfo,
        GtfsTarget::FareAttributes => GtfsFiles::FareAttributes,
        GtfsTarget::FareRules => GtfsFiles::FareRules,
        GtfsTarget::Translations => GtfsFiles::Translations,
        GtfsTarget::Attributions => GtfsFiles::Attributions,
    }
}
