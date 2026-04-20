use gapline::cli::{OutputFormat, render_read_results, render_report};
use gapline_core::config::Config;
use gapline_core::crud::read::ReadResult;
use gapline_core::validation::{Severity, ValidationError, ValidationReport};
use std::fs;
use std::path::Path;
use tempfile::NamedTempFile;

/// Default test config used by all `render_report` invocations in this file.
/// Pre-bound `min_severity = None` so existing assertions about counts and
/// content remain unchanged.
fn test_config() -> Config {
    Config::default()
}

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
    render_report(
        &report,
        OutputFormat::Text,
        Path::new("test_feed.zip"),
        Some(temp_file.path()),
        &test_config(),
    )
    .unwrap();

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
    render_report(
        &report,
        OutputFormat::Text,
        Path::new("test_feed.zip"),
        Some(temp_file.path()),
        &test_config(),
    )
    .unwrap();

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
    render_report(
        &report,
        OutputFormat::Text,
        Path::new("test_feed.zip"),
        Some(temp_file.path()),
        &test_config(),
    )
    .unwrap();

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
    render_report(
        &report,
        OutputFormat::Json,
        Path::new("test_feed.zip"),
        Some(temp_file.path()),
        &test_config(),
    )
    .unwrap();

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
    render_report(
        &report,
        OutputFormat::Json,
        Path::new("test_feed.zip"),
        Some(temp_file.path()),
        &test_config(),
    )
    .unwrap();

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

    render_report(
        &report,
        OutputFormat::Json,
        Path::new("test_feed.zip"),
        Some(&path),
        &test_config(),
    )
    .unwrap();

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
    let result = render_report(
        &report,
        OutputFormat::Json,
        Path::new("test_feed.zip"),
        Some(&bad_path),
        &test_config(),
    );

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
    render_report(
        &report,
        OutputFormat::Text,
        Path::new("test_feed.zip"),
        Some(temp_file.path()),
        &test_config(),
    )
    .unwrap();

    let content = fs::read_to_string(temp_file.path()).unwrap();

    // Should not contain ANSI escape codes
    assert!(!content.contains("\x1b["));
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
    render_report(
        &report,
        OutputFormat::Text,
        Path::new("test_feed.zip"),
        Some(temp_file.path()),
        &test_config(),
    )
    .unwrap();

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
    render_report(
        &report,
        OutputFormat::Text,
        Path::new("test_feed.zip"),
        Some(temp_file.path()),
        &test_config(),
    )
    .unwrap();

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
    render_report(
        &report,
        OutputFormat::Text,
        Path::new("test_feed.zip"),
        Some(temp_file.path()),
        &test_config(),
    )
    .unwrap();

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

fn render_validation_to_string(report: &ValidationReport, format: OutputFormat) -> String {
    let temp = NamedTempFile::new().unwrap();
    render_report(
        report,
        format,
        Path::new("test_feed.zip"),
        Some(temp.path()),
        &test_config(),
    )
    .unwrap();
    fs::read_to_string(temp.path()).unwrap()
}

fn render_read_to_string(result: &ReadResult, format: OutputFormat) -> String {
    let temp = NamedTempFile::new().unwrap();
    render_read_results(result, format, Some(temp.path())).unwrap();
    fs::read_to_string(temp.path()).unwrap()
}

#[test]
fn validate_csv_has_expected_headers() {
    let report = ValidationReport::from(create_test_errors_1());
    let content = render_validation_to_string(&report, OutputFormat::Csv);
    let mut lines = content.lines();
    let header = lines.next().unwrap();
    assert_eq!(
        header,
        "rule_id,section,severity,message,file_name,line_number,field_name,value"
    );
}

#[test]
fn validate_csv_one_row_per_error() {
    let report = ValidationReport::from(create_test_errors_1());
    let content = render_validation_to_string(&report, OutputFormat::Csv);
    let mut rdr = csv::Reader::from_reader(content.as_bytes());
    let rows: Vec<_> = rdr.records().map(|r| r.unwrap()).collect();
    assert_eq!(rows.len(), 4);
}

#[test]
fn validate_csv_escapes_commas_and_quotes() {
    let errors = vec![
        ValidationError::new("e1", "1", Severity::Error)
            .message("foo, bar \"baz\"")
            .file("stops.txt"),
    ];
    let report = ValidationReport::from(errors);
    let content = render_validation_to_string(&report, OutputFormat::Csv);

    let mut rdr = csv::Reader::from_reader(content.as_bytes());
    let rec = rdr.records().next().unwrap().unwrap();
    assert_eq!(&rec[3], "foo, bar \"baz\"");
}

#[test]
fn validate_csv_none_fields_are_empty_string() {
    let errors = vec![ValidationError::new("e1", "1", Severity::Error).message("no line number")];
    let report = ValidationReport::from(errors);
    let content = render_validation_to_string(&report, OutputFormat::Csv);
    let mut rdr = csv::Reader::from_reader(content.as_bytes());
    let rec = rdr.records().next().unwrap().unwrap();
    assert_eq!(&rec[4], "");
    assert_eq!(&rec[5], "");
    assert_eq!(&rec[6], "");
    assert_eq!(&rec[7], "");
    assert!(!content.contains("null"));
}

#[test]
fn validate_csv_zero_errors_emits_header_only() {
    let report = ValidationReport::from(vec![]);
    let content = render_validation_to_string(&report, OutputFormat::Csv);
    let lines: Vec<_> = content.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 1);
    assert!(lines[0].starts_with("rule_id,"));
}

#[test]
fn validate_xml_is_well_formed() {
    let report = ValidationReport::from(create_test_errors_1());
    let content = render_validation_to_string(&report, OutputFormat::Xml);
    let mut rdr = quick_xml::Reader::from_str(&content);
    loop {
        match rdr.read_event() {
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {}
            Err(e) => panic!("malformed XML: {e}"),
        }
    }
}

#[test]
fn validate_xml_contains_all_fields() {
    let errors = vec![
        ValidationError::new("e1", "1", Severity::Error)
            .message("Invalid lat")
            .file("stops.txt")
            .line(42)
            .field("stop_lat")
            .value("999.0"),
    ];
    let report = ValidationReport::from(errors);
    let content = render_validation_to_string(&report, OutputFormat::Xml);
    assert!(content.contains("<rule_id>e1</rule_id>"));
    assert!(content.contains("<section>1</section>"));
    assert!(content.contains("<severity>error</severity>"));
    assert!(content.contains("<message>Invalid lat</message>"));
    assert!(content.contains("<file_name>stops.txt</file_name>"));
    assert!(content.contains("<line_number>42</line_number>"));
    assert!(content.contains("<field_name>stop_lat</field_name>"));
    assert!(content.contains("<value>999.0</value>"));
}

#[test]
fn validate_xml_is_pretty_printed() {
    let report = ValidationReport::from(create_test_errors_1());
    let content = render_validation_to_string(&report, OutputFormat::Xml);
    assert!(content.contains('\n'));
    assert!(content.contains("  <"));
}

#[test]
fn validate_xml_zero_errors_has_valid_root() {
    let report = ValidationReport::from(vec![]);
    let content = render_validation_to_string(&report, OutputFormat::Xml);
    assert!(content.contains("<validation_report>"));
    assert!(content.contains("</validation_report>"));
    assert!(content.contains("<error_count>0</error_count>"));
}

#[test]
fn read_csv_headers_match_result() {
    let result = sample_read_result();
    let content = render_read_to_string(&result, OutputFormat::Csv);
    let mut rdr = csv::Reader::from_reader(content.as_bytes());
    let headers = rdr.headers().unwrap().clone();
    let got: Vec<&str> = headers.iter().collect();
    assert_eq!(got, vec!["stop_id", "stop_name", "stop_lat"]);
}

#[test]
fn read_csv_none_fields_are_empty_string() {
    let result = sample_read_result();
    let content = render_read_to_string(&result, OutputFormat::Csv);
    let mut rdr = csv::Reader::from_reader(content.as_bytes());
    let rows: Vec<_> = rdr.records().map(|r| r.unwrap()).collect();
    assert_eq!(&rows[1][1], "");
    assert!(!content.contains("null"));
}

#[test]
fn read_csv_empty_result_emits_headers_only() {
    let result = ReadResult {
        headers: vec!["stop_id", "stop_name"],
        rows: vec![],
        file_name: "stops.txt",
    };
    let content = render_read_to_string(&result, OutputFormat::Csv);
    let lines: Vec<_> = content.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "stop_id,stop_name");
}

#[test]
fn read_xml_is_well_formed() {
    let result = sample_read_result();
    let content = render_read_to_string(&result, OutputFormat::Xml);
    let mut rdr = quick_xml::Reader::from_str(&content);
    loop {
        match rdr.read_event() {
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {}
            Err(e) => panic!("malformed XML: {e}"),
        }
    }
}

#[test]
fn read_xml_root_has_file_attr() {
    let result = sample_read_result();
    let content = render_read_to_string(&result, OutputFormat::Xml);
    assert!(content.contains(r#"<records file="stops.txt">"#));
    assert!(content.contains("<record>"));
    assert!(content.contains(r#"<field name="stop_id">S1</field>"#));
}

#[test]
fn read_xml_none_fields_are_empty_elements() {
    let result = sample_read_result();
    let content = render_read_to_string(&result, OutputFormat::Xml);
    assert!(content.contains(r#"<field name="stop_name"/>"#));
}

#[test]
fn read_xml_pretty_printed() {
    let result = sample_read_result();
    let content = render_read_to_string(&result, OutputFormat::Xml);
    assert!(content.contains('\n'));
    assert!(content.contains("  <record>"));
}

#[test]
fn validate_xml_written_to_file() {
    let report = ValidationReport::from(create_test_errors_1());
    let temp = NamedTempFile::new().unwrap();
    render_report(
        &report,
        OutputFormat::Xml,
        Path::new("test_feed.zip"),
        Some(temp.path()),
        &test_config(),
    )
    .unwrap();
    let content = fs::read_to_string(temp.path()).unwrap();
    assert!(content.starts_with("<?xml"));
}

#[test]
fn validate_csv_written_to_file() {
    let report = ValidationReport::from(create_test_errors_1());
    let temp = NamedTempFile::new().unwrap();
    render_report(
        &report,
        OutputFormat::Csv,
        Path::new("test_feed.zip"),
        Some(temp.path()),
        &test_config(),
    )
    .unwrap();
    let content = fs::read_to_string(temp.path()).unwrap();
    assert!(content.starts_with("rule_id,"));
}

#[test]
fn read_csv_written_to_file() {
    let result = sample_read_result();
    let temp = NamedTempFile::new().unwrap();
    render_read_results(&result, OutputFormat::Csv, Some(temp.path())).unwrap();
    let content = fs::read_to_string(temp.path()).unwrap();
    assert!(content.starts_with("stop_id,"));
}

#[test]
fn read_xml_written_to_file() {
    let result = sample_read_result();
    let temp = NamedTempFile::new().unwrap();
    render_read_results(&result, OutputFormat::Xml, Some(temp.path())).unwrap();
    let content = fs::read_to_string(temp.path()).unwrap();
    assert!(content.starts_with("<?xml"));
}

// ===========================================================================
// HTML format
// ===========================================================================

#[test]
fn validate_html_contains_summary_and_verdict_fail() {
    let report = ValidationReport::from(create_test_errors_1());
    let content = render_validation_to_string(&report, OutputFormat::Html);

    assert!(content.starts_with("<!DOCTYPE html>"));
    assert!(content.contains(r#"id="summary""#));
    assert!(content.contains(r#"id="verdict""#));
    assert!(content.contains("FAIL"));
    assert!(content.contains("verdict fail"));
    assert!(content.contains(r#"<span class="n">2</span> errors"#));
    assert!(content.contains(r#"<span class="n">1</span> warnings"#));
    assert!(content.contains(r#"<span class="n">1</span> infos"#));
}

#[test]
fn validate_html_empty_report_shows_pass() {
    let report = ValidationReport::from(vec![]);
    let content = render_validation_to_string(&report, OutputFormat::Html);

    assert!(content.contains("PASS"));
    assert!(content.contains("No issues found"));
    assert!(content.contains("verdict pass"));
    assert!(!content.contains(r#"class="row row-error""#));
    assert!(content.contains("empty-state"));
}

#[test]
fn validate_html_groups_errors_by_file_with_counts() {
    let errors = vec![
        ValidationError::new("e1", "1", Severity::Error)
            .message("a")
            .file("stops.txt"),
        ValidationError::new("e2", "1", Severity::Error)
            .message("b")
            .file("stops.txt"),
        ValidationError::new("e3", "1", Severity::Error)
            .message("c")
            .file("stops.txt"),
        ValidationError::new("e4", "1", Severity::Error)
            .message("d")
            .file("trips.txt"),
        ValidationError::new("e5", "1", Severity::Error)
            .message("e")
            .file("trips.txt"),
    ];
    let report = ValidationReport::from(errors);
    let content = render_validation_to_string(&report, OutputFormat::Html);

    assert!(content.contains(r#"data-file-count="3""#));
    assert!(content.contains(r#"data-file-count="2""#));
    let groups = content.matches(r#"class="file-group""#).count();
    assert_eq!(groups, 2);
}

#[test]
fn validate_html_escapes_user_values_against_xss() {
    let errors = vec![
        ValidationError::new("e1", "1", Severity::Error)
            .message("boom")
            .file("stops.txt")
            .line(1)
            .field("stop_name")
            .value("<script>alert(1)</script>"),
    ];
    let report = ValidationReport::from(errors);
    let content = render_validation_to_string(&report, OutputFormat::Html);

    assert!(content.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(!content.contains("<script>alert(1)</script>"));
}

#[test]
fn validate_html_includes_metadata_header() {
    let report = ValidationReport::from(create_test_errors_1());
    let content = render_validation_to_string(&report, OutputFormat::Html);

    assert!(content.contains("test_feed.zip"));
    assert!(content.contains(&format!("gapline v{}", env!("CARGO_PKG_VERSION"))));
    // Generated timestamp: %Y-%m-%d %H:%M:%S — check for the "20" century prefix
    // next to the "Generated:" label
    let gen_pos = content.find("Generated:").expect("Generated label present");
    let tail = &content[gen_pos..gen_pos + 40];
    assert!(
        tail.contains("20"),
        "expected a year-like token near Generated label, got: {tail}"
    );
}

#[test]
fn validate_html_contains_severity_toggles() {
    let report = ValidationReport::from(create_test_errors_1());
    let content = render_validation_to_string(&report, OutputFormat::Html);

    assert!(content.contains(r#"id="toggle-error""#));
    assert!(content.contains(r#"id="toggle-warning""#));
    assert!(content.contains(r#"id="toggle-info""#));
    assert!(content.contains("hide-error"));
}

#[test]
fn validate_html_writes_to_file_and_is_parseable() {
    let report = ValidationReport::from(create_test_errors_1());
    let temp = NamedTempFile::new().unwrap();
    render_report(
        &report,
        OutputFormat::Html,
        Path::new("my_feed.zip"),
        Some(temp.path()),
        &test_config(),
    )
    .unwrap();
    let content = fs::read_to_string(temp.path()).unwrap();
    assert!(content.starts_with("<!DOCTYPE html>"));
    assert!(content.ends_with("</html>\n") || content.ends_with("</html>"));
    assert!(content.contains("my_feed.zip"));
}

#[test]
fn parser_accepts_html_format() {
    assert!(matches!(
        OutputFormat::from_config_str("html"),
        Some(OutputFormat::Html)
    ));
    assert!(matches!(
        OutputFormat::from_config_str("HTML"),
        Some(OutputFormat::Html)
    ));
}

#[test]
fn read_html_renders_table_with_escaped_values() {
    let result = ReadResult {
        headers: vec!["stop_id", "stop_name"],
        rows: vec![
            vec![Some("S1".into()), Some("A&B <x>".into())],
            vec![Some("S2".into()), None],
        ],
        file_name: "stops.txt",
    };
    let content = render_read_to_string(&result, OutputFormat::Html);

    assert!(content.starts_with("<!DOCTYPE html>"));
    assert!(content.contains("<table>"));
    assert!(content.contains("<th>stop_id</th>"));
    assert!(content.contains("<th>stop_name</th>"));
    assert!(content.contains("A&amp;B &lt;x&gt;"));
    assert!(!content.contains("A&B <x>"));
    assert!(content.contains(r#"class="empty""#));
    assert!(content.contains("Found 2 records in stops.txt"));
}

#[test]
fn rules_html_groups_by_stage() {
    use gapline::cli::{RuleEntry, Stage, render_rules_list};
    let entries = vec![
        RuleEntry::new("struct_rule_a", Severity::Error, Stage::Structural),
        RuleEntry::new("sem_rule_a", Severity::Warning, Stage::Semantic),
        RuleEntry::new("sem_rule_b", Severity::Info, Stage::Semantic),
    ];
    let temp = NamedTempFile::new().unwrap();
    render_rules_list(&entries, OutputFormat::Html, Some(temp.path())).unwrap();
    let content = fs::read_to_string(temp.path()).unwrap();

    assert!(content.starts_with("<!DOCTYPE html>"));
    assert!(content.contains("structural (1)"));
    assert!(content.contains("semantic (2)"));
    assert!(content.contains("<code>struct_rule_a</code>"));
    assert!(content.contains("<code>sem_rule_a</code>"));
    assert!(content.contains(r#"class="sev error""#));
    assert!(content.contains(r#"class="sev warning""#));
    assert!(content.contains(r#"class="sev info""#));
}
