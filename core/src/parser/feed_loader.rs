use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use crate::parser::error::ParserError;
use crate::parser::feed_source::{FeedSource, GtfsFiles};

/// Entry point for loading a GTFS feed from a ZIP archive or directory.
///
/// `FeedLoader` detects the feed format (ZIP or directory) and returns a
/// [`FeedSource`] that provides uniform access to the raw file contents.
/// It does **not** parse CSV data into structs — that is handled by later stages.
///
/// Only files matching a known [`GtfsFiles`] variant are indexed.
/// Unknown files (e.g. `custom_data.txt`) are silently ignored.
pub struct FeedLoader;

impl FeedLoader {
    /// Opens a GTFS feed at the given path.
    ///
    /// # Behaviour
    ///
    /// - If `path` does not exist, returns [`ParserError::FileNotFound`].
    /// - If `path` is a `.zip` file, the archive is read in memory via the `zip` crate.
    ///   Files inside a single subdirectory (e.g. `gtfs/agency.txt`) have their prefix
    ///   normalized. Only files matching a known [`GtfsFiles`] variant are kept.
    /// - If `path` is a directory, only `.txt` files at the root level that match a
    ///   known [`GtfsFiles`] variant are listed.
    /// - Otherwise, returns [`ParserError::NotAGtfsFeed`].
    ///
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

    /// Opens a ZIP archive and reads all recognized GTFS entries into memory.
    fn open_zip(path: &Path) -> Result<FeedSource, ParserError> {
        let has_zip_extension = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"));

        if !has_zip_extension {
            return Err(ParserError::NotAGtfsFeed(path.to_path_buf()));
        }

        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // First pass: collect raw entry names (excluding directories).
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

        // Detect a common subdirectory prefix shared by all entries.
        let prefix = Self::detect_common_prefix(&raw_names);

        // Second pass: read file contents, normalize names, and keep only recognized files.
        let mut files = HashMap::new();
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            if entry.is_dir() {
                continue;
            }

            let raw_name = entry.name().to_owned();
            let normalized = raw_name.strip_prefix(&prefix).unwrap_or(&raw_name);

            let Ok(gtfs_file) = GtfsFiles::try_from(normalized) else {
                continue; // Unknown file — skip silently
            };

            let capacity = usize::try_from(entry.size()).unwrap_or(0);
            let mut buf = Vec::with_capacity(capacity);
            entry.read_to_end(&mut buf)?;

            files.insert(gtfs_file, buf);
        }

        Ok(FeedSource::Zip { files })
    }

    /// Opens a directory and lists recognized GTFS `.txt` files at the root level.
    fn open_directory(path: &Path) -> Result<FeedSource, ParserError> {
        let mut file_names = Vec::new();

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if !file_type.is_file() {
                continue;
            }

            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if let Ok(gtfs_file) = GtfsFiles::try_from(name_str.as_ref()) {
                file_names.push(gtfs_file);
            }
        }

        file_names.sort_by_key(std::string::ToString::to_string);

        Ok(FeedSource::Directory {
            path: path.to_path_buf(),
            file_names,
        })
    }

    /// Detects a common directory prefix shared by all file entries.
    ///
    /// For example, if all entries start with `"gtfs/"`, the prefix is `"gtfs/"`.
    /// Returns an empty string if there is no common prefix or entries are at root level.
    fn detect_common_prefix(names: &[String]) -> String {
        if names.is_empty() {
            return String::new();
        }

        // Find the prefix of the first entry (everything up to and including the last `/`).
        let first = &names[0];
        let Some(slash_pos) = first.rfind('/') else {
            return String::new();
        };
        let candidate = &first[..=slash_pos];

        // Check if all other entries share this prefix.
        let all_share = names.iter().all(|name| name.starts_with(candidate));
        if all_share {
            candidate.to_owned()
        } else {
            String::new()
        }
    }
}
