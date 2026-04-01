//! Tests for section 3 — Field Type Validation.

use headway_core::models::*;
use headway_core::parser::error::{ParseError, ParseErrorKind};
use headway_core::validation::field_type::field_type_rules::{
    FieldTypeValidator, is_valid_color, is_valid_currency, is_valid_email, is_valid_language_code,
    is_valid_phone, is_valid_timezone, is_valid_url,
};
use headway_core::validation::field_type::numeric_rules::NumericRangeValidator;
use headway_core::validation::field_type::parse_error_converter;
use headway_core::validation::field_type::text_rules::{
    TextValidator, has_invalid_chars, has_non_ascii_or_non_printable, is_poorly_cased,
};
use headway_core::validation::{Severity, ValidationRule};

// ---------------------------------------------------------------------------
// Pure validation functions
// ---------------------------------------------------------------------------

#[test]
fn valid_url() {
    assert!(is_valid_url("http://stm.info"));
    assert!(is_valid_url("https://stm.info"));
    assert!(!is_valid_url("www.stm.info"));
    assert!(!is_valid_url("ftp://stm.info"));
    assert!(!is_valid_url(""));
}

#[test]
fn valid_timezone() {
    assert!(is_valid_timezone("America/Montreal"));
    assert!(is_valid_timezone("America/New_York"));
    assert!(is_valid_timezone("Europe/Paris"));
    assert!(!is_valid_timezone("US/East"));
    assert!(!is_valid_timezone("Invalid/Zone"));
    assert!(!is_valid_timezone(""));
}

#[test]
fn valid_color() {
    assert!(is_valid_color("00AAFF"));
    assert!(is_valid_color("ffffff"));
    assert!(is_valid_color("000000"));
    assert!(!is_valid_color("#00AAFF"));
    assert!(!is_valid_color("00AA"));
    assert!(!is_valid_color("GGG000"));
    assert!(!is_valid_color(""));
}

#[test]
fn valid_language_code() {
    assert!(is_valid_language_code("fr"));
    assert!(is_valid_language_code("en"));
    assert!(is_valid_language_code("en-US"));
    assert!(is_valid_language_code("zh-Hant"));
    assert!(!is_valid_language_code("french"));
    assert!(!is_valid_language_code(""));
    assert!(!is_valid_language_code("x"));
}

#[test]
fn valid_currency() {
    assert!(is_valid_currency("EUR"));
    assert!(is_valid_currency("USD"));
    assert!(is_valid_currency("CAD"));
    assert!(!is_valid_currency("EURO"));
    assert!(!is_valid_currency("usd"));
    assert!(!is_valid_currency(""));
}

#[test]
fn valid_email() {
    assert!(is_valid_email("test@example.com"));
    assert!(is_valid_email("a@b.c"));
    assert!(!is_valid_email("not-an-email"));
    assert!(!is_valid_email("@example.com"));
    assert!(!is_valid_email(""));
}

#[test]
fn valid_phone() {
    assert!(is_valid_phone("+1-514-555-0100"));
    assert!(is_valid_phone("514-555-0100"));
    assert!(!is_valid_phone(""));
    assert!(!is_valid_phone("   "));
}

#[test]
fn invalid_chars_detection() {
    assert!(has_invalid_chars("hello\x00world"));
    assert!(has_invalid_chars("test\x01"));
    assert!(!has_invalid_chars("hello world"));
    assert!(!has_invalid_chars("with\ttab"));
    assert!(!has_invalid_chars("with\nnewline"));
}

#[test]
fn non_printable_detection() {
    // Accented characters are valid UTF-8 text, not flagged
    assert!(!has_non_ascii_or_non_printable("café"));
    assert!(!has_non_ascii_or_non_printable("Université"));
    assert!(!has_non_ascii_or_non_printable("Dépôt Grenay"));
    assert!(!has_non_ascii_or_non_printable("hello"));
    // Control characters are flagged
    assert!(has_non_ascii_or_non_printable("hello\x00world"));
    assert!(has_non_ascii_or_non_printable("test\x01"));
    // Whitespace exceptions
    assert!(!has_non_ascii_or_non_printable("with\ttab"));
}

#[test]
fn poorly_cased_detection() {
    assert!(is_poorly_cased("GARE DU NORD"));
    assert!(is_poorly_cased("gare du nord"));
    assert!(!is_poorly_cased("Gare du Nord"));
    assert!(!is_poorly_cased("A"));
    assert!(!is_poorly_cased("123"));
}

// ---------------------------------------------------------------------------
// FieldTypeValidator rule tests
// ---------------------------------------------------------------------------

fn make_agency(url: &str, tz: &str) -> Agency {
    Agency {
        agency_id: None,
        agency_name: "Test".to_string(),
        agency_url: url.into(),
        agency_timezone: tz.into(),
        agency_lang: None,
        agency_phone: None,
        agency_fare_url: None,
        agency_email: None,
    }
}

#[test]
fn invalid_url_in_agency() {
    let feed = GtfsFeed {
        agencies: vec![make_agency("www.stm.info", "America/Montreal")],
        ..Default::default()
    };
    let errors = FieldTypeValidator.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "invalid_url");
    assert_eq!(errors[0].field_name.as_deref(), Some("agency_url"));
}

#[test]
fn invalid_timezone_in_agency() {
    let feed = GtfsFeed {
        agencies: vec![make_agency("https://stm.info", "US/East")],
        ..Default::default()
    };
    let errors = FieldTypeValidator.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "invalid_timezone");
}

#[test]
fn invalid_color_in_route() {
    let feed = GtfsFeed {
        routes: vec![Route {
            route_id: "R1".into(),
            agency_id: None,
            route_short_name: None,
            route_long_name: None,
            route_desc: None,
            route_type: RouteType::Bus,
            route_url: None,
            route_color: Some("#00AAFF".into()),
            route_text_color: Some("00AA".into()),
            route_sort_order: None,
            continuous_pickup: None,
            continuous_drop_off: None,
            network_id: None,
        }],
        ..Default::default()
    };
    let errors = FieldTypeValidator.validate(&feed);
    assert_eq!(errors.len(), 2);
    assert!(errors.iter().all(|e| e.rule_id == "invalid_color"));
}

#[test]
fn invalid_currency_in_fare() {
    let feed = GtfsFeed {
        fare_attributes: vec![FareAttribute {
            fare_id: "F1".into(),
            price: 3.50,
            currency_type: "EURO".into(),
            payment_method: 0,
            transfers: None,
            agency_id: None,
            transfer_duration: None,
        }],
        ..Default::default()
    };
    let errors = FieldTypeValidator.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "invalid_currency");
}

#[test]
fn valid_feed_no_field_type_errors() {
    let feed = GtfsFeed {
        agencies: vec![Agency {
            agency_id: None,
            agency_name: "Test".to_string(),
            agency_url: "https://stm.info".into(),
            agency_timezone: "America/Montreal".into(),
            agency_lang: Some("fr".into()),
            agency_phone: Some("+1-514-555-0100".into()),
            agency_fare_url: None,
            agency_email: Some("test@stm.info".into()),
        }],
        ..Default::default()
    };
    let errors = FieldTypeValidator.validate(&feed);
    assert!(errors.is_empty());
}

// ---------------------------------------------------------------------------
// NumericRangeValidator
// ---------------------------------------------------------------------------

fn stop_with_coords(lat: f64, lon: f64) -> Stop {
    Stop {
        stop_id: "S1".into(),
        stop_code: None,
        stop_name: None,
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(lat)),
        stop_lon: Some(Longitude(lon)),
        zone_id: None,
        stop_url: None,
        location_type: None,
        parent_station: None,
        stop_timezone: None,
        wheelchair_boarding: None,
        level_id: None,
        platform_code: None,
    }
}

#[test]
fn valid_coordinates() {
    let feed = GtfsFeed {
        stops: vec![stop_with_coords(45.5, -73.6)],
        ..Default::default()
    };
    assert!(NumericRangeValidator.validate(&feed).is_empty());
}

#[test]
fn latitude_out_of_range() {
    let feed = GtfsFeed {
        stops: vec![stop_with_coords(95.0, -73.6)],
        ..Default::default()
    };
    let errors = NumericRangeValidator.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "number_out_of_range");
    assert_eq!(errors[0].field_name.as_deref(), Some("stop_lat"));
}

#[test]
fn longitude_out_of_range() {
    let feed = GtfsFeed {
        stops: vec![stop_with_coords(45.5, -200.0)],
        ..Default::default()
    };
    let errors = NumericRangeValidator.validate(&feed);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field_name.as_deref(), Some("stop_lon"));
}

#[test]
fn boundary_values_valid() {
    let feed = GtfsFeed {
        stops: vec![
            stop_with_coords(90.0, 180.0),
            stop_with_coords(-90.0, -180.0),
        ],
        ..Default::default()
    };
    assert!(NumericRangeValidator.validate(&feed).is_empty());
}

// ---------------------------------------------------------------------------
// TextValidator
// ---------------------------------------------------------------------------

fn stop_with_name(name: &str) -> Stop {
    Stop {
        stop_id: "S1".into(),
        stop_code: None,
        stop_name: Some(name.to_string()),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: None,
        stop_lon: None,
        zone_id: None,
        stop_url: None,
        location_type: None,
        parent_station: None,
        stop_timezone: None,
        wheelchair_boarding: None,
        level_id: None,
        platform_code: None,
    }
}

#[test]
fn mixed_case_warning() {
    let feed = GtfsFeed {
        stops: vec![stop_with_name("GARE DU NORD")],
        ..Default::default()
    };
    let errors = TextValidator.validate(&feed);
    let mixed = errors
        .iter()
        .filter(|e| e.rule_id == "mixed_case_recommended_field")
        .count();
    assert_eq!(mixed, 1);
}

#[test]
fn proper_case_no_warning() {
    let feed = GtfsFeed {
        stops: vec![stop_with_name("Gare du Nord")],
        ..Default::default()
    };
    let errors = TextValidator.validate(&feed);
    let mixed = errors
        .iter()
        .filter(|e| e.rule_id == "mixed_case_recommended_field")
        .count();
    assert_eq!(mixed, 0);
}

// ---------------------------------------------------------------------------
// ParseError converter
// ---------------------------------------------------------------------------

#[test]
fn converts_invalid_integer() {
    let pe = ParseError {
        file_name: "stop_times.txt".to_string(),
        line_number: 5,
        field_name: "stop_sequence".to_string(),
        value: "abc".to_string(),
        kind: ParseErrorKind::InvalidInteger,
    };
    let errors = parse_error_converter::convert(&[pe]);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule_id, "invalid_integer");
    assert_eq!(errors[0].severity, Severity::Error);
    assert_eq!(errors[0].file_name.as_deref(), Some("stop_times.txt"));
    assert_eq!(errors[0].line_number, Some(5));
    assert_eq!(errors[0].field_name.as_deref(), Some("stop_sequence"));
    assert_eq!(errors[0].value.as_deref(), Some("abc"));
}

#[test]
fn converts_missing_required() {
    let pe = ParseError {
        file_name: "agency.txt".to_string(),
        line_number: 2,
        field_name: "agency_name".to_string(),
        value: String::new(),
        kind: ParseErrorKind::MissingRequired,
    };
    let errors = parse_error_converter::convert(&[pe]);
    assert_eq!(errors[0].rule_id, "missing_required_field");
}

#[test]
fn converts_invalid_enum_as_warning() {
    let pe = ParseError {
        file_name: "routes.txt".to_string(),
        line_number: 3,
        field_name: "route_type".to_string(),
        value: "99".to_string(),
        kind: ParseErrorKind::InvalidEnum,
    };
    let errors = parse_error_converter::convert(&[pe]);
    assert_eq!(errors[0].rule_id, "unexpected_enum_value");
    assert_eq!(errors[0].severity, Severity::Warning);
}

#[test]
fn converts_all_parse_error_kinds() {
    let kinds = [
        (ParseErrorKind::InvalidInteger, "invalid_integer"),
        (ParseErrorKind::InvalidFloat, "invalid_float"),
        (ParseErrorKind::InvalidDate, "invalid_date"),
        (ParseErrorKind::InvalidTime, "invalid_time"),
        (ParseErrorKind::InvalidEnum, "unexpected_enum_value"),
        (ParseErrorKind::MissingRequired, "missing_required_field"),
    ];
    for (kind, expected_rule) in kinds {
        let pe = ParseError {
            file_name: "test.txt".to_string(),
            line_number: 1,
            field_name: "f".to_string(),
            value: "v".to_string(),
            kind,
        };
        let errors = parse_error_converter::convert(&[pe]);
        assert_eq!(errors[0].rule_id, expected_rule);
    }
}
