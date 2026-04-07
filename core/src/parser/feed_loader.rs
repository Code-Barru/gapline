use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::Path;

use crate::models::GtfsFeed;
use crate::parser::error::{ParseError, ParserError};
use crate::parser::feed_source::{FeedSource, GtfsFiles};
use crate::parser::file_parsers;

pub struct FeedLoader;

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
        // Each branch is independent — no shared mutable state.
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

        // feed_info is small — parse sequentially
        let feed_info_r = if has(GtfsFiles::FeedInfo) {
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
        let (feed_info, feed_info_line_count, mut feed_info_errors) = feed_info_r;
        all_errors.append(&mut feed_info_errors);

        let loaded_files: HashSet<String> = available
            .iter()
            .map(std::string::ToString::to_string)
            .collect();

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

        // loaded_files reflects ALL files in the source, not just parsed ones
        let loaded_files: HashSet<String> = available
            .iter()
            .map(std::string::ToString::to_string)
            .collect();

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
        };

        (feed, all_errors)
    }

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
                let entry = archive.by_index(i).ok()?;
                if entry.is_dir() {
                    None
                } else {
                    Some(entry.name().to_owned())
                }
            })
            .collect();

        let prefix = Self::detect_common_prefix(&raw_names);

        let mut files = HashMap::new();
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            if entry.is_dir() {
                continue;
            }

            let raw_name = entry.name().to_owned();
            let normalized = raw_name.strip_prefix(&prefix).unwrap_or(&raw_name);

            let Ok(gtfs_file) = GtfsFiles::try_from(normalized) else {
                continue;
            };

            let capacity = usize::try_from(entry.size()).unwrap_or(0);
            let mut buf = Vec::with_capacity(capacity);
            entry.read_to_end(&mut buf)?;

            files.insert(gtfs_file, buf);
        }

        Ok(FeedSource::Zip {
            files,
            raw_entry_names: raw_names,
        })
    }

    fn open_directory(path: &Path) -> Result<FeedSource, ParserError> {
        let mut file_names = Vec::new();
        let mut raw_entry_names = Vec::new();

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
            }
        }

        file_names.sort_by_key(std::string::ToString::to_string);
        raw_entry_names.sort();

        Ok(FeedSource::Directory {
            path: path.to_path_buf(),
            file_names,
            raw_entry_names,
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
