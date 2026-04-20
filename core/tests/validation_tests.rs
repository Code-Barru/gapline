use gapline_core::models::GtfsFeed;
use gapline_core::validation::{Severity, ValidationError, ValidationReport, ValidationRule};

// ---------- Test 1 : Minimal builder ----------

#[test]
fn test_builder_minimal() {
    let error = ValidationError::new("test_rule", "1", Severity::Error);
    let json: serde_json::Value = serde_json::to_value(&error).unwrap();

    assert_eq!(json["rule_id"], "test_rule");
    assert_eq!(json["section"], "1");
    assert_eq!(json["severity"], "error");
    assert_eq!(json["message"], "");
    assert!(json["file_name"].is_null());
    assert!(json["line_number"].is_null());
    assert!(json["field_name"].is_null());
    assert!(json["value"].is_null());
}

// ---------- Test 2 : Complete builder ----------

#[test]
fn test_builder_complete() {
    let error = ValidationError::new("test_rule", "1", Severity::Error)
        .message("bad")
        .file("stops.txt")
        .line(42)
        .field("stop_id")
        .value("XYZ");

    let json: serde_json::Value = serde_json::to_value(&error).unwrap();

    assert_eq!(json["rule_id"], "test_rule");
    assert_eq!(json["section"], "1");
    assert_eq!(json["severity"], "error");
    assert_eq!(json["message"], "bad");
    assert_eq!(json["file_name"], "stops.txt");
    assert_eq!(json["line_number"], 42);
    assert_eq!(json["field_name"], "stop_id");
    assert_eq!(json["value"], "XYZ");
}

// ---------- Test 3 : Builder chaining ----------

#[test]
fn test_builder_chaining() {
    let _error = ValidationError::new("rule", "1", Severity::Warning)
        .message("a")
        .file("b")
        .line(1)
        .field("c")
        .value("d");
}

// ---------- Test 4 : JSON serialization ----------

#[test]
fn test_json_serialization() {
    let error = ValidationError::new("json_rule", "2", Severity::Warning)
        .message("missing field")
        .file("routes.txt")
        .line(10)
        .field("route_id")
        .value("");

    let json_string = serde_json::to_string(&error).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&json_string).unwrap();
    assert!(parsed.is_object());

    assert_eq!(parsed["rule_id"], "json_rule");
    assert_eq!(parsed["severity"], "warning");
    assert_eq!(parsed["message"], "missing field");

    let minimal = ValidationError::new("min", "0", Severity::Info);
    let minimal_json: serde_json::Value = serde_json::to_value(&minimal).unwrap();
    assert!(minimal_json["file_name"].is_null());
    assert!(minimal_json["line_number"].is_null());
    assert!(minimal_json["field_name"].is_null());
    assert!(minimal_json["value"].is_null());
}

// ---------- Test 5 : Severity ordering ----------

#[test]
fn test_severity_ordering() {
    assert!(Severity::Error > Severity::Warning);
    assert!(Severity::Warning > Severity::Info);
    assert!(Severity::Error > Severity::Info);
}

// ---------- Test 6 : Severity Display ----------

#[test]
fn test_severity_display() {
    assert_eq!(format!("{}", Severity::Error), "ERROR");
    assert_eq!(format!("{}", Severity::Warning), "WARNING");
    assert_eq!(format!("{}", Severity::Info), "INFO");
}

// ---------- Test 7 : Report counts ----------

#[test]
fn test_report_counts() {
    let errors = vec![
        ValidationError::new("r1", "1", Severity::Error),
        ValidationError::new("r2", "1", Severity::Error),
        ValidationError::new("r3", "2", Severity::Warning),
        ValidationError::new("r4", "2", Severity::Warning),
        ValidationError::new("r5", "2", Severity::Warning),
        ValidationError::new("r6", "3", Severity::Info),
    ];

    let report = ValidationReport::from(errors);

    assert_eq!(report.error_count(), 2);
    assert_eq!(report.warning_count(), 3);
    assert_eq!(report.info_count(), 1);
}

// ---------- Test 8 : Report has_errors (false) ----------

#[test]
fn test_report_has_errors_false() {
    let errors = vec![
        ValidationError::new("r1", "1", Severity::Warning),
        ValidationError::new("r2", "1", Severity::Info),
    ];

    let report = ValidationReport::from(errors);

    assert!(!report.has_errors());
}

// ---------- Test 9 : Report has_errors (true) ----------

#[test]
fn test_report_has_errors_true() {
    let errors = vec![ValidationError::new("r1", "1", Severity::Error)];

    let report = ValidationReport::from(errors);

    assert!(report.has_errors());
}

// ---------- Test 10 : ValidationRule trait ----------

struct DummyRule;

impl ValidationRule for DummyRule {
    fn rule_id(&self) -> &'static str {
        "dummy_rule"
    }

    fn section(&self) -> &'static str {
        "99"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, _feed: &GtfsFeed) -> Vec<ValidationError> {
        vec![
            ValidationError::new(self.rule_id(), self.section(), self.severity())
                .message("dummy validation error"),
        ]
    }
}

#[test]
fn test_validation_rule_trait() {
    let rule = DummyRule;
    let feed = GtfsFeed::default();

    let results = rule.validate(&feed);

    assert_eq!(results.len(), 1);

    let json: serde_json::Value = serde_json::to_value(&results[0]).unwrap();
    assert_eq!(json["rule_id"], "dummy_rule");
    assert_eq!(json["section"], "99");
    assert_eq!(json["severity"], "warning");
    assert_eq!(json["message"], "dummy validation error");
}
