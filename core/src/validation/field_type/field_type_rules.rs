//! Validates field types: URL, Timezone, Color, Language, Currency, Email, Phone.
//!
//! A single `FieldTypeValidator` iterates over the feed once, checking each
//! typed wrapper field against its format rules. Each violation produces a
//! `ValidationError` with the specific `rule_id` for that field type.

use std::sync::LazyLock;

use regex::Regex;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const ISO_4217_CODES: &[&str; 157] = &[
    "AED", "AFN", "ALL", "AMD", "ANG", "AOA", "ARS", "AUD", "AWG", "AZN", "BAM", "BBD", "BDT",
    "BGN", "BHD", "BIF", "BMD", "BND", "BOB", "BRL", "BSD", "BTN", "BWP", "BYN", "BZD", "CAD",
    "CDF", "CHF", "CLP", "CNY", "COP", "CRC", "CUP", "CVE", "CZK", "DJF", "DKK", "DOP", "DZD",
    "EGP", "ERN", "ETB", "EUR", "FJD", "FKP", "GBP", "GEL", "GHS", "GIP", "GMD", "GNF", "GTQ",
    "GYD", "HKD", "HNL", "HRK", "HTG", "HUF", "IDR", "ILS", "INR", "IQD", "IRR", "ISK", "JMD",
    "JOD", "JPY", "KES", "KGS", "KHR", "KMF", "KPW", "KRW", "KWD", "KYD", "KZT", "LAK", "LBP",
    "LKR", "LRD", "LSL", "LYD", "MAD", "MDL", "MGA", "MKD", "MMK", "MNT", "MOP", "MRU", "MUR",
    "MVR", "MWK", "MXN", "MYR", "MZN", "NAD", "NGN", "NIO", "NOK", "NPR", "NZD", "OMR", "PAB",
    "PEN", "PGK", "PHP", "PKR", "PLN", "PYG", "QAR", "RON", "RSD", "RUB", "RWF", "SAR", "SBD",
    "SCR", "SDG", "SEK", "SGD", "SHP", "SLE", "SLL", "SOS", "SRD", "SSP", "STN", "SVC", "SYP",
    "SZL", "THB", "TJS", "TMT", "TND", "TOP", "TRY", "TTD", "TWD", "TZS", "UAH", "UGX", "USD",
    "UYU", "UZS", "VES", "VND", "VUV", "WST", "XAF", "XCD", "XOF", "XPF", "YER", "ZAR", "ZMW",
    "ZWL",
];

static BCP47_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z]{2,3}(-[a-zA-Z0-9]{1,8})*$").unwrap());

static EMAIL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap());

#[must_use]
pub fn is_valid_url(value: &str) -> bool {
    url::Url::parse(value)
        .map(|u| u.scheme() == "http" || u.scheme() == "https")
        .unwrap_or(false)
}

#[must_use]
pub fn is_valid_timezone(value: &str) -> bool {
    value.parse::<chrono_tz::Tz>().is_ok()
}

#[must_use]
pub fn is_valid_color(value: &str) -> bool {
    value.len() == 6 && value.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn is_valid_language_code(value: &str) -> bool {
    BCP47_RE.is_match(value)
}

#[must_use]
pub fn is_valid_currency(value: &str) -> bool {
    ISO_4217_CODES.contains(&value)
}

pub fn is_valid_email(value: &str) -> bool {
    EMAIL_RE.is_match(value)
}

static PHONE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9+\-.()\s]{5,}$").unwrap());

#[must_use]
pub fn is_valid_phone(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && PHONE_RE.is_match(trimmed)
}

fn err(rule_id: &str, file: &str, field: &str, value: &str, line: usize) -> ValidationError {
    ValidationError::new(rule_id, "3", Severity::Error)
        .file(file)
        .field(field)
        .value(value)
        .line(line)
        .message(format!("Invalid {field}: '{value}'"))
}

macro_rules! check_url {
    ($errors:expr, $file:expr, $field:expr, $val:expr, $line:expr) => {
        let s: &str = $val.as_ref();
        if !is_valid_url(s) {
            $errors.push(err("invalid_url", $file, $field, s, $line));
        }
    };
}

macro_rules! check_opt_url {
    ($errors:expr, $file:expr, $field:expr, $val:expr, $line:expr) => {
        if let Some(ref v) = $val {
            check_url!($errors, $file, $field, v, $line);
        }
    };
}

macro_rules! check_tz {
    ($errors:expr, $file:expr, $field:expr, $val:expr, $line:expr) => {
        let s: &str = $val.as_ref();
        if !is_valid_timezone(s) {
            $errors.push(err("invalid_timezone", $file, $field, s, $line));
        }
    };
}

macro_rules! check_opt_tz {
    ($errors:expr, $file:expr, $field:expr, $val:expr, $line:expr) => {
        if let Some(ref v) = $val {
            check_tz!($errors, $file, $field, v, $line);
        }
    };
}

macro_rules! check_opt_color {
    ($errors:expr, $file:expr, $field:expr, $val:expr, $line:expr) => {
        if let Some(ref v) = $val {
            let s: &str = v.as_ref();
            if !is_valid_color(s) {
                $errors.push(err("invalid_color", $file, $field, s, $line));
            }
        }
    };
}

macro_rules! check_opt_lang {
    ($errors:expr, $file:expr, $field:expr, $val:expr, $line:expr) => {
        if let Some(ref v) = $val {
            let s: &str = v.as_ref();
            if !is_valid_language_code(s) {
                $errors.push(err("invalid_language_code", $file, $field, s, $line));
            }
        }
    };
}

macro_rules! check_lang {
    ($errors:expr, $file:expr, $field:expr, $val:expr, $line:expr) => {
        let s: &str = $val.as_ref();
        if !is_valid_language_code(s) {
            $errors.push(err("invalid_language_code", $file, $field, s, $line));
        }
    };
}

macro_rules! check_opt_email {
    ($errors:expr, $file:expr, $field:expr, $val:expr, $line:expr) => {
        if let Some(ref v) = $val {
            let s: &str = v.as_ref();
            if !is_valid_email(s) {
                $errors.push(err("invalid_email", $file, $field, s, $line));
            }
        }
    };
}

macro_rules! check_opt_phone {
    ($errors:expr, $file:expr, $field:expr, $val:expr, $line:expr) => {
        if let Some(ref v) = $val {
            let s: &str = v.as_ref();
            if !is_valid_phone(s) {
                $errors.push(err("invalid_phone_number", $file, $field, s, $line));
            }
        }
    };
}

macro_rules! check_currency {
    ($errors:expr, $file:expr, $field:expr, $val:expr, $line:expr) => {
        let s: &str = $val.as_ref();
        if !is_valid_currency(s) {
            $errors.push(err("invalid_currency", $file, $field, s, $line));
        }
    };
}

pub struct FieldTypeValidator;

fn check_agencies(feed: &GtfsFeed, errors: &mut Vec<ValidationError>) {
    for (i, a) in feed.agencies.iter().enumerate() {
        let line = i + 2;
        check_url!(errors, "agency.txt", "agency_url", a.agency_url, line);
        check_tz!(
            errors,
            "agency.txt",
            "agency_timezone",
            a.agency_timezone,
            line
        );
        check_opt_lang!(errors, "agency.txt", "agency_lang", a.agency_lang, line);
        check_opt_email!(errors, "agency.txt", "agency_email", a.agency_email, line);
        check_opt_phone!(errors, "agency.txt", "agency_phone", a.agency_phone, line);
        check_opt_url!(
            errors,
            "agency.txt",
            "agency_fare_url",
            a.agency_fare_url,
            line
        );
    }
}

fn check_stops(feed: &GtfsFeed, errors: &mut Vec<ValidationError>) {
    for (i, s) in feed.stops.iter().enumerate() {
        let line = i + 2;
        check_opt_url!(errors, "stops.txt", "stop_url", s.stop_url, line);
        check_opt_tz!(errors, "stops.txt", "stop_timezone", s.stop_timezone, line);
    }
}

fn check_routes(feed: &GtfsFeed, errors: &mut Vec<ValidationError>) {
    for (i, r) in feed.routes.iter().enumerate() {
        let line = i + 2;
        check_opt_url!(errors, "routes.txt", "route_url", r.route_url, line);
        check_opt_color!(errors, "routes.txt", "route_color", r.route_color, line);
        check_opt_color!(
            errors,
            "routes.txt",
            "route_text_color",
            r.route_text_color,
            line
        );
    }
}

fn check_feed_info(feed: &GtfsFeed, errors: &mut Vec<ValidationError>) {
    if let Some(ref fi) = feed.feed_info {
        let line = 2;
        check_url!(
            errors,
            "feed_info.txt",
            "feed_publisher_url",
            fi.feed_publisher_url,
            line
        );
        check_lang!(errors, "feed_info.txt", "feed_lang", fi.feed_lang, line);
        check_opt_lang!(
            errors,
            "feed_info.txt",
            "default_lang",
            fi.default_lang,
            line
        );
        check_opt_email!(
            errors,
            "feed_info.txt",
            "feed_contact_email",
            fi.feed_contact_email,
            line
        );
        check_opt_url!(
            errors,
            "feed_info.txt",
            "feed_contact_url",
            fi.feed_contact_url,
            line
        );
    }
}

fn check_fares_translations_attributions(feed: &GtfsFeed, errors: &mut Vec<ValidationError>) {
    for (i, fa) in feed.fare_attributes.iter().enumerate() {
        let line = i + 2;
        check_currency!(
            errors,
            "fare_attributes.txt",
            "currency_type",
            fa.currency_type,
            line
        );
    }

    for (i, t) in feed.translations.iter().enumerate() {
        let line = i + 2;
        check_lang!(errors, "translations.txt", "language", t.language, line);
    }

    for (i, a) in feed.attributions.iter().enumerate() {
        let line = i + 2;
        check_opt_url!(
            errors,
            "attributions.txt",
            "attribution_url",
            a.attribution_url,
            line
        );
        check_opt_email!(
            errors,
            "attributions.txt",
            "attribution_email",
            a.attribution_email,
            line
        );
        check_opt_phone!(
            errors,
            "attributions.txt",
            "attribution_phone",
            a.attribution_phone,
            line
        );
    }
}

impl ValidationRule for FieldTypeValidator {
    fn rule_id(&self) -> &'static str {
        "field_type_validator"
    }

    fn section(&self) -> &'static str {
        "3"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        check_agencies(feed, &mut errors);
        check_stops(feed, &mut errors);
        check_routes(feed, &mut errors);
        check_feed_info(feed, &mut errors);
        check_fares_translations_attributions(feed, &mut errors);
        errors
    }
}
