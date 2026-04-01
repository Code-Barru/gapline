//! Text quality validations: invalid characters, non-ASCII, mixed case.

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

#[must_use]
pub fn has_invalid_chars(value: &str) -> bool {
    value
        .bytes()
        .any(|b| b == 0 || (b < 0x20 && b != b'\t' && b != b'\n' && b != b'\r'))
}

#[must_use]
pub fn has_non_ascii_or_non_printable(value: &str) -> bool {
    value.chars().any(|c| {
        if c == '\t' || c == '\n' || c == '\r' {
            return false;
        }
        c.is_control()
    })
}

#[must_use]
pub fn is_poorly_cased(value: &str) -> bool {
    if value.len() < 2 {
        return false;
    }
    let alpha_chars: Vec<char> = value.chars().filter(|c| c.is_alphabetic()).collect();
    if alpha_chars.len() < 2 {
        return false;
    }
    alpha_chars.iter().all(|c| c.is_uppercase()) || alpha_chars.iter().all(|c| c.is_lowercase())
}

fn collect_text_fields(feed: &GtfsFeed) -> Vec<(&'static str, &'static str, &str, usize)> {
    let mut fields = Vec::new();

    for (i, a) in feed.agencies.iter().enumerate() {
        let line = i + 2;
        fields.push(("agency.txt", "agency_name", a.agency_name.as_str(), line));
    }

    for (i, s) in feed.stops.iter().enumerate() {
        let line = i + 2;
        if let Some(ref name) = s.stop_name {
            fields.push(("stops.txt", "stop_name", name.as_str(), line));
        }
        if let Some(ref desc) = s.stop_desc {
            fields.push(("stops.txt", "stop_desc", desc.as_str(), line));
        }
    }

    for (i, r) in feed.routes.iter().enumerate() {
        let line = i + 2;
        if let Some(ref name) = r.route_short_name {
            fields.push(("routes.txt", "route_short_name", name.as_str(), line));
        }
        if let Some(ref name) = r.route_long_name {
            fields.push(("routes.txt", "route_long_name", name.as_str(), line));
        }
        if let Some(ref desc) = r.route_desc {
            fields.push(("routes.txt", "route_desc", desc.as_str(), line));
        }
    }

    for (i, t) in feed.trips.iter().enumerate() {
        let line = i + 2;
        if let Some(ref hs) = t.trip_headsign {
            fields.push(("trips.txt", "trip_headsign", hs.as_str(), line));
        }
    }

    for (i, st) in feed.stop_times.iter().enumerate() {
        let line = i + 2;
        if let Some(ref hs) = st.stop_headsign {
            fields.push(("stop_times.txt", "stop_headsign", hs.as_str(), line));
        }
    }

    fields
}

const MIXED_CASE_FIELDS: &[(&str, &str); 4] = &[
    ("stops.txt", "stop_name"),
    ("routes.txt", "route_long_name"),
    ("trips.txt", "trip_headsign"),
    ("stop_times.txt", "stop_headsign"),
];

pub struct TextValidator;

impl ValidationRule for TextValidator {
    fn rule_id(&self) -> &'static str {
        "text_validator"
    }

    fn section(&self) -> &'static str {
        "3"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let fields = collect_text_fields(feed);

        for (file, field, value, line) in &fields {
            if has_invalid_chars(value) {
                errors.push(
                    ValidationError::new("invalid_character", "3", Severity::Error)
                        .file(*file)
                        .field(*field)
                        .value(*value)
                        .line(*line)
                        .message(format!("Invalid character in {field}: '{value}'")),
                );
            }

            if has_non_ascii_or_non_printable(value) {
                errors.push(
                    ValidationError::new("non_ascii_or_non_printable_char", "3", Severity::Warning)
                        .file(*file)
                        .field(*field)
                        .value(*value)
                        .line(*line)
                        .message(format!(
                            "Non-ASCII or non-printable character in {field}: '{value}'"
                        )),
                );
            }

            if MIXED_CASE_FIELDS.contains(&(*file, *field)) && is_poorly_cased(value) {
                errors.push(
                    ValidationError::new("mixed_case_recommended_field", "3", Severity::Warning)
                        .file(*file)
                        .field(*field)
                        .value(*value)
                        .line(*line)
                        .message(format!(
                            "Field {field} is all uppercase or all lowercase, mixed case recommended: '{value}'"
                        )),
                );
            }
        }

        errors
    }
}
