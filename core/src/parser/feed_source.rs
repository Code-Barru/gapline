use std::collections::HashMap;
use std::fmt;
use std::io::{BufRead, BufReader, Cursor};
use std::path::PathBuf;

use crate::parser::error::ParserError;

/// Abstraction over a loaded GTFS feed, whether from a ZIP archive or a directory.
///
/// `FeedSource` provides uniform access to the raw file contents of a GTFS feed.
/// It does **not** parse CSV data — it only provides file names and byte-level readers
/// for downstream stages (structural validation, then CSV parsing).
///
/// Only files matching a known [`GtfsFiles`] variant are indexed for data access.
/// However, **all** original entry names (including unknown files and subdirectory
/// prefixes) are preserved in `raw_entry_names` for structural validation rules
/// (e.g. `unknown_file`, `invalid_input_files_in_subfolder`).
#[derive(Debug)]
pub enum FeedSource {
    /// A feed loaded from a ZIP archive. All file contents are held in memory.
    Zip {
        /// Map from GTFS file type to raw bytes.
        files: HashMap<GtfsFiles, Vec<u8>>,
        /// Original entry names from the archive (before prefix normalization,
        /// including unknown files, excluding directory entries).
        raw_entry_names: Vec<String>,
    },
    /// A feed loaded from a directory on disk.
    Directory {
        /// Path to the directory.
        path: PathBuf,
        /// GTFS file types found at the root of the directory.
        file_names: Vec<GtfsFiles>,
        /// All file and directory entry names found at any level (relative to root).
        raw_entry_names: Vec<String>,
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
            FeedSource::Zip { files, .. } => files.keys().copied().collect(),
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
            | FeedSource::Directory {
                raw_entry_names, ..
            } => raw_entry_names,
        }
    }

    /// Returns a buffered reader for the given GTFS file within the feed.
    ///
    /// # Errors
    ///
    /// Returns [`ParserError::GtfsFileNotFound`] if the file is not present in the feed.
    /// Returns [`ParserError::Io`] if the file cannot be read from disk (directory feeds).
    pub fn read_file(&self, name: GtfsFiles) -> Result<Box<dyn BufRead + '_>, ParserError> {
        match self {
            FeedSource::Zip { files, .. } => {
                let bytes = files
                    .get(&name)
                    .ok_or(ParserError::GtfsFileNotFound(name))?;
                Ok(Box::new(BufReader::new(Cursor::new(bytes))))
            }
            FeedSource::Directory {
                path, file_names, ..
            } => {
                if !file_names.contains(&name) {
                    return Err(ParserError::GtfsFileNotFound(name));
                }
                let file = std::fs::File::open(path.join(name.to_string()))?;
                Ok(Box::new(BufReader::new(file)))
            }
        }
    }
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
            Self::FareLegJoinRules => &["from_leg_group_id", "to_leg_group_id", "join_type"],
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
