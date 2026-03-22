use std::collections::HashMap;
use std::fmt;
use std::io::{BufRead, BufReader, Cursor};
use std::path::PathBuf;

use crate::core::parser::error::ParserError;

/// Abstraction over a loaded GTFS feed, whether from a ZIP archive or a directory.
///
/// `FeedSource` provides uniform access to the raw file contents of a GTFS feed.
/// It does **not** parse CSV data — it only provides file names and byte-level readers
/// for downstream stages (structural validation, then CSV parsing).
///
/// Only files matching a known [`GtfsFiles`] variant are indexed. Unknown files
/// (e.g. `custom_data.txt`) are silently ignored at load time.
#[derive(Debug)]
pub enum FeedSource {
    /// A feed loaded from a ZIP archive. All file contents are held in memory.
    Zip {
        /// Map from GTFS file type to raw bytes.
        files: HashMap<GtfsFiles, Vec<u8>>,
    },
    /// A feed loaded from a directory on disk.
    Directory {
        /// Path to the directory.
        path: PathBuf,
        /// GTFS file types found at the root of the directory.
        file_names: Vec<GtfsFiles>,
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
            FeedSource::Zip { files } => files.keys().copied().collect(),
            FeedSource::Directory { file_names, .. } => file_names.clone(),
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
            FeedSource::Zip { files } => {
                let bytes = files
                    .get(&name)
                    .ok_or(ParserError::GtfsFileNotFound(name))?;
                Ok(Box::new(BufReader::new(Cursor::new(bytes))))
            }
            FeedSource::Directory { path, file_names } => {
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
