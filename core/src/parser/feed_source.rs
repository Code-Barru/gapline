use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Read, Write};
use std::path::{Path, PathBuf};

use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::parser::error::ParserError;

/// Abstraction over a loaded GTFS feed, whether from a ZIP archive or a directory.
///
/// `FeedSource` provides uniform access to the raw file contents of a GTFS feed.
/// It does **not** parse CSV data - it only provides file names and byte-level readers
/// for downstream stages (structural validation, then CSV parsing).
///
/// Only files matching a known [`GtfsFiles`] variant are indexed for data access.
/// However, **all** original entry names (including unknown files and subdirectory
/// prefixes) are preserved in `raw_entry_names` for structural validation rules
/// (e.g. `unknown_file`, `invalid_input_files_in_subfolder`).
///
/// The `Zip` variant stores only metadata (file path + entry index). File contents
/// are decompressed on demand via [`read_file`](Self::read_file), so opening a ZIP
/// is nearly instant regardless of its size.
#[derive(Debug)]
pub enum FeedSource {
    /// A feed backed by a ZIP archive on disk. Contents are read on demand.
    Zip {
        /// Path to the `.zip` file.
        path: PathBuf,
        /// Maps each recognized GTFS file to its entry name inside the archive.
        index: HashMap<GtfsFiles, String>,
        /// Original entry names from the archive (before prefix normalization,
        /// including unknown files, excluding directory entries).
        raw_entry_names: Vec<String>,
        geojson_bytes: Option<Vec<u8>>,
    },
    /// An in-memory feed for testing. Behaves like a ZIP but needs no file on disk.
    #[doc(hidden)]
    InMemory {
        /// File contents keyed by GTFS file type.
        files: HashMap<GtfsFiles, Vec<u8>>,
        /// Raw entry names for structural validation.
        raw_entry_names: Vec<String>,
        geojson_bytes: Option<Vec<u8>>,
    },
    /// A feed loaded from a directory on disk.
    Directory {
        /// Path to the directory.
        path: PathBuf,
        /// GTFS file types found at the root of the directory.
        file_names: Vec<GtfsFiles>,
        /// All file and directory entry names found at any level (relative to root).
        raw_entry_names: Vec<String>,
        geojson_bytes: Option<Vec<u8>>,
    },
}

impl FeedSource {
    /// Returns the list of known GTFS files present in the feed.
    ///
    /// For ZIP feeds, these are the files that matched a [`GtfsFiles`] variant.
    /// For directory feeds, these are the `.txt` files at the root level that
    /// matched a known variant.
    #[must_use]
    pub fn file_names(&self) -> Vec<GtfsFiles> {
        match self {
            FeedSource::Zip { index, .. } => index.keys().copied().collect(),
            FeedSource::InMemory { files, .. } => files.keys().copied().collect(),
            FeedSource::Directory { file_names, .. } => file_names.clone(),
        }
    }

    /// Returns all original entry names found in the feed source.
    ///
    /// For ZIP feeds, these are the raw archive entry names before prefix
    /// normalization, including unknown files (but excluding directory entries).
    /// For directory feeds, these are all file names found at any level,
    /// relative to the root directory.
    ///
    /// This is used by structural validation rules such as `unknown_file` and
    /// `invalid_input_files_in_subfolder`.
    #[must_use]
    pub fn raw_entry_names(&self) -> &[String] {
        match self {
            FeedSource::Zip {
                raw_entry_names, ..
            }
            | FeedSource::InMemory {
                raw_entry_names, ..
            }
            | FeedSource::Directory {
                raw_entry_names, ..
            } => raw_entry_names,
        }
    }

    /// Returns a buffered reader for the given GTFS file within the feed.
    ///
    /// For ZIP feeds the entry is decompressed on demand - only the requested
    /// file is read from the archive.
    ///
    /// # Errors
    ///
    /// Returns [`ParserError::GtfsFileNotFound`] if the file is not present in the feed.
    /// Returns [`ParserError::Io`] if the file cannot be read from disk.
    pub fn read_file(&self, name: GtfsFiles) -> Result<Box<dyn BufRead + '_>, ParserError> {
        match self {
            FeedSource::Zip { path, index, .. } => {
                let bytes = read_zip_entry(path, index, name)?;
                Ok(Box::new(BufReader::new(Cursor::new(bytes))))
            }
            FeedSource::InMemory { files, .. } => {
                let bytes = files
                    .get(&name)
                    .ok_or(ParserError::GtfsFileNotFound(name))?;
                Ok(Box::new(BufReader::new(Cursor::new(bytes.clone()))))
            }
            FeedSource::Directory {
                path, file_names, ..
            } => {
                if !file_names.contains(&name) {
                    return Err(ParserError::GtfsFileNotFound(name));
                }
                let file = File::open(path.join(name.to_string()))?;
                Ok(Box::new(BufReader::new(file)))
            }
        }
    }

    /// Returns the raw bytes for the given GTFS file.
    ///
    /// # Errors
    ///
    /// Returns [`ParserError::GtfsFileNotFound`] if the file is not present.
    /// Returns [`ParserError::Io`] if the file cannot be read from disk.
    pub fn read_file_bytes(&self, name: GtfsFiles) -> Result<Cow<'_, [u8]>, ParserError> {
        match self {
            FeedSource::Zip { path, index, .. } => {
                let bytes = read_zip_entry(path, index, name)?;
                Ok(Cow::Owned(bytes))
            }
            FeedSource::InMemory { files, .. } => {
                let bytes = files
                    .get(&name)
                    .ok_or(ParserError::GtfsFileNotFound(name))?;
                Ok(Cow::Borrowed(bytes))
            }
            FeedSource::Directory {
                path, file_names, ..
            } => {
                if !file_names.contains(&name) {
                    return Err(ParserError::GtfsFileNotFound(name));
                }
                let mut buf = Vec::new();
                File::open(path.join(name.to_string()))?.read_to_end(&mut buf)?;
                Ok(Cow::Owned(buf))
            }
        }
    }

    /// Returns the raw bytes of `locations.geojson`, or `None` if absent.
    #[must_use]
    pub fn read_geojson_locations(&self) -> Option<&[u8]> {
        match self {
            FeedSource::Zip { geojson_bytes, .. }
            | FeedSource::InMemory { geojson_bytes, .. }
            | FeedSource::Directory { geojson_bytes, .. } => geojson_bytes.as_deref(),
        }
    }

    /// Copies all ZIP entries except `exclude` into the given ZIP writer.
    ///
    /// Unmodified files are decompressed from the source archive and written
    /// into the output archive. This avoids re-serializing records that haven't
    /// changed.
    ///
    /// No-op for directory sources.
    ///
    /// # Errors
    ///
    /// Returns [`ParserError`] on I/O or ZIP failures.
    pub fn copy_zip_entries_except(
        &self,
        exclude: GtfsFiles,
        writer: &mut ZipWriter<File>,
        opts: SimpleFileOptions,
    ) -> Result<(), ParserError> {
        let FeedSource::Zip { path, index, .. } = self else {
            return Ok(());
        };

        let file = File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for (&gtfs_file, entry_name) in index {
            if gtfs_file == exclude {
                continue;
            }
            let mut entry = archive.by_name(entry_name)?;
            let capacity = usize::try_from(entry.size()).unwrap_or(0);
            let mut buf = Vec::with_capacity(capacity);
            entry.read_to_end(&mut buf)?;

            writer.start_file(gtfs_file.to_string(), opts)?;
            writer.write_all(&buf)?;
        }

        Ok(())
    }

    /// Copies all ZIP entries except those in `exclude` into `writer`.
    ///
    /// # Errors
    ///
    /// Returns [`ParserError`] on I/O or ZIP failures.
    pub fn copy_zip_entries_except_set(
        &self,
        exclude: &HashSet<GtfsFiles>,
        writer: &mut ZipWriter<File>,
        opts: SimpleFileOptions,
    ) -> Result<(), ParserError> {
        let FeedSource::Zip { path, index, .. } = self else {
            return Ok(());
        };

        let file = File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for (&gtfs_file, entry_name) in index {
            if exclude.contains(&gtfs_file) {
                continue;
            }
            let mut entry = archive.by_name(entry_name)?;
            let capacity = usize::try_from(entry.size()).unwrap_or(0);
            let mut buf = Vec::with_capacity(capacity);
            entry.read_to_end(&mut buf)?;

            writer.start_file(gtfs_file.to_string(), opts)?;
            writer.write_all(&buf)?;
        }

        Ok(())
    }

    /// Loads all ZIP entries into memory in a single pass.
    ///
    /// Converts a `Zip` source into `InMemory` so that subsequent parallel
    /// `read_file` calls don't each re-open the archive. This is the fast path
    /// for commands that need every file (e.g. `validate`, `read`).
    ///
    /// No-op for `Directory` and `InMemory` variants.
    ///
    /// # Errors
    ///
    /// Returns [`ParserError`] on I/O or ZIP failures.
    pub fn preload(&mut self) -> Result<(), ParserError> {
        // Only act on Zip; Directory and InMemory are already efficient.
        let FeedSource::Zip {
            path,
            index,
            raw_entry_names,
            geojson_bytes,
        } = self
        else {
            return Ok(());
        };

        let file = File::open(&*path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let mut files = HashMap::with_capacity(index.len());
        for (&gtfs_file, entry_name) in index.iter() {
            let mut entry = archive.by_name(entry_name)?;
            let capacity = usize::try_from(entry.size()).unwrap_or(0);
            let mut buf = Vec::with_capacity(capacity);
            entry.read_to_end(&mut buf)?;
            files.insert(gtfs_file, buf);
        }

        *self = FeedSource::InMemory {
            files,
            raw_entry_names: std::mem::take(raw_entry_names),
            geojson_bytes: geojson_bytes.take(),
        };

        Ok(())
    }
}

/// Decompresses a single entry from a ZIP archive.
fn read_zip_entry(
    path: &Path,
    index: &HashMap<GtfsFiles, String>,
    name: GtfsFiles,
) -> Result<Vec<u8>, ParserError> {
    let entry_name = index
        .get(&name)
        .ok_or(ParserError::GtfsFileNotFound(name))?;

    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut entry = archive.by_name(entry_name)?;

    let capacity = usize::try_from(entry.size()).unwrap_or(0);
    let mut buf = Vec::with_capacity(capacity);
    entry.read_to_end(&mut buf)?;
    Ok(buf)
}

/// Known GTFS Schedule file types.
///
/// Each variant corresponds to a file defined in the
/// [GTFS Schedule Reference](https://gtfs.org/documentation/schedule/reference/#dataset-files).
///
/// `locations.geojson` is intentionally excluded (not a `.txt` CSV file).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GtfsFiles {
    /// `agency.txt`
    Agency,
    /// `stops.txt`
    Stops,
    /// `routes.txt`
    Routes,
    /// `trips.txt`
    Trips,
    /// `stop_times.txt`
    StopTimes,
    /// `calendar.txt`
    Calendar,
    /// `calendar_dates.txt`
    CalendarDates,
    /// `fare_attributes.txt`
    FareAttributes,
    /// `fare_rules.txt`
    FareRules,
    /// `timeframes.txt`
    Timeframes,
    /// `rider_categories.txt`
    RiderCategories,
    /// `fare_media.txt`
    FareMedia,
    /// `fare_products.txt`
    FareProducts,
    /// `fare_leg_rules.txt`
    FareLegRules,
    /// `fare_leg_join_rules.txt`
    FareLegJoinRules,
    /// `fare_transfer_rules.txt`
    FareTransferRules,
    /// `areas.txt`
    Areas,
    /// `stop_areas.txt`
    StopAreas,
    /// `networks.txt`
    Networks,
    /// `route_networks.txt`
    RouteNetworks,
    /// `shapes.txt`
    Shapes,
    /// `frequencies.txt`
    Frequencies,
    /// `transfers.txt`
    Transfers,
    /// `pathways.txt`
    Pathways,
    /// `levels.txt`
    Levels,
    /// `location_groups.txt`
    LocationGroups,
    /// `location_group_stops.txt`
    LocationGroupStops,
    /// `booking_rules.txt`
    BookingRules,
    /// `translations.txt`
    Translations,
    /// `feed_info.txt`
    FeedInfo,
    /// `attributions.txt`
    Attributions,
}

impl GtfsFiles {
    /// Returns the recognized column names for this GTFS file according to the
    /// [GTFS Schedule Reference](https://gtfs.org/documentation/schedule/reference/).
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn expected_columns(self) -> &'static [&'static str] {
        match self {
            Self::Agency => &[
                "agency_id",
                "agency_name",
                "agency_url",
                "agency_timezone",
                "agency_lang",
                "agency_phone",
                "agency_fare_url",
                "agency_email",
            ],
            Self::Stops => &[
                "stop_id",
                "stop_code",
                "stop_name",
                "tts_stop_name",
                "stop_desc",
                "stop_lat",
                "stop_lon",
                "zone_id",
                "stop_url",
                "location_type",
                "parent_station",
                "stop_timezone",
                "wheelchair_boarding",
                "level_id",
                "platform_code",
            ],
            Self::Routes => &[
                "route_id",
                "agency_id",
                "route_short_name",
                "route_long_name",
                "route_desc",
                "route_type",
                "route_url",
                "route_color",
                "route_text_color",
                "route_sort_order",
                "continuous_pickup",
                "continuous_drop_off",
                "network_id",
            ],
            Self::Trips => &[
                "route_id",
                "service_id",
                "trip_id",
                "trip_headsign",
                "trip_short_name",
                "direction_id",
                "block_id",
                "shape_id",
                "wheelchair_accessible",
                "bikes_allowed",
            ],
            Self::StopTimes => &[
                "trip_id",
                "arrival_time",
                "departure_time",
                "stop_id",
                "stop_sequence",
                "stop_headsign",
                "pickup_type",
                "drop_off_type",
                "continuous_pickup",
                "continuous_drop_off",
                "shape_dist_traveled",
                "timepoint",
                "start_pickup_drop_off_window",
                "end_pickup_drop_off_window",
                "mean_duration_factor",
                "mean_duration_offset",
                "safe_duration_factor",
                "safe_duration_offset",
            ],
            Self::Calendar => &[
                "service_id",
                "monday",
                "tuesday",
                "wednesday",
                "thursday",
                "friday",
                "saturday",
                "sunday",
                "start_date",
                "end_date",
            ],
            Self::CalendarDates => &["service_id", "date", "exception_type"],
            Self::FareAttributes => &[
                "fare_id",
                "price",
                "currency_type",
                "payment_method",
                "transfers",
                "agency_id",
                "transfer_duration",
            ],
            Self::FareRules => &[
                "fare_id",
                "route_id",
                "origin_id",
                "destination_id",
                "contains_id",
            ],
            Self::Timeframes => &["timeframe_group_id", "start_time", "end_time", "service_id"],
            Self::RiderCategories => &[
                "rider_category_id",
                "rider_category_name",
                "min_age",
                "max_age",
                "eligibility_url",
            ],
            Self::FareMedia => &["fare_media_id", "fare_media_name", "fare_media_type"],
            Self::FareProducts => &[
                "fare_product_id",
                "fare_product_name",
                "fare_media_id",
                "amount",
                "currency",
                "rider_category_id",
            ],
            Self::FareLegRules => &[
                "leg_group_id",
                "network_id",
                "from_area_id",
                "to_area_id",
                "from_timeframe_group_id",
                "to_timeframe_group_id",
                "fare_product_id",
                "rule_priority",
            ],
            Self::FareLegJoinRules => &[
                "from_network_id",
                "to_network_id",
                "from_stop_id",
                "to_stop_id",
            ],
            Self::FareTransferRules => &[
                "from_leg_group_id",
                "to_leg_group_id",
                "transfer_count",
                "duration_limit",
                "duration_limit_type",
                "fare_transfer_type",
                "fare_product_id",
            ],
            Self::Areas => &["area_id", "area_name"],
            Self::StopAreas => &["area_id", "stop_id"],
            Self::Networks => &["network_id", "network_name"],
            Self::RouteNetworks => &["network_id", "route_id"],
            Self::Shapes => &[
                "shape_id",
                "shape_pt_lat",
                "shape_pt_lon",
                "shape_pt_sequence",
                "shape_dist_traveled",
            ],
            Self::Frequencies => &[
                "trip_id",
                "start_time",
                "end_time",
                "headway_secs",
                "exact_times",
            ],
            Self::Transfers => &[
                "from_stop_id",
                "to_stop_id",
                "from_route_id",
                "to_route_id",
                "from_trip_id",
                "to_trip_id",
                "transfer_type",
                "min_transfer_time",
            ],
            Self::Pathways => &[
                "pathway_id",
                "from_stop_id",
                "to_stop_id",
                "pathway_mode",
                "is_bidirectional",
                "length",
                "traversal_time",
                "stair_count",
                "max_slope",
                "min_width",
                "signposted_as",
                "reversed_signposted_as",
            ],
            Self::Levels => &["level_id", "level_index", "level_name"],
            Self::LocationGroups => &["location_group_id", "location_group_name"],
            Self::LocationGroupStops => &["location_group_id", "stop_id"],
            Self::BookingRules => &[
                "booking_rule_id",
                "booking_type",
                "prior_notice_duration_min",
                "prior_notice_duration_max",
                "prior_notice_last_day",
                "prior_notice_last_time",
                "prior_notice_start_day",
                "prior_notice_start_time",
                "prior_notice_service_id",
                "message",
                "pickup_message",
                "drop_off_message",
                "phone_number",
                "info_url",
                "booking_url",
            ],
            Self::Translations => &[
                "table_name",
                "field_name",
                "language",
                "translation",
                "record_id",
                "record_sub_id",
                "field_value",
            ],
            Self::FeedInfo => &[
                "feed_publisher_name",
                "feed_publisher_url",
                "feed_lang",
                "default_lang",
                "feed_start_date",
                "feed_end_date",
                "feed_version",
                "feed_contact_email",
                "feed_contact_url",
            ],
            Self::Attributions => &[
                "attribution_id",
                "agency_id",
                "route_id",
                "trip_id",
                "organization_name",
                "is_producer",
                "is_operator",
                "is_authority",
                "attribution_url",
                "attribution_email",
                "attribution_phone",
            ],
        }
    }

    /// Returns the **required** column names for this GTFS file according to the
    /// [GTFS Schedule Reference](https://gtfs.org/documentation/schedule/reference/).
    ///
    /// Conditionally required columns are not included here - they depend on
    /// context that structural validation cannot evaluate (e.g. number of agencies,
    /// `location_type` values). Those are validated in later sections.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn required_columns(self) -> &'static [&'static str] {
        match self {
            Self::Agency => &["agency_name", "agency_url", "agency_timezone"],
            Self::Stops => &["stop_id"],
            Self::Routes => &["route_id", "route_type"],
            Self::Trips => &["route_id", "service_id", "trip_id"],
            Self::StopTimes => &["trip_id", "stop_id", "stop_sequence"],
            Self::Calendar => &[
                "service_id",
                "monday",
                "tuesday",
                "wednesday",
                "thursday",
                "friday",
                "saturday",
                "sunday",
                "start_date",
                "end_date",
            ],
            Self::CalendarDates => &["service_id", "date", "exception_type"],
            Self::FareAttributes => &[
                "fare_id",
                "price",
                "currency_type",
                "payment_method",
                "transfers",
            ],
            Self::FareRules => &["fare_id"],
            Self::Timeframes => &["timeframe_group_id", "start_time", "end_time"],
            Self::RiderCategories => &["rider_category_id", "rider_category_name"],
            Self::FareMedia => &["fare_media_id", "fare_media_type"],
            Self::FareProducts => &["fare_product_id", "amount", "currency"],
            Self::FareLegRules => &["fare_product_id"],
            Self::FareLegJoinRules => &["from_network_id", "to_network_id"],
            Self::FareTransferRules => &["fare_transfer_type"],
            Self::Areas => &["area_id"],
            Self::StopAreas => &["area_id", "stop_id"],
            Self::Networks => &["network_id", "network_name"],
            Self::RouteNetworks => &["network_id", "route_id"],
            Self::Shapes => &[
                "shape_id",
                "shape_pt_lat",
                "shape_pt_lon",
                "shape_pt_sequence",
            ],
            Self::Frequencies => &["trip_id", "start_time", "end_time", "headway_secs"],
            Self::Transfers => &["from_stop_id", "to_stop_id"],
            Self::Pathways => &[
                "pathway_id",
                "from_stop_id",
                "to_stop_id",
                "pathway_mode",
                "is_bidirectional",
            ],
            Self::Levels => &["level_id", "level_index"],
            Self::LocationGroups => &["location_group_id"],
            Self::LocationGroupStops => &["location_group_id", "stop_id"],
            Self::BookingRules => &["booking_rule_id", "booking_type"],
            Self::Translations => &["table_name", "field_name", "language", "translation"],
            Self::FeedInfo => &["feed_publisher_name", "feed_publisher_url", "feed_lang"],
            Self::Attributions => &["organization_name"],
        }
    }
}

impl fmt::Display for GtfsFiles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Agency => "agency.txt",
            Self::Stops => "stops.txt",
            Self::Routes => "routes.txt",
            Self::Trips => "trips.txt",
            Self::StopTimes => "stop_times.txt",
            Self::Calendar => "calendar.txt",
            Self::CalendarDates => "calendar_dates.txt",
            Self::FareAttributes => "fare_attributes.txt",
            Self::FareRules => "fare_rules.txt",
            Self::Timeframes => "timeframes.txt",
            Self::RiderCategories => "rider_categories.txt",
            Self::FareMedia => "fare_media.txt",
            Self::FareProducts => "fare_products.txt",
            Self::FareLegRules => "fare_leg_rules.txt",
            Self::FareLegJoinRules => "fare_leg_join_rules.txt",
            Self::FareTransferRules => "fare_transfer_rules.txt",
            Self::Areas => "areas.txt",
            Self::StopAreas => "stop_areas.txt",
            Self::Networks => "networks.txt",
            Self::RouteNetworks => "route_networks.txt",
            Self::Shapes => "shapes.txt",
            Self::Frequencies => "frequencies.txt",
            Self::Transfers => "transfers.txt",
            Self::Pathways => "pathways.txt",
            Self::Levels => "levels.txt",
            Self::LocationGroups => "location_groups.txt",
            Self::LocationGroupStops => "location_group_stops.txt",
            Self::BookingRules => "booking_rules.txt",
            Self::Translations => "translations.txt",
            Self::FeedInfo => "feed_info.txt",
            Self::Attributions => "attributions.txt",
        };
        write!(f, "{name}")
    }
}

impl TryFrom<&str> for GtfsFiles {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "agency.txt" => Ok(Self::Agency),
            "stops.txt" => Ok(Self::Stops),
            "routes.txt" => Ok(Self::Routes),
            "trips.txt" => Ok(Self::Trips),
            "stop_times.txt" => Ok(Self::StopTimes),
            "calendar.txt" => Ok(Self::Calendar),
            "calendar_dates.txt" => Ok(Self::CalendarDates),
            "fare_attributes.txt" => Ok(Self::FareAttributes),
            "fare_rules.txt" => Ok(Self::FareRules),
            "timeframes.txt" => Ok(Self::Timeframes),
            "rider_categories.txt" => Ok(Self::RiderCategories),
            "fare_media.txt" => Ok(Self::FareMedia),
            "fare_products.txt" => Ok(Self::FareProducts),
            "fare_leg_rules.txt" => Ok(Self::FareLegRules),
            "fare_leg_join_rules.txt" => Ok(Self::FareLegJoinRules),
            "fare_transfer_rules.txt" => Ok(Self::FareTransferRules),
            "areas.txt" => Ok(Self::Areas),
            "stop_areas.txt" => Ok(Self::StopAreas),
            "networks.txt" => Ok(Self::Networks),
            "route_networks.txt" => Ok(Self::RouteNetworks),
            "shapes.txt" => Ok(Self::Shapes),
            "frequencies.txt" => Ok(Self::Frequencies),
            "transfers.txt" => Ok(Self::Transfers),
            "pathways.txt" => Ok(Self::Pathways),
            "levels.txt" => Ok(Self::Levels),
            "location_groups.txt" => Ok(Self::LocationGroups),
            "location_group_stops.txt" => Ok(Self::LocationGroupStops),
            "booking_rules.txt" => Ok(Self::BookingRules),
            "translations.txt" => Ok(Self::Translations),
            "feed_info.txt" => Ok(Self::FeedInfo),
            "attributions.txt" => Ok(Self::Attributions),
            _ => Err(()),
        }
    }
}
