use headway::cli::{OutputFormat, render_read_results, render_report};
use headway_core::crud::read::ReadResult;
use headway_core::validation::{Severity, ValidationError, ValidationReport};
use std::fs;
use tempfile::NamedTempFile;

// Helper to create test errors
fn create_test_errors_1() -> Vec<ValidationError> {
    vec![
        ValidationError::new("e1", "1", Severity::Error)
            .message("Stop latitude out of range")
            .file("stops.txt")
            .line(10)
            .field("stop_lat")
            .value("999.0"),
        ValidationError::new("e2", "1", Severity::Error)
            .message("Invalid stop ID")
            .file("stops.txt")
            .line(15),
        ValidationError::new("w1", "2", Severity::Warning)
            .message("Deprecated field usage")
            .file("routes.txt")
            .line(5),
        ValidationError::new("i1", "3", Severity::Info)
            .message("Consider adding route color")
            .file("routes.txt"),
    ]
}

// Test 1: Text format default - grouped by file
#[test]
fn test_text_format_grouped_by_file() {
    let errors = create_test_errors_1();
    let report = ValidationReport::from(errors);

    let temp_file = NamedTempFile::new().unwrap();
    render_report(&report, OutputFormat::Text, Some(temp_file.path())).unwrap();

    let content = fs::read_to_string(temp_file.path()).unwrap();

    // Verify routes.txt errors come before stops.txt (alphabetical order)
    let routes_pos = content.find("routes.txt").unwrap();
    let stops_pos = content.find("stops.txt").unwrap();
    assert!(
        routes_pos < stops_pos,
        "Errors should be grouped by file in alphabetical order"
    );

    // Verify summary shows correct counts
    assert!(content.contains("2 Errors"));
    assert!(content.contains("1 Warning"));
    assert!(content.contains("1 Info"));
    assert!(content.contains("FAIL"));
}

// Test 2: Text format - error without optional context
#[test]
fn test_text_format_error_without_context() {
    let errors =
        vec![ValidationError::new("r1", "1", Severity::Error).message("Missing required file")];
    let report = ValidationReport::from(errors);

    let temp_file = NamedTempFile::new().unwrap();
    render_report(&report, OutputFormat::Text, Some(temp_file.path())).unwrap();

    let content = fs::read_to_string(temp_file.path()).unwrap();

    // Should display error without file/line context
    assert!(content.contains("[ERROR]"));
    assert!(content.contains("r1"));
    assert!(content.contains("Missing required file"));
}

// Test 3: Text format - error with full context
#[test]
fn test_text_format_error_with_full_context() {
    let errors = vec![
        ValidationError::new("r1", "1", Severity::Error)
            .message("Invalid latitude")
            .file("stops.txt")
            .line(42)
            .field("stop_lat")
            .value("999.0"),
    ];
    let report = ValidationReport::from(errors);

    let temp_file = NamedTempFile::new().unwrap();
    render_report(&report, OutputFormat::Text, Some(temp_file.path())).unwrap();

    let content = fs::read_to_string(temp_file.path()).unwrap();

    // Should display full context
    assert!(content.contains("stops.txt:42"));
    assert!(content.contains("stop_lat = 999.0"));
}

// Test 4: JSON format
#[test]
fn test_json_format() {
    let errors = create_test_errors_1();
    let report = ValidationReport::from(errors);

    let temp_file = NamedTempFile::new().unwrap();
    render_report(&report, OutputFormat::Json, Some(temp_file.path())).unwrap();

    let content = fs::read_to_string(temp_file.path()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Verify structure
    assert!(json.get("errors").is_some());
    assert!(json.get("summary").is_some());

    let errors = json["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 4);

    let summary = &json["summary"];
    assert_eq!(summary["error_count"], 2);
    assert_eq!(summary["warning_count"], 1);
    assert_eq!(summary["info_count"], 1);
    assert_eq!(summary["passed"], false);
}

// Test 5: JSON format - empty report
#[test]
fn test_json_format_empty_report() {
    let errors: Vec<ValidationError> = vec![];
    let report = ValidationReport::from(errors);

    let temp_file = NamedTempFile::new().unwrap();
    render_report(&report, OutputFormat::Json, Some(temp_file.path())).unwrap();

    let content = fs::read_to_string(temp_file.path()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    let errors = json["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 0);

    let summary = &json["summary"];
    assert_eq!(summary["error_count"], 0);
    assert_eq!(summary["warning_count"], 0);
    assert_eq!(summary["info_count"], 0);
    assert_eq!(summary["passed"], true);
}

// Test 6: File writing
#[test]
fn test_file_writing() {
    let errors = create_test_errors_1();
    let report = ValidationReport::from(errors);

    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();

    render_report(&report, OutputFormat::Json, Some(&path)).unwrap();

    // Verify file exists and is parsable
    let content = fs::read_to_string(&path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(json.get("errors").is_some());
}

// Test 7: File writing - nonexistent directory
#[test]
fn test_file_writing_nonexistent_directory() {
    let errors = create_test_errors_1();
    let report = ValidationReport::from(errors);

    let bad_path = std::path::PathBuf::from("/nonexistent/dir/report.json");
    let result = render_report(&report, OutputFormat::Json, Some(&bad_path));

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Cannot write to"));
    assert!(err_msg.contains("/nonexistent/dir/report.json"));
}

// Test 8: TTY detection - pipe (file output has no ANSI codes)
#[test]
fn test_tty_detection_pipe() {
    let errors = vec![ValidationError::new("e1", "1", Severity::Error).message("Test error")];
    let report = ValidationReport::from(errors);

    let temp_file = NamedTempFile::new().unwrap();
    render_report(&report, OutputFormat::Text, Some(temp_file.path())).unwrap();

    let content = fs::read_to_string(temp_file.path()).unwrap();

    // Should not contain ANSI escape codes
    assert!(!content.contains("\x1b["));
}

// Test 9: Format not supported - XML
#[test]
fn test_format_xml_not_supported() {
    let errors = create_test_errors_1();
    let report = ValidationReport::from(errors);

    let result = render_report(&report, OutputFormat::Xml, None);

    assert!(result.is_err());
    // The error message is printed to stderr, but the Result is Err
}

// Test 10: Format not supported - CSV
#[test]
fn test_format_csv_not_supported() {
    let errors = create_test_errors_1();
    let report = ValidationReport::from(errors);

    let result = render_report(&report, OutputFormat::Csv, None);

    assert!(result.is_err());
}

// Test 11: Summary PASS - only warnings
#[test]
fn test_summary_pass_with_warnings() {
    let errors = vec![
        ValidationError::new("w1", "2", Severity::Warning).message("Warning 1"),
        ValidationError::new("w2", "2", Severity::Warning).message("Warning 2"),
    ];
    let report = ValidationReport::from(errors);

    let temp_file = NamedTempFile::new().unwrap();
    render_report(&report, OutputFormat::Text, Some(temp_file.path())).unwrap();

    let content = fs::read_to_string(temp_file.path()).unwrap();

    assert!(content.contains("PASS"));
    assert!(!content.contains("FAIL"));
}

// Test 16: Summary FAIL - has errors
#[test]
fn test_summary_fail_with_errors() {
    let errors = vec![
        ValidationError::new("e1", "1", Severity::Error).message("Error 1"),
        ValidationError::new("w1", "2", Severity::Warning).message("Warning 1"),
    ];
    let report = ValidationReport::from(errors);

    let temp_file = NamedTempFile::new().unwrap();
    render_report(&report, OutputFormat::Text, Some(temp_file.path())).unwrap();

    let content = fs::read_to_string(temp_file.path()).unwrap();

    assert!(content.contains("FAIL"));
    assert!(!content.contains("PASS"));
}

// Test 17: Grouping by file
#[test]
fn test_grouping_by_file() {
    let errors = vec![
        ValidationError::new("e1", "1", Severity::Error)
            .message("Error in stops")
            .file("stops.txt"),
        ValidationError::new("e2", "1", Severity::Error)
            .message("Another error in stops")
            .file("stops.txt"),
        ValidationError::new("e3", "1", Severity::Error)
            .message("Error in stops again")
            .file("stops.txt"),
        ValidationError::new("e4", "1", Severity::Error)
            .message("Error in trips")
            .file("trips.txt"),
        ValidationError::new("e5", "1", Severity::Error)
            .message("Another error in trips")
            .file("trips.txt"),
    ];
    let report = ValidationReport::from(errors);

    let temp_file = NamedTempFile::new().unwrap();
    render_report(&report, OutputFormat::Text, Some(temp_file.path())).unwrap();

    let content = fs::read_to_string(temp_file.path()).unwrap();

    // Find all positions of file references
    let stops_positions: Vec<usize> = content.match_indices("stops.txt").map(|(i, _)| i).collect();
    let trips_positions: Vec<usize> = content.match_indices("trips.txt").map(|(i, _)| i).collect();

    // All stops.txt references should come before all trips.txt references
    // (because stops comes after trips alphabetically)
    if !trips_positions.is_empty() && !stops_positions.is_empty() {
        let max_stops = stops_positions.iter().max().unwrap();
        let min_trips = trips_positions.iter().min().unwrap();
        assert!(
            max_stops > min_trips,
            "stops.txt errors should come after trips.txt errors (alphabetical)"
        );
    }
}

// ===========================================================================
// Read results rendering
// ===========================================================================

fn sample_read_result() -> ReadResult {
    ReadResult {
        headers: vec!["stop_id", "stop_name", "stop_lat"],
        rows: vec![
            vec![
                Some("S1".into()),
                Some("Gare A".into()),
                Some("48.85".into()),
            ],
            vec![Some("S2".into()), None, Some("48.86".into())],
        ],
        file_name: "stops.txt",
    }
}

#[test]
fn read_text_table_output() {
    let result = sample_read_result();
    let temp = NamedTempFile::new().unwrap();

    render_read_results(&result, OutputFormat::Text, Some(temp.path())).unwrap();
    let content = fs::read_to_string(temp.path()).unwrap();

    assert!(content.contains("stop_id"));
    assert!(content.contains("stop_name"));
    assert!(content.contains("Gare A"));
    assert!(content.contains("S2"));
    assert!(content.contains("Found 2 records in stops.txt"));
}

#[test]
fn read_text_zero_records() {
    let result = ReadResult {
        headers: vec!["stop_id"],
        rows: vec![],
        file_name: "stops.txt",
    };
    let temp = NamedTempFile::new().unwrap();

    render_read_results(&result, OutputFormat::Text, Some(temp.path())).unwrap();
    let content = fs::read_to_string(temp.path()).unwrap();

    assert!(content.contains("0 records found"));
    assert!(!content.contains("Found"));
}

#[test]
fn read_json_valid_array() {
    let result = sample_read_result();
    let temp = NamedTempFile::new().unwrap();

    render_read_results(&result, OutputFormat::Json, Some(temp.path())).unwrap();
    let content = fs::read_to_string(temp.path()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 2);

    assert_eq!(arr[0]["stop_id"], "S1");
    assert_eq!(arr[0]["stop_name"], "Gare A");
    assert_eq!(arr[1]["stop_name"], serde_json::Value::Null);
}

#[test]
fn read_json_empty_array() {
    let result = ReadResult {
        headers: vec!["stop_id"],
        rows: vec![],
        file_name: "stops.txt",
    };
    let temp = NamedTempFile::new().unwrap();

    render_read_results(&result, OutputFormat::Json, Some(temp.path())).unwrap();
    let content = fs::read_to_string(temp.path()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(json.as_array().unwrap().len(), 0);
}

#[test]
fn read_file_output() {
    let result = sample_read_result();
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_path_buf();

    render_read_results(&result, OutputFormat::Json, Some(&path)).unwrap();

    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();
    assert!(!content.is_empty());
    serde_json::from_str::<serde_json::Value>(&content).unwrap();
}
