use std::io::{BufRead, Write};
use std::path::Path;

use headway_core::parser::{FeedLoader, FeedSource, GtfsFiles, ParserError};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Creates a ZIP archive in memory with the given entries (name -> content).
fn create_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (name, content) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap();
    }
    buf
}

/// Writes raw bytes to a file in the given directory.
fn write_zip_file(dir: &TempDir, name: &str, content: &[u8]) -> std::path::PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path
}

/// Reads the full content of a `BufRead` into a string.
fn read_to_string(reader: &mut dyn BufRead) -> String {
    let mut s = String::new();
    reader.read_to_string(&mut s).unwrap();
    s
}

// ---------------------------------------------------------------------------
// Test 1: ZIP valid — file names (CA2, CA4, cas test 1)
// ---------------------------------------------------------------------------

#[test]
fn test_zip_valid_file_names() {
    let dir = TempDir::new().unwrap();
    let zip_bytes = create_zip(&[
        ("agency.txt", b"agency_id,agency_name\n1,Test Agency"),
        ("stops.txt", b"stop_id,stop_name\n100,Main St"),
        ("routes.txt", b"route_id,route_short_name\nR1,Red"),
    ]);
    let zip_path = write_zip_file(&dir, "feed.zip", &zip_bytes);

    let source = FeedLoader::open(&zip_path).unwrap();
    let mut names = source.file_names();
    names.sort_by_key(|f| f.to_string());

    assert_eq!(
        names,
        vec![GtfsFiles::Agency, GtfsFiles::Routes, GtfsFiles::Stops]
    );
}

// ---------------------------------------------------------------------------
// Test 2: ZIP valid — read file (CA5, cas test 2)
// ---------------------------------------------------------------------------

#[test]
fn test_zip_read_file() {
    let dir = TempDir::new().unwrap();
    let content = b"agency_id,agency_name\n1,Test Agency";
    let zip_bytes = create_zip(&[("agency.txt", content)]);
    let zip_path = write_zip_file(&dir, "feed.zip", &zip_bytes);

    let source = FeedLoader::open(&zip_path).unwrap();
    let mut reader = source.read_file(GtfsFiles::Agency).unwrap();
    let text = read_to_string(&mut *reader);

    assert_eq!(text, "agency_id,agency_name\n1,Test Agency");
}

// ---------------------------------------------------------------------------
// Test 3: ZIP with subdirectory — prefix normalization (CA9, cas test 3)
// ---------------------------------------------------------------------------

#[test]
fn test_zip_with_subdirectory() {
    let dir = TempDir::new().unwrap();
    let zip_bytes = create_zip(&[
        ("gtfs/agency.txt", b"agency_id\n1"),
        ("gtfs/stops.txt", b"stop_id\n100"),
    ]);
    let zip_path = write_zip_file(&dir, "feed.zip", &zip_bytes);

    let source = FeedLoader::open(&zip_path).unwrap();
    let mut names = source.file_names();
    names.sort_by_key(|f| f.to_string());

    assert_eq!(names, vec![GtfsFiles::Agency, GtfsFiles::Stops]);

    // Verify the file is readable via the enum variant.
    let mut reader = source.read_file(GtfsFiles::Agency).unwrap();
    let text = read_to_string(&mut *reader);
    assert_eq!(text, "agency_id\n1");
}

// ---------------------------------------------------------------------------
// Test 4: Directory valid — file names (CA3, cas test 4)
// ---------------------------------------------------------------------------

#[test]
fn test_directory_valid_file_names() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("agency.txt"), "agency_id\n1").unwrap();
    std::fs::write(dir.path().join("stops.txt"), "stop_id\n100").unwrap();
    std::fs::write(dir.path().join("calendar.txt"), "service_id\nS1").unwrap();

    let source = FeedLoader::open(dir.path()).unwrap();
    let mut names = source.file_names();
    names.sort_by_key(|f| f.to_string());

    assert_eq!(
        names,
        vec![GtfsFiles::Agency, GtfsFiles::Calendar, GtfsFiles::Stops]
    );
}

// ---------------------------------------------------------------------------
// Test 5: Directory valid — read file (CA5, cas test 5)
// ---------------------------------------------------------------------------

#[test]
fn test_directory_read_file() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("stops.txt"),
        "stop_id,stop_name\n100,Main St",
    )
    .unwrap();

    let source = FeedLoader::open(dir.path()).unwrap();
    let mut reader = source.read_file(GtfsFiles::Stops).unwrap();
    let text = read_to_string(&mut *reader);

    assert_eq!(text, "stop_id,stop_name\n100,Main St");
}

// ---------------------------------------------------------------------------
// Test 6: Directory filters non-.txt and unknown .txt files (CA3, cas test 6)
// ---------------------------------------------------------------------------

#[test]
fn test_directory_filters_non_gtfs() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("agency.txt"), "data").unwrap();
    std::fs::write(dir.path().join("readme.md"), "# Readme").unwrap();
    std::fs::write(dir.path().join(".DS_Store"), "binary").unwrap();
    std::fs::write(dir.path().join("custom_data.txt"), "unknown").unwrap();

    let source = FeedLoader::open(dir.path()).unwrap();
    let names = source.file_names();

    assert_eq!(names, vec![GtfsFiles::Agency]);
}

// ---------------------------------------------------------------------------
// Test 7: Path not found (CA6, cas test 7)
// ---------------------------------------------------------------------------

#[test]
fn test_path_not_found() {
    let result = FeedLoader::open(Path::new("/tmp/headway_nonexistent_path_12345.zip"));

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, ParserError::FileNotFound(ref p) if p.to_string_lossy().contains("headway_nonexistent")),
        "Expected FileNotFound, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Test 8: Corrupted ZIP (CA7, cas test 8)
// ---------------------------------------------------------------------------

#[test]
fn test_zip_corrupted() {
    let dir = TempDir::new().unwrap();
    // Write random bytes with a .zip extension.
    let path = write_zip_file(&dir, "bad.zip", b"this is not a zip file at all");

    let result = FeedLoader::open(&path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, ParserError::ZipExtraction(_)),
        "Expected ZipExtraction, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Test 9: Not a ZIP and not a directory (CA8, cas test 9)
// ---------------------------------------------------------------------------

#[test]
fn test_not_zip_not_directory() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("feed.csv");
    std::fs::write(&path, "some,csv,data").unwrap();

    let result = FeedLoader::open(&path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, ParserError::NotAGtfsFeed(_)),
        "Expected NotAGtfsFeed, got: {err}"
    );
    // Verify the error message includes the file name.
    let msg = format!("{err}");
    assert!(
        msg.contains("feed.csv"),
        "Error message should include file name: {msg}"
    );
}

// ---------------------------------------------------------------------------
// Test 10: Empty ZIP (cas test 10)
// ---------------------------------------------------------------------------

#[test]
fn test_zip_empty() {
    let dir = TempDir::new().unwrap();
    let zip_bytes = create_zip(&[]);
    let zip_path = write_zip_file(&dir, "empty.zip", &zip_bytes);

    let source = FeedLoader::open(&zip_path).unwrap();
    let names = source.file_names();

    assert!(names.is_empty());
}

// ---------------------------------------------------------------------------
// Test 11: Read nonexistent GTFS file in feed (cas test 11)
// ---------------------------------------------------------------------------

#[test]
fn test_read_nonexistent_file_in_feed() {
    let dir = TempDir::new().unwrap();
    let zip_bytes = create_zip(&[("agency.txt", b"data")]);
    let zip_path = write_zip_file(&dir, "feed.zip", &zip_bytes);

    let source = FeedLoader::open(&zip_path).unwrap();
    let result = source.read_file(GtfsFiles::Translations);

    match result {
        Err(ParserError::GtfsFileNotFound(GtfsFiles::Translations)) => {} // expected
        Err(other) => panic!("Expected GtfsFileNotFound(Translations), got: {other}"),
        Ok(_) => panic!("Expected error for missing GTFS file in feed"),
    }
}

// ---------------------------------------------------------------------------
// Test 12: Empty directory (cas test 12)
// ---------------------------------------------------------------------------

#[test]
fn test_directory_empty() {
    let dir = TempDir::new().unwrap();

    let source = FeedLoader::open(dir.path()).unwrap();
    let names = source.file_names();

    assert!(names.is_empty());
}

// ---------------------------------------------------------------------------
// Test 13: ZIP with UTF-8 BOM preserved (CA11, cas test 13)
// ---------------------------------------------------------------------------

#[test]
fn test_zip_utf8_bom_preserved() {
    let dir = TempDir::new().unwrap();
    // UTF-8 BOM followed by CSV content.
    let bom_content = b"\xEF\xBB\xBFagency_id,agency_name\n1,Test";
    let zip_bytes = create_zip(&[("agency.txt", bom_content)]);
    let zip_path = write_zip_file(&dir, "feed.zip", &zip_bytes);

    let source = FeedLoader::open(&zip_path).unwrap();
    let mut reader = source.read_file(GtfsFiles::Agency).unwrap();
    let text = read_to_string(&mut *reader);

    // BOM should be preserved (stripping is responsibility of the CSV parser in HW-006/007).
    assert!(
        text.starts_with('\u{FEFF}'),
        "UTF-8 BOM should be preserved"
    );
    assert!(text.contains("agency_id"));
}

// ---------------------------------------------------------------------------
// Test 14: Insufficient permissions (cas test 14) — Linux only
// ---------------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn test_insufficient_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let zip_bytes = create_zip(&[("agency.txt", b"data")]);
    let zip_path = write_zip_file(&dir, "noperm.zip", &zip_bytes);

    // Remove read permissions.
    std::fs::set_permissions(&zip_path, std::fs::Permissions::from_mode(0o000)).unwrap();

    let result = FeedLoader::open(&zip_path);

    // Restore permissions so TempDir cleanup doesn't fail.
    std::fs::set_permissions(&zip_path, std::fs::Permissions::from_mode(0o644)).unwrap();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, ParserError::Io(_)),
        "Expected Io error for permission denied, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Test 15: ParserError implements Display and Error (CA10)
// ---------------------------------------------------------------------------

#[test]
fn test_parser_error_display_and_error() {
    // FileNotFound
    let err = ParserError::FileNotFound(std::path::PathBuf::from("/some/path"));
    let msg = format!("{err}");
    assert!(msg.contains("/some/path"));

    // GtfsFileNotFound
    let err = ParserError::GtfsFileNotFound(GtfsFiles::Agency);
    let msg = format!("{err}");
    assert!(
        msg.contains("agency.txt"),
        "Display should use file name: {msg}"
    );

    // Debug
    let debug = format!("{err:?}");
    assert!(!debug.is_empty());

    // std::error::Error
    let _: &dyn std::error::Error = &err;
}

// ---------------------------------------------------------------------------
// Test 16: read_file on directory — nonexistent GTFS file
// ---------------------------------------------------------------------------

#[test]
fn test_directory_read_nonexistent_file() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("agency.txt"), "data").unwrap();

    let source = FeedLoader::open(dir.path()).unwrap();
    let result = source.read_file(GtfsFiles::Translations);

    match result {
        Err(ParserError::GtfsFileNotFound(GtfsFiles::Translations)) => {} // expected
        Err(other) => panic!("Expected GtfsFileNotFound(Translations), got: {other}"),
        Ok(_) => panic!("Expected error for missing GTFS file in directory feed"),
    }
}

// ---------------------------------------------------------------------------
// Test 17: FeedSource variant is Zip for zip files, Directory for dirs
// ---------------------------------------------------------------------------

#[test]
fn test_feed_source_variants() {
    let dir = TempDir::new().unwrap();

    // ZIP variant
    let zip_bytes = create_zip(&[("agency.txt", b"data")]);
    let zip_path = write_zip_file(&dir, "feed.zip", &zip_bytes);
    let source = FeedLoader::open(&zip_path).unwrap();
    assert!(matches!(source, FeedSource::Zip { .. }));

    // Directory variant
    let sub_dir = TempDir::new().unwrap();
    std::fs::write(sub_dir.path().join("agency.txt"), "data").unwrap();
    let source = FeedLoader::open(sub_dir.path()).unwrap();
    assert!(matches!(source, FeedSource::Directory { .. }));
}

// ---------------------------------------------------------------------------
// Test 18: TryFrom<&str> — valid filename
// ---------------------------------------------------------------------------

#[test]
fn test_try_from_valid() {
    assert_eq!(GtfsFiles::try_from("agency.txt"), Ok(GtfsFiles::Agency));
    assert_eq!(GtfsFiles::try_from("stops.txt"), Ok(GtfsFiles::Stops));
    assert_eq!(
        GtfsFiles::try_from("stop_times.txt"),
        Ok(GtfsFiles::StopTimes)
    );
    assert_eq!(
        GtfsFiles::try_from("fare_leg_join_rules.txt"),
        Ok(GtfsFiles::FareLegJoinRules)
    );
    assert_eq!(
        GtfsFiles::try_from("attributions.txt"),
        Ok(GtfsFiles::Attributions)
    );
}

// ---------------------------------------------------------------------------
// Test 19: TryFrom<&str> — unknown filename
// ---------------------------------------------------------------------------

#[test]
fn test_try_from_unknown() {
    assert_eq!(GtfsFiles::try_from("custom.txt"), Err(()));
    assert_eq!(GtfsFiles::try_from("readme.md"), Err(()));
    assert_eq!(GtfsFiles::try_from(""), Err(()));
    assert_eq!(GtfsFiles::try_from("AGENCY.TXT"), Err(()));
}

// ---------------------------------------------------------------------------
// Test 20: Display — shows correct filename
// ---------------------------------------------------------------------------

#[test]
fn test_display() {
    assert_eq!(format!("{}", GtfsFiles::Agency), "agency.txt");
    assert_eq!(format!("{}", GtfsFiles::StopTimes), "stop_times.txt");
    assert_eq!(
        format!("{}", GtfsFiles::CalendarDates),
        "calendar_dates.txt"
    );
    assert_eq!(format!("{}", GtfsFiles::FeedInfo), "feed_info.txt");
    assert_eq!(
        format!("{}", GtfsFiles::FareLegJoinRules),
        "fare_leg_join_rules.txt"
    );
}

// ---------------------------------------------------------------------------
// Test 21: Unknown .txt files in ZIP are ignored
// ---------------------------------------------------------------------------

#[test]
fn test_zip_ignores_unknown_files() {
    let dir = TempDir::new().unwrap();
    let zip_bytes = create_zip(&[
        ("agency.txt", b"agency_id\n1"),
        ("custom_data.txt", b"something"),
        ("notes.txt", b"internal notes"),
        ("stops.txt", b"stop_id\n100"),
    ]);
    let zip_path = write_zip_file(&dir, "feed.zip", &zip_bytes);

    let source = FeedLoader::open(&zip_path).unwrap();
    let mut names = source.file_names();
    names.sort_by_key(|f| f.to_string());

    // Only recognized GTFS files should be present.
    assert_eq!(names, vec![GtfsFiles::Agency, GtfsFiles::Stops]);
}
