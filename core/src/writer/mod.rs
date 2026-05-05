//! Feed writer - writes modified GTFS data back to disk.
//!
//! Two strategies are supported:
//! - **ZIP**: copies unmodified files from the source, re-serializes only the changed file.
//! - **Directory**: writes only the modified `.txt` file into the directory.
//!
//! Uses the [`Filterable`] trait to generically convert records into CSV rows.

use std::collections::HashSet;
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
    #[error("Cannot open source feed for writing: {0}")]
    Source(String),
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

/// Like [`write_feed`], but atomic: writes to a `.zip.tmp` then renames.
///
/// # Errors
///
/// Returns [`WriteError`] on I/O or serialization failure.
pub fn write_feed_atomic(feed: &GtfsFeed, path: &Path) -> Result<(), WriteError> {
    let mut tmp = path.to_path_buf();
    tmp.set_extension("zip.tmp");
    write_feed(feed, &tmp)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Writes a modified feed back to disk, re-serializing only the changed target.
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
        FeedSource::InMemory { .. } => write_feed(feed, output),
        FeedSource::Zip { path: src_path, .. } => {
            let changed_gtfs = target_to_gtfs_file(target);

            // When overwriting the source ZIP, write to a temp file first then
            // rename atomically. Otherwise File::create would truncate the
            // source before copy_zip_entries_except can read from it.
            let same_file = output == src_path;
            let actual_output = if same_file {
                let mut tmp = output.to_path_buf();
                tmp.set_extension("zip.tmp");
                tmp
            } else {
                output.to_path_buf()
            };

            let out = File::create(&actual_output)?;
            let mut zip = ZipWriter::new(out);
            let opts = SimpleFileOptions::default();

            // Copy unmodified files from the source ZIP (decompresses on demand)
            source
                .copy_zip_entries_except(changed_gtfs, &mut zip, opts)
                .map_err(|e| WriteError::Io(std::io::Error::other(e.to_string())))?;

            // Write the modified target from the in-memory feed
            write_target_to_zip(feed, target, &mut zip, opts)?;

            zip.finish()?;

            if same_file {
                std::fs::rename(&actual_output, output)?;
            }

            Ok(())
        }
    }
}

/// Writes multiple modified targets back to disk in a single pass.
///
/// # Errors
///
/// Returns [`WriteError`] on I/O, CSV, or ZIP failures.
pub fn write_modified_targets(
    feed: &GtfsFeed,
    source: &FeedSource,
    targets: &[GtfsTarget],
    output: &Path,
) -> Result<(), WriteError> {
    if targets.len() == 1 {
        return write_modified(feed, source, targets[0], output);
    }

    match source {
        FeedSource::Directory { path, .. } => {
            let dir = if output.is_dir() {
                output
            } else {
                path.as_path()
            };
            for &t in targets {
                write_target_to_file(feed, t, &dir.join(t.file_name()))?;
            }
            Ok(())
        }
        FeedSource::InMemory { .. } => write_feed(feed, output),
        FeedSource::Zip { path: src_path, .. } => {
            let exclude: HashSet<_> = targets.iter().map(|&t| target_to_gtfs_file(t)).collect();

            let same_file = output == src_path;
            let actual_output = if same_file {
                let mut tmp = output.to_path_buf();
                tmp.set_extension("zip.tmp");
                tmp
            } else {
                output.to_path_buf()
            };

            let out = File::create(&actual_output)?;
            let mut zip = ZipWriter::new(out);
            let opts = SimpleFileOptions::default();

            source
                .copy_zip_entries_except_set(&exclude, &mut zip, opts)
                .map_err(|e| WriteError::Io(std::io::Error::other(e.to_string())))?;

            for &t in targets {
                write_target_to_zip(feed, t, &mut zip, opts)?;
            }

            zip.finish()?;

            if same_file {
                std::fs::rename(&actual_output, output)?;
            }

            Ok(())
        }
    }
}

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
        GtfsTarget::FareMedia => serialize_records(&feed.fare_media, &mut w),
        GtfsTarget::FareProducts => serialize_records(&feed.fare_products, &mut w),
        GtfsTarget::FareLegRules => serialize_records(&feed.fare_leg_rules, &mut w),
        GtfsTarget::FareTransferRules => serialize_records(&feed.fare_transfer_rules, &mut w),
        GtfsTarget::RiderCategories => serialize_records(&feed.rider_categories, &mut w),
        GtfsTarget::Timeframes => serialize_records(&feed.timeframes, &mut w),
        GtfsTarget::Areas => serialize_records(&feed.areas, &mut w),
        GtfsTarget::StopAreas => serialize_records(&feed.stop_areas, &mut w),
        GtfsTarget::Networks => serialize_records(&feed.networks, &mut w),
        GtfsTarget::RouteNetworks => serialize_records(&feed.route_networks, &mut w),
        GtfsTarget::FareLegJoinRules => serialize_records(&feed.fare_leg_join_rules, &mut w),
    }
}

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
        GtfsTarget::FareMedia => serialize_records(&feed.fare_media, zip),
        GtfsTarget::FareProducts => serialize_records(&feed.fare_products, zip),
        GtfsTarget::FareLegRules => serialize_records(&feed.fare_leg_rules, zip),
        GtfsTarget::FareTransferRules => serialize_records(&feed.fare_transfer_rules, zip),
        GtfsTarget::RiderCategories => serialize_records(&feed.rider_categories, zip),
        GtfsTarget::Timeframes => serialize_records(&feed.timeframes, zip),
        GtfsTarget::Areas => serialize_records(&feed.areas, zip),
        GtfsTarget::StopAreas => serialize_records(&feed.stop_areas, zip),
        GtfsTarget::Networks => serialize_records(&feed.networks, zip),
        GtfsTarget::RouteNetworks => serialize_records(&feed.route_networks, zip),
        GtfsTarget::FareLegJoinRules => serialize_records(&feed.fare_leg_join_rules, zip),
    }
}

fn serialize_records<T: Filterable>(records: &[T], w: &mut impl Write) -> Result<(), WriteError> {
    let headers = T::valid_fields();
    let header_line = headers.join(",");
    w.write_all(header_line.as_bytes())?;
    w.write_all(b"\n")?;

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
        GtfsTarget::FareMedia => GtfsFiles::FareMedia,
        GtfsTarget::FareProducts => GtfsFiles::FareProducts,
        GtfsTarget::FareLegRules => GtfsFiles::FareLegRules,
        GtfsTarget::FareTransferRules => GtfsFiles::FareTransferRules,
        GtfsTarget::RiderCategories => GtfsFiles::RiderCategories,
        GtfsTarget::Timeframes => GtfsFiles::Timeframes,
        GtfsTarget::Areas => GtfsFiles::Areas,
        GtfsTarget::StopAreas => GtfsFiles::StopAreas,
        GtfsTarget::Networks => GtfsFiles::Networks,
        GtfsTarget::RouteNetworks => GtfsFiles::RouteNetworks,
        GtfsTarget::FareLegJoinRules => GtfsFiles::FareLegJoinRules,
    }
}
