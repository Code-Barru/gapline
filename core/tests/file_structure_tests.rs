use std::collections::HashMap;

use headway_core::parser::{FeedSource, GtfsFiles};
use headway_core::validation::Severity;
use headway_core::validation::file_structure::{
    CsvParsingFailedRule, DuplicatedColumnRule, EmptyColumnNameRule, EmptyFileRule, EmptyRowRule,
    InvalidInputFilesInSubfolderRule, InvalidRowLengthRule, MissingCalendarFilesRule,
    MissingRecommendedFileRule, MissingRequiredFileRule, NewLineInValueRule,
    StructuralValidationRule, TooManyRowsRule, UnknownColumnRule, UnknownFileRule,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Builds a `FeedSource::InMemory` with the given known GTFS files.
/// Each file gets dummy content so `read_file` works.
fn zip_source(files: &[GtfsFiles]) -> FeedSource {
    let map: HashMap<GtfsFiles, Vec<u8>> = files
        .iter()
        .map(|f| (*f, b"header\ndata".to_vec()))
        .collect();
    FeedSource::InMemory {
        files: map,
        raw_entry_names: files.iter().map(std::string::ToString::to_string).collect(),
    }
}

/// Builds a `FeedSource::InMemory` with explicit content per file.
fn zip_source_with_content(entries: &[(GtfsFiles, &[u8])]) -> FeedSource {
    let map: HashMap<GtfsFiles, Vec<u8>> = entries
        .iter()
        .map(|(f, content)| (*f, content.to_vec()))
        .collect();
    let raw: Vec<String> = entries.iter().map(|(f, _)| f.to_string()).collect();
    FeedSource::InMemory {
        files: map,
        raw_entry_names: raw,
    }
}

/// Builds a `FeedSource::InMemory` with explicit content AND custom raw entry names.
fn zip_source_with_raw(entries: &[(GtfsFiles, &[u8])], raw_entry_names: Vec<String>) -> FeedSource {
    let map: HashMap<GtfsFiles, Vec<u8>> = entries
        .iter()
        .map(|(f, content)| (*f, content.to_vec()))
        .collect();
    FeedSource::InMemory {
        files: map,
        raw_entry_names,
    }
}

// ===========================================================================
// missing_required_file
// ===========================================================================

#[test]
fn missing_required_file_all_present() {
    let source = zip_source(&[
        GtfsFiles::Agency,
        GtfsFiles::Routes,
        GtfsFiles::Trips,
        GtfsFiles::StopTimes,
        GtfsFiles::Calendar,
    ]);
    let errors = MissingRequiredFileRule.validate(&source);
    assert!(errors.is_empty());
}

#[test]
fn missing_required_file_agency_missing() {
    let source = zip_source(&[GtfsFiles::Routes, GtfsFiles::Trips, GtfsFiles::StopTimes]);
    let errors = MissingRequiredFileRule.validate(&source);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "missing_required_file");
    assert_eq!(errors[0].file_name.as_deref(), Some("agency.txt"));
    assert_eq!(errors[0].severity, Severity::Error);
}

#[test]
fn missing_required_file_multiple_missing() {
    let source = zip_source(&[GtfsFiles::Agency, GtfsFiles::Calendar]);
    let errors = MissingRequiredFileRule.validate(&source);
    assert_eq!(errors.len(), 3);

    let missing: Vec<&str> = errors
        .iter()
        .filter_map(|e| e.file_name.as_deref())
        .collect();
    assert!(missing.contains(&"routes.txt"));
    assert!(missing.contains(&"trips.txt"));
    assert!(missing.contains(&"stop_times.txt"));
}

#[test]
fn missing_required_file_empty_feed() {
    let source = zip_source(&[]);
    let errors = MissingRequiredFileRule.validate(&source);
    assert_eq!(errors.len(), 4);
}

// ===========================================================================
// missing_calendar_and_calendar_date_files
// ===========================================================================

#[test]
fn missing_calendar_files_both_absent() {
    let source = zip_source(&[GtfsFiles::Agency, GtfsFiles::Routes]);
    let errors = MissingCalendarFilesRule.validate(&source);
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].rule_id,
        "missing_calendar_and_calendar_date_files"
    );
    assert_eq!(errors[0].severity, Severity::Error);
}

#[test]
fn missing_calendar_files_calendar_present() {
    let source = zip_source(&[GtfsFiles::Calendar]);
    let errors = MissingCalendarFilesRule.validate(&source);
    assert!(errors.is_empty());
}

#[test]
fn missing_calendar_files_calendar_dates_present() {
    let source = zip_source(&[GtfsFiles::CalendarDates]);
    let errors = MissingCalendarFilesRule.validate(&source);
    assert!(errors.is_empty());
}

#[test]
fn missing_calendar_files_both_present() {
    let source = zip_source(&[GtfsFiles::Calendar, GtfsFiles::CalendarDates]);
    let errors = MissingCalendarFilesRule.validate(&source);
    assert!(errors.is_empty());
}

// ===========================================================================
// empty_file
// ===========================================================================

#[test]
fn empty_file_zero_bytes() {
    let source = zip_source_with_content(&[(GtfsFiles::Agency, b"")]);
    let errors = EmptyFileRule.validate(&source);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "empty_file");
    assert_eq!(errors[0].file_name.as_deref(), Some("agency.txt"));
    assert_eq!(errors[0].severity, Severity::Error);
}

#[test]
fn empty_file_header_only() {
    let source = zip_source_with_content(&[(GtfsFiles::Stops, b"stop_id,stop_name\n")]);
    let errors = EmptyFileRule.validate(&source);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "empty_file");
    assert_eq!(errors[0].file_name.as_deref(), Some("stops.txt"));
}

#[test]
fn empty_file_header_no_trailing_newline() {
    let source = zip_source_with_content(&[(GtfsFiles::Stops, b"stop_id,stop_name")]);
    let errors = EmptyFileRule.validate(&source);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "empty_file");
}

#[test]
fn empty_file_valid_with_data() {
    let source = zip_source_with_content(&[(GtfsFiles::Agency, b"agency_id\n1")]);
    let errors = EmptyFileRule.validate(&source);
    assert!(errors.is_empty());
}

#[test]
fn empty_file_multiple_empty() {
    let source = zip_source_with_content(&[
        (GtfsFiles::Agency, b""),
        (GtfsFiles::Stops, b"stop_id\n"),
        (GtfsFiles::Routes, b"route_id\nR1"),
    ]);
    let errors = EmptyFileRule.validate(&source);
    assert_eq!(errors.len(), 2);
}

// ===========================================================================
// empty_column_name
// ===========================================================================

#[test]
fn empty_column_name_detected() {
    let source = zip_source_with_content(&[(GtfsFiles::Routes, b"route_id,,route_type\nR1,x,3")]);
    let errors = EmptyColumnNameRule.validate(&source);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "empty_column_name");
    assert_eq!(errors[0].file_name.as_deref(), Some("routes.txt"));
    assert_eq!(errors[0].line_number, Some(1));
    assert_eq!(errors[0].severity, Severity::Error);
}

#[test]
fn empty_column_name_valid_header() {
    let source = zip_source_with_content(&[(GtfsFiles::Agency, b"agency_id,agency_name\n1,Test")]);
    let errors = EmptyColumnNameRule.validate(&source);
    assert!(errors.is_empty());
}

#[test]
fn empty_column_name_trailing_comma() {
    let source = zip_source_with_content(&[(GtfsFiles::Stops, b"stop_id,stop_name,\n1,Main,")]);
    let errors = EmptyColumnNameRule.validate(&source);
    assert_eq!(errors.len(), 1);
}

// ===========================================================================
// duplicated_column
// ===========================================================================

#[test]
fn duplicated_column_detected() {
    let source =
        zip_source_with_content(&[(GtfsFiles::Trips, b"trip_id,route_id,trip_id\n1,R1,1")]);
    let errors = DuplicatedColumnRule.validate(&source);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "duplicated_column");
    assert_eq!(errors[0].file_name.as_deref(), Some("trips.txt"));
    assert_eq!(errors[0].field_name.as_deref(), Some("trip_id"));
    assert_eq!(errors[0].severity, Severity::Error);
}

#[test]
fn duplicated_column_none() {
    let source =
        zip_source_with_content(&[(GtfsFiles::Trips, b"trip_id,route_id,service_id\n1,R1,S1")]);
    let errors = DuplicatedColumnRule.validate(&source);
    assert!(errors.is_empty());
}

// ===========================================================================
// invalid_row_length
// ===========================================================================

#[test]
fn invalid_row_length_too_many() {
    let source = zip_source_with_content(&[(
        GtfsFiles::Stops,
        b"stop_id,stop_name,stop_lat\n1,Main,0,extra,bonus",
    )]);
    let errors = InvalidRowLengthRule.validate(&source);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "invalid_row_length");
    assert_eq!(errors[0].line_number, Some(2));
    assert_eq!(errors[0].value.as_deref(), Some("5"));
}

#[test]
fn invalid_row_length_too_few() {
    let source = zip_source_with_content(&[(GtfsFiles::Stops, b"stop_id,stop_name,stop_lat\n1")]);
    let errors = InvalidRowLengthRule.validate(&source);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].value.as_deref(), Some("1"));
}

#[test]
fn invalid_row_length_valid() {
    let source =
        zip_source_with_content(&[(GtfsFiles::Stops, b"stop_id,stop_name\n1,Main\n2,Oak")]);
    let errors = InvalidRowLengthRule.validate(&source);
    assert!(errors.is_empty());
}

// ===========================================================================
// new_line_in_value
// ===========================================================================

#[test]
fn new_line_in_value_detected() {
    // Unclosed quote with a newline inside.
    let content = b"stop_id,stop_name\n1,\"Main\nStreet\"";
    let source = zip_source_with_content(&[(GtfsFiles::Stops, content)]);
    let errors = NewLineInValueRule.validate(&source);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "new_line_in_value");
    assert_eq!(errors[0].file_name.as_deref(), Some("stops.txt"));
    assert_eq!(errors[0].line_number, Some(2));
}

#[test]
fn new_line_in_value_escaped_quotes_ok() {
    // Properly escaped double quotes, no newline issue.
    let content = b"stop_id,stop_name\n1,\"Main \"\"Street\"\"\"";
    let source = zip_source_with_content(&[(GtfsFiles::Stops, content)]);
    let errors = NewLineInValueRule.validate(&source);
    assert!(errors.is_empty());
}

#[test]
fn new_line_in_value_no_quotes() {
    let source =
        zip_source_with_content(&[(GtfsFiles::Stops, b"stop_id,stop_name\n1,Main\n2,Oak")]);
    let errors = NewLineInValueRule.validate(&source);
    assert!(errors.is_empty());
}

// ===========================================================================
// invalid_input_files_in_subfolder
// ===========================================================================

#[test]
fn subfolder_detected() {
    let source = zip_source_with_raw(
        &[(GtfsFiles::Agency, b"agency_id\n1")],
        vec!["gtfs/agency.txt".to_owned(), "gtfs/stops.txt".to_owned()],
    );
    let errors = InvalidInputFilesInSubfolderRule.validate(&source);
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0].rule_id, "invalid_input_files_in_subfolder");
}

#[test]
fn subfolder_none() {
    let source = zip_source_with_raw(
        &[(GtfsFiles::Agency, b"agency_id\n1")],
        vec!["agency.txt".to_owned()],
    );
    let errors = InvalidInputFilesInSubfolderRule.validate(&source);
    assert!(errors.is_empty());
}

// ===========================================================================
// csv_parsing_failed
// ===========================================================================

#[test]
fn csv_parsing_failed_invalid_utf8() {
    let invalid_bytes: &[u8] = &[0xFF, 0xFE, 0x00, 0x01];
    let source = zip_source_with_content(&[(GtfsFiles::Agency, invalid_bytes)]);
    let errors = CsvParsingFailedRule.validate(&source);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "csv_parsing_failed");
    assert_eq!(errors[0].severity, Severity::Error);
}

#[test]
fn csv_parsing_failed_valid_utf8() {
    let source = zip_source_with_content(&[(GtfsFiles::Agency, b"agency_id\n1")]);
    let errors = CsvParsingFailedRule.validate(&source);
    assert!(errors.is_empty());
}

// ===========================================================================
// too_many_rows
// ===========================================================================

#[test]
fn too_many_rows_exceeded() {
    let content = b"stop_id\n1\n2\n3\n4\n5";
    let source = zip_source_with_content(&[(GtfsFiles::Stops, content)]);
    let errors = TooManyRowsRule::new(Some(3)).validate(&source);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "too_many_rows");
    assert_eq!(errors[0].value.as_deref(), Some("5"));
}

#[test]
fn too_many_rows_within_limit() {
    let content = b"stop_id\n1\n2\n3";
    let source = zip_source_with_content(&[(GtfsFiles::Stops, content)]);
    let errors = TooManyRowsRule::new(Some(3)).validate(&source);
    assert!(errors.is_empty());
}

#[test]
fn too_many_rows_disabled() {
    let content = b"stop_id\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10";
    let source = zip_source_with_content(&[(GtfsFiles::Stops, content)]);
    let errors = TooManyRowsRule::new(None).validate(&source);
    assert!(errors.is_empty());
}

// ===========================================================================
// missing_recommended_file
// ===========================================================================

#[test]
fn missing_recommended_file_both_absent() {
    let source = zip_source(&[GtfsFiles::Agency]);
    let errors = MissingRecommendedFileRule.validate(&source);
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0].rule_id, "missing_recommended_file");
    assert_eq!(errors[0].severity, Severity::Warning);

    let files: Vec<&str> = errors
        .iter()
        .filter_map(|e| e.file_name.as_deref())
        .collect();
    assert!(files.contains(&"feed_info.txt"));
    assert!(files.contains(&"shapes.txt"));
}

#[test]
fn missing_recommended_file_all_present() {
    let source = zip_source(&[GtfsFiles::Agency, GtfsFiles::FeedInfo, GtfsFiles::Shapes]);
    let errors = MissingRecommendedFileRule.validate(&source);
    assert!(errors.is_empty());
}

// ===========================================================================
// empty_row
// ===========================================================================

#[test]
fn empty_row_detected() {
    let source = zip_source_with_content(&[(GtfsFiles::Stops, b"stop_id\n1\n   \n3")]);
    let errors = EmptyRowRule.validate(&source);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "empty_row");
    assert_eq!(errors[0].line_number, Some(3));
    assert_eq!(errors[0].severity, Severity::Warning);
}

#[test]
fn empty_row_none() {
    let source = zip_source_with_content(&[(GtfsFiles::Stops, b"stop_id\n1\n2\n3")]);
    let errors = EmptyRowRule.validate(&source);
    assert!(errors.is_empty());
}

// ===========================================================================
// unknown_file
// ===========================================================================

#[test]
fn unknown_file_detected() {
    let source = zip_source_with_raw(
        &[(GtfsFiles::Agency, b"agency_id\n1")],
        vec!["agency.txt".to_owned(), "README.md".to_owned()],
    );
    let errors = UnknownFileRule.validate(&source);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "unknown_file");
    assert_eq!(errors[0].file_name.as_deref(), Some("README.md"));
    assert_eq!(errors[0].severity, Severity::Info);
}

#[test]
fn unknown_file_none() {
    let source = zip_source_with_raw(
        &[(GtfsFiles::Agency, b"agency_id\n1")],
        vec!["agency.txt".to_owned()],
    );
    let errors = UnknownFileRule.validate(&source);
    assert!(errors.is_empty());
}

// ===========================================================================
// unknown_column
// ===========================================================================

#[test]
fn unknown_column_detected() {
    let source = zip_source_with_content(&[(
        GtfsFiles::Agency,
        b"agency_id,agency_name,agency_color\n1,Test,red",
    )]);
    let errors = UnknownColumnRule.validate(&source);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "unknown_column");
    assert_eq!(errors[0].field_name.as_deref(), Some("agency_color"));
    assert_eq!(errors[0].file_name.as_deref(), Some("agency.txt"));
    assert_eq!(errors[0].severity, Severity::Info);
}

#[test]
fn unknown_column_none() {
    let source = zip_source_with_content(&[(
        GtfsFiles::Agency,
        b"agency_id,agency_name,agency_url,agency_timezone\n1,Test,http://x,UTC",
    )]);
    let errors = UnknownColumnRule.validate(&source);
    assert!(errors.is_empty());
}
