use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::models::GtfsFeed;
use crate::parser::error::{ParseError, ParseErrorKind, ParserError};
use crate::parser::feed_source::{FeedSource, GtfsFiles};
use crate::parser::file_parsers;

const LOCATIONS_GEOJSON: &str = "locations.geojson";

pub struct FeedLoader;

fn parse_geojson_locations(
    source: &FeedSource,
) -> (Vec<crate::models::GeoJsonLocation>, Vec<ParseError>) {
    let Some(bytes) = source.read_geojson_locations() else {
        return (vec![], vec![]);
    };

    match file_parsers::locations_geojson::parse(bytes) {
        Ok((locations, feature_errors)) => {
            let errors = feature_errors
                .into_iter()
                .map(|e| ParseError {
                    file_name: LOCATIONS_GEOJSON.to_string(),
                    line_number: 0,
                    field_name: String::new(),
                    value: e.feature_index.map(|i| i.to_string()).unwrap_or_default(),
                    kind: ParseErrorKind::InvalidGeoJson(e.message),
                })
                .collect();
            (locations, errors)
        }
        Err(parser_err) => (
            vec![],
            vec![ParseError {
                file_name: LOCATIONS_GEOJSON.to_string(),
                line_number: 0,
                field_name: String::new(),
                value: String::new(),
                kind: ParseErrorKind::InvalidGeoJson(parser_err.to_string()),
            }],
        ),
    }
}

impl FeedLoader {
    /// # Errors
    ///
    /// Returns [`ParserError`] on missing path, corrupt ZIP, I/O failure, or
    /// unsupported file type.
    pub fn open(path: &Path) -> Result<FeedSource, ParserError> {
        if !path.exists() {
            return Err(ParserError::FileNotFound(path.to_path_buf()));
        }

        if path.is_file() {
            return Self::open_zip(path);
        }

        if path.is_dir() {
            return Self::open_directory(path);
        }

        Err(ParserError::NotAGtfsFeed(path.to_path_buf()))
    }

    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn load(source: &FeedSource) -> (GtfsFeed, Vec<ParseError>) {
        let available = source.file_names();
        let has = |f: GtfsFiles| available.contains(&f);

        macro_rules! parse_vec {
            ($file:expr, $parser:path) => {
                if has($file) {
                    match source.read_file($file) {
                        Ok(r) => $parser(r),
                        Err(_) => (vec![], vec![]),
                    }
                } else {
                    (vec![], vec![])
                }
            };
        }

        // Parse all files in parallel via nested rayon::join.
        // Each branch is independent - no shared mutable state.
        let (
            ((agencies_r, stops_r), (routes_r, trips_r)),
            ((stop_times_r, calendars_r), (cal_dates_r, shapes_r)),
        ) = rayon::join(
            || {
                rayon::join(
                    || {
                        rayon::join(
                            || parse_vec!(GtfsFiles::Agency, file_parsers::agency::parse),
                            || parse_vec!(GtfsFiles::Stops, file_parsers::stops::parse),
                        )
                    },
                    || {
                        rayon::join(
                            || parse_vec!(GtfsFiles::Routes, file_parsers::routes::parse),
                            || parse_vec!(GtfsFiles::Trips, file_parsers::trips::parse),
                        )
                    },
                )
            },
            || {
                rayon::join(
                    || {
                        rayon::join(
                            || parse_vec!(GtfsFiles::StopTimes, file_parsers::stop_times::parse),
                            || parse_vec!(GtfsFiles::Calendar, file_parsers::calendar::parse),
                        )
                    },
                    || {
                        rayon::join(
                            || {
                                parse_vec!(
                                    GtfsFiles::CalendarDates,
                                    file_parsers::calendar_dates::parse
                                )
                            },
                            || parse_vec!(GtfsFiles::Shapes, file_parsers::shapes::parse),
                        )
                    },
                )
            },
        );

        let (
            ((freqs_r, transfers_r), (pathways_r, levels_r)),
            ((fare_attr_r, fare_rules_r), (translations_r, attributions_r)),
        ) = rayon::join(
            || {
                rayon::join(
                    || {
                        rayon::join(
                            || parse_vec!(GtfsFiles::Frequencies, file_parsers::frequencies::parse),
                            || parse_vec!(GtfsFiles::Transfers, file_parsers::transfers::parse),
                        )
                    },
                    || {
                        rayon::join(
                            || parse_vec!(GtfsFiles::Pathways, file_parsers::pathways::parse),
                            || parse_vec!(GtfsFiles::Levels, file_parsers::levels::parse),
                        )
                    },
                )
            },
            || {
                rayon::join(
                    || {
                        rayon::join(
                            || {
                                parse_vec!(
                                    GtfsFiles::FareAttributes,
                                    file_parsers::fare_attributes::parse
                                )
                            },
                            || parse_vec!(GtfsFiles::FareRules, file_parsers::fare_rules::parse),
                        )
                    },
                    || {
                        rayon::join(
                            || {
                                parse_vec!(
                                    GtfsFiles::Translations,
                                    file_parsers::translations::parse
                                )
                            },
                            || {
                                parse_vec!(
                                    GtfsFiles::Attributions,
                                    file_parsers::attributions::parse
                                )
                            },
                        )
                    },
                )
            },
        );

        // feed_info is small - parse sequentially
        let feed_info_r = if has(GtfsFiles::FeedInfo) {
            match source.read_file(GtfsFiles::FeedInfo) {
                Ok(r) => file_parsers::feed_info::parse(r),
                Err(_) => (None, 0, vec![]),
            }
        } else {
            (None, 0, vec![])
        };

        let booking_rules_r =
            parse_vec!(GtfsFiles::BookingRules, file_parsers::booking_rules::parse);
        let location_groups_r = parse_vec!(
            GtfsFiles::LocationGroups,
            file_parsers::location_groups::parse
        );
        let location_group_stops_r = parse_vec!(
            GtfsFiles::LocationGroupStops,
            file_parsers::location_group_stops::parse
        );

        let fare_media_r = parse_vec!(GtfsFiles::FareMedia, file_parsers::fare_media::parse);
        let fare_products_r =
            parse_vec!(GtfsFiles::FareProducts, file_parsers::fare_products::parse);
        let fare_leg_rules_r =
            parse_vec!(GtfsFiles::FareLegRules, file_parsers::fare_leg_rules::parse);
        let fare_transfer_rules_r = parse_vec!(
            GtfsFiles::FareTransferRules,
            file_parsers::fare_transfer_rules::parse
        );
        let rider_categories_r = parse_vec!(
            GtfsFiles::RiderCategories,
            file_parsers::rider_categories::parse
        );
        let timeframes_r = parse_vec!(GtfsFiles::Timeframes, file_parsers::timeframes::parse);
        let areas_r = parse_vec!(GtfsFiles::Areas, file_parsers::areas::parse);
        let stop_areas_r = parse_vec!(GtfsFiles::StopAreas, file_parsers::stop_areas::parse);
        let networks_r = parse_vec!(GtfsFiles::Networks, file_parsers::networks::parse);
        let route_networks_r = parse_vec!(
            GtfsFiles::RouteNetworks,
            file_parsers::route_networks::parse
        );
        let fare_leg_join_rules_r = parse_vec!(
            GtfsFiles::FareLegJoinRules,
            file_parsers::fare_leg_join_rules::parse
        );

        let mut all_errors = Vec::new();

        macro_rules! unpack {
            ($result:expr, $errors:expr) => {{
                $errors.append(&mut $result.1);
                $result.0
            }};
        }

        let mut agencies_r = agencies_r;
        let mut stops_r = stops_r;
        let mut routes_r = routes_r;
        let mut trips_r = trips_r;
        let mut stop_times_r = stop_times_r;
        let mut calendars_r = calendars_r;
        let mut cal_dates_r = cal_dates_r;
        let mut shapes_r = shapes_r;
        let mut freqs_r = freqs_r;
        let mut transfers_r = transfers_r;
        let mut pathways_r = pathways_r;
        let mut levels_r = levels_r;
        let mut fare_attr_r = fare_attr_r;
        let mut fare_rules_r = fare_rules_r;
        let mut translations_r = translations_r;
        let mut attributions_r = attributions_r;
        let mut booking_rules_r = booking_rules_r;
        let mut location_groups_r = location_groups_r;
        let mut location_group_stops_r = location_group_stops_r;
        let mut fare_media_r = fare_media_r;
        let mut fare_products_r = fare_products_r;
        let mut fare_leg_rules_r = fare_leg_rules_r;
        let mut fare_transfer_rules_r = fare_transfer_rules_r;
        let mut rider_categories_r = rider_categories_r;
        let mut timeframes_r = timeframes_r;
        let mut areas_r = areas_r;
        let mut stop_areas_r = stop_areas_r;
        let mut networks_r = networks_r;
        let mut route_networks_r = route_networks_r;
        let mut fare_leg_join_rules_r = fare_leg_join_rules_r;
        let (feed_info, feed_info_line_count, mut feed_info_errors) = feed_info_r;
        all_errors.append(&mut feed_info_errors);

        let (geojson_locations, mut geojson_errors) = parse_geojson_locations(source);
        all_errors.append(&mut geojson_errors);

        let mut loaded_files: HashSet<String> = available
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        if source.read_geojson_locations().is_some() {
            loaded_files.insert(LOCATIONS_GEOJSON.to_string());
        }

        let feed = GtfsFeed {
            loaded_files,
            agencies: unpack!(agencies_r, all_errors),
            stops: unpack!(stops_r, all_errors),
            routes: unpack!(routes_r, all_errors),
            trips: unpack!(trips_r, all_errors),
            stop_times: unpack!(stop_times_r, all_errors),
            calendars: unpack!(calendars_r, all_errors),
            calendar_dates: unpack!(cal_dates_r, all_errors),
            shapes: unpack!(shapes_r, all_errors),
            frequencies: unpack!(freqs_r, all_errors),
            transfers: unpack!(transfers_r, all_errors),
            pathways: unpack!(pathways_r, all_errors),
            levels: unpack!(levels_r, all_errors),
            feed_info,
            feed_info_line_count,
            fare_attributes: unpack!(fare_attr_r, all_errors),
            fare_rules: unpack!(fare_rules_r, all_errors),
            translations: unpack!(translations_r, all_errors),
            attributions: unpack!(attributions_r, all_errors),
            booking_rules: unpack!(booking_rules_r, all_errors),
            location_groups: unpack!(location_groups_r, all_errors),
            location_group_stops: unpack!(location_group_stops_r, all_errors),
            fare_media: unpack!(fare_media_r, all_errors),
            fare_products: unpack!(fare_products_r, all_errors),
            fare_leg_rules: unpack!(fare_leg_rules_r, all_errors),
            fare_transfer_rules: unpack!(fare_transfer_rules_r, all_errors),
            rider_categories: unpack!(rider_categories_r, all_errors),
            timeframes: unpack!(timeframes_r, all_errors),
            areas: unpack!(areas_r, all_errors),
            stop_areas: unpack!(stop_areas_r, all_errors),
            networks: unpack!(networks_r, all_errors),
            route_networks: unpack!(route_networks_r, all_errors),
            fare_leg_join_rules: unpack!(fare_leg_join_rules_r, all_errors),
            geojson_locations,
        };

        (feed, all_errors)
    }

    /// Loads only the specified GTFS files from a feed source.
    ///
    /// `loaded_files` is populated from all files present in the source (not just
    /// the ones parsed), so that `feed.has_file("shapes.txt")` returns the correct
    /// answer for conditional field checks.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn load_only(
        source: &FeedSource,
        files: &HashSet<GtfsFiles>,
    ) -> (GtfsFeed, Vec<ParseError>) {
        let available = source.file_names();
        let want = |f: GtfsFiles| available.contains(&f) && files.contains(&f);

        macro_rules! parse_vec {
            ($file:expr, $parser:path) => {
                if want($file) {
                    match source.read_file($file) {
                        Ok(r) => $parser(r),
                        Err(_) => (vec![], vec![]),
                    }
                } else {
                    (vec![], vec![])
                }
            };
        }

        let agencies_r = parse_vec!(GtfsFiles::Agency, file_parsers::agency::parse);
        let stops_r = parse_vec!(GtfsFiles::Stops, file_parsers::stops::parse);
        let routes_r = parse_vec!(GtfsFiles::Routes, file_parsers::routes::parse);
        let trips_r = parse_vec!(GtfsFiles::Trips, file_parsers::trips::parse);
        let stop_times_r = parse_vec!(GtfsFiles::StopTimes, file_parsers::stop_times::parse);
        let calendars_r = parse_vec!(GtfsFiles::Calendar, file_parsers::calendar::parse);
        let cal_dates_r = parse_vec!(
            GtfsFiles::CalendarDates,
            file_parsers::calendar_dates::parse
        );
        let shapes_r = parse_vec!(GtfsFiles::Shapes, file_parsers::shapes::parse);
        let freqs_r = parse_vec!(GtfsFiles::Frequencies, file_parsers::frequencies::parse);
        let transfers_r = parse_vec!(GtfsFiles::Transfers, file_parsers::transfers::parse);
        let pathways_r = parse_vec!(GtfsFiles::Pathways, file_parsers::pathways::parse);
        let levels_r = parse_vec!(GtfsFiles::Levels, file_parsers::levels::parse);
        let fare_attr_r = parse_vec!(
            GtfsFiles::FareAttributes,
            file_parsers::fare_attributes::parse
        );
        let fare_rules_r = parse_vec!(GtfsFiles::FareRules, file_parsers::fare_rules::parse);
        let translations_r = parse_vec!(GtfsFiles::Translations, file_parsers::translations::parse);
        let attributions_r = parse_vec!(GtfsFiles::Attributions, file_parsers::attributions::parse);
        let booking_rules_r =
            parse_vec!(GtfsFiles::BookingRules, file_parsers::booking_rules::parse);
        let location_groups_r = parse_vec!(
            GtfsFiles::LocationGroups,
            file_parsers::location_groups::parse
        );
        let location_group_stops_r = parse_vec!(
            GtfsFiles::LocationGroupStops,
            file_parsers::location_group_stops::parse
        );

        let fare_media_r = parse_vec!(GtfsFiles::FareMedia, file_parsers::fare_media::parse);
        let fare_products_r =
            parse_vec!(GtfsFiles::FareProducts, file_parsers::fare_products::parse);
        let fare_leg_rules_r =
            parse_vec!(GtfsFiles::FareLegRules, file_parsers::fare_leg_rules::parse);
        let fare_transfer_rules_r = parse_vec!(
            GtfsFiles::FareTransferRules,
            file_parsers::fare_transfer_rules::parse
        );
        let rider_categories_r = parse_vec!(
            GtfsFiles::RiderCategories,
            file_parsers::rider_categories::parse
        );
        let timeframes_r = parse_vec!(GtfsFiles::Timeframes, file_parsers::timeframes::parse);
        let areas_r = parse_vec!(GtfsFiles::Areas, file_parsers::areas::parse);
        let stop_areas_r = parse_vec!(GtfsFiles::StopAreas, file_parsers::stop_areas::parse);
        let networks_r = parse_vec!(GtfsFiles::Networks, file_parsers::networks::parse);
        let route_networks_r = parse_vec!(
            GtfsFiles::RouteNetworks,
            file_parsers::route_networks::parse
        );
        let fare_leg_join_rules_r = parse_vec!(
            GtfsFiles::FareLegJoinRules,
            file_parsers::fare_leg_join_rules::parse
        );

        let feed_info_r = if want(GtfsFiles::FeedInfo) {
            match source.read_file(GtfsFiles::FeedInfo) {
                Ok(r) => file_parsers::feed_info::parse(r),
                Err(_) => (None, 0, vec![]),
            }
        } else {
            (None, 0, vec![])
        };

        let mut all_errors = Vec::new();

        macro_rules! unpack {
            ($result:expr, $errors:expr) => {{
                let mut r = $result;
                $errors.append(&mut r.1);
                r.0
            }};
        }

        let (feed_info, feed_info_line_count, mut feed_info_errors) = feed_info_r;
        all_errors.append(&mut feed_info_errors);

        let (geojson_locations, mut geojson_errors) = parse_geojson_locations(source);
        all_errors.append(&mut geojson_errors);

        // loaded_files reflects ALL files in the source, not just parsed ones
        let mut loaded_files: HashSet<String> = available
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        if source.read_geojson_locations().is_some() {
            loaded_files.insert(LOCATIONS_GEOJSON.to_string());
        }

        let feed = GtfsFeed {
            loaded_files,
            agencies: unpack!(agencies_r, all_errors),
            stops: unpack!(stops_r, all_errors),
            routes: unpack!(routes_r, all_errors),
            trips: unpack!(trips_r, all_errors),
            stop_times: unpack!(stop_times_r, all_errors),
            calendars: unpack!(calendars_r, all_errors),
            calendar_dates: unpack!(cal_dates_r, all_errors),
            shapes: unpack!(shapes_r, all_errors),
            frequencies: unpack!(freqs_r, all_errors),
            transfers: unpack!(transfers_r, all_errors),
            pathways: unpack!(pathways_r, all_errors),
            levels: unpack!(levels_r, all_errors),
            feed_info,
            feed_info_line_count,
            fare_attributes: unpack!(fare_attr_r, all_errors),
            fare_rules: unpack!(fare_rules_r, all_errors),
            translations: unpack!(translations_r, all_errors),
            attributions: unpack!(attributions_r, all_errors),
            booking_rules: unpack!(booking_rules_r, all_errors),
            location_groups: unpack!(location_groups_r, all_errors),
            location_group_stops: unpack!(location_group_stops_r, all_errors),
            fare_media: unpack!(fare_media_r, all_errors),
            fare_products: unpack!(fare_products_r, all_errors),
            fare_leg_rules: unpack!(fare_leg_rules_r, all_errors),
            fare_transfer_rules: unpack!(fare_transfer_rules_r, all_errors),
            rider_categories: unpack!(rider_categories_r, all_errors),
            timeframes: unpack!(timeframes_r, all_errors),
            areas: unpack!(areas_r, all_errors),
            stop_areas: unpack!(stop_areas_r, all_errors),
            networks: unpack!(networks_r, all_errors),
            route_networks: unpack!(route_networks_r, all_errors),
            fare_leg_join_rules: unpack!(fare_leg_join_rules_r, all_errors),
            geojson_locations,
        };

        (feed, all_errors)
    }

    /// Opens a ZIP file and indexes its entries without decompressing any content.
    ///
    /// The resulting [`FeedSource::Zip`] stores only the file path and an index
    /// mapping each recognized [`GtfsFiles`] variant to its entry name inside the
    /// archive. Actual decompression happens lazily via [`FeedSource::read_file`].
    fn open_zip(path: &Path) -> Result<FeedSource, ParserError> {
        let has_zip_extension = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"));

        if !has_zip_extension {
            return Err(ParserError::NotAGtfsFeed(path.to_path_buf()));
        }

        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let raw_names: Vec<String> = (0..archive.len())
            .filter_map(|i| {
                let name = archive.name_for_index(i)?;
                // Skip directory entries
                if name.ends_with('/') {
                    None
                } else {
                    Some(name.to_owned())
                }
            })
            .collect();

        let prefix = Self::detect_common_prefix(&raw_names);

        // Build index: GtfsFiles → entry name (no decompression)
        let mut index = HashMap::new();
        let mut geojson_entry: Option<String> = None;
        for raw_name in &raw_names {
            let normalized = raw_name.strip_prefix(&prefix).unwrap_or(raw_name);
            if let Ok(gtfs_file) = GtfsFiles::try_from(normalized) {
                index.insert(gtfs_file, raw_name.clone());
            } else if normalized == LOCATIONS_GEOJSON {
                geojson_entry = Some(raw_name.clone());
            }
        }

        let geojson_bytes = match geojson_entry {
            Some(name) => {
                use std::io::Read;
                let mut entry = archive.by_name(&name)?;
                let cap = usize::try_from(entry.size()).unwrap_or(0);
                let mut buf = Vec::with_capacity(cap);
                entry.read_to_end(&mut buf)?;
                Some(buf)
            }
            None => None,
        };

        Ok(FeedSource::Zip {
            path: path.to_path_buf(),
            index,
            raw_entry_names: raw_names,
            geojson_bytes,
        })
    }

    fn open_directory(path: &Path) -> Result<FeedSource, ParserError> {
        let mut file_names = Vec::new();
        let mut raw_entry_names = Vec::new();
        let mut has_geojson = false;

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if !file_type.is_file() {
                continue;
            }

            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            raw_entry_names.push(name_str.to_string());

            if let Ok(gtfs_file) = GtfsFiles::try_from(name_str.as_ref()) {
                file_names.push(gtfs_file);
            } else if name_str.as_ref() == LOCATIONS_GEOJSON {
                has_geojson = true;
            }
        }

        file_names.sort_by_key(std::string::ToString::to_string);
        raw_entry_names.sort();

        let geojson_bytes = if has_geojson {
            Some(std::fs::read(path.join(LOCATIONS_GEOJSON))?)
        } else {
            None
        };

        Ok(FeedSource::Directory {
            path: path.to_path_buf(),
            file_names,
            raw_entry_names,
            geojson_bytes,
        })
    }

    fn detect_common_prefix(names: &[String]) -> String {
        if names.is_empty() {
            return String::new();
        }

        let first = &names[0];
        let Some(slash_pos) = first.rfind('/') else {
            return String::new();
        };
        let candidate = &first[..=slash_pos];

        let all_share = names.iter().all(|name| name.starts_with(candidate));
        if all_share {
            candidate.to_owned()
        } else {
            String::new()
        }
    }
}
