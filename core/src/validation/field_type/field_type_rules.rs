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

#[must_use]
pub fn is_valid_phone(value: &str) -> bool {
    !value.trim().is_empty()
}

fn err(rule_id: &str, file: &str, field: &str, value: &str) -> ValidationError {
    ValidationError::new(rule_id, "3", Severity::Error)
        .file(file)
        .field(field)
        .value(value)
        .message(format!("Invalid {field}: '{value}'"))
}

macro_rules! check_url {
    ($errors:expr, $file:expr, $field:expr, $val:expr) => {
        let s: &str = $val.as_ref();
        if !is_valid_url(s) {
            $errors.push(err("invalid_url", $file, $field, s));
        }
    };
}

macro_rules! check_opt_url {
    ($errors:expr, $file:expr, $field:expr, $val:expr) => {
        if let Some(ref v) = $val {
            check_url!($errors, $file, $field, v);
        }
    };
}

macro_rules! check_tz {
    ($errors:expr, $file:expr, $field:expr, $val:expr) => {
        let s: &str = $val.as_ref();
        if !is_valid_timezone(s) {
            $errors.push(err("invalid_timezone", $file, $field, s));
        }
    };
}

macro_rules! check_opt_tz {
    ($errors:expr, $file:expr, $field:expr, $val:expr) => {
        if let Some(ref v) = $val {
            check_tz!($errors, $file, $field, v);
        }
    };
}

macro_rules! check_opt_color {
    ($errors:expr, $file:expr, $field:expr, $val:expr) => {
        if let Some(ref v) = $val {
            let s: &str = v.as_ref();
            if !is_valid_color(s) {
                $errors.push(err("invalid_color", $file, $field, s));
            }
        }
    };
}

macro_rules! check_opt_lang {
    ($errors:expr, $file:expr, $field:expr, $val:expr) => {
        if let Some(ref v) = $val {
            let s: &str = v.as_ref();
            if !is_valid_language_code(s) {
                $errors.push(err("invalid_language_code", $file, $field, s));
            }
        }
    };
}

macro_rules! check_lang {
    ($errors:expr, $file:expr, $field:expr, $val:expr) => {
        let s: &str = $val.as_ref();
        if !is_valid_language_code(s) {
            $errors.push(err("invalid_language_code", $file, $field, s));
        }
    };
}

macro_rules! check_opt_email {
    ($errors:expr, $file:expr, $field:expr, $val:expr) => {
        if let Some(ref v) = $val {
            let s: &str = v.as_ref();
            if !is_valid_email(s) {
                $errors.push(err("invalid_email", $file, $field, s));
            }
        }
    };
}

macro_rules! check_opt_phone {
    ($errors:expr, $file:expr, $field:expr, $val:expr) => {
        if let Some(ref v) = $val {
            let s: &str = v.as_ref();
            if !is_valid_phone(s) {
                $errors.push(err("invalid_phone_number", $file, $field, s));
            }
        }
    };
}

macro_rules! check_currency {
    ($errors:expr, $file:expr, $field:expr, $val:expr) => {
        let s: &str = $val.as_ref();
        if !is_valid_currency(s) {
            $errors.push(err("invalid_currency", $file, $field, s));
        }
    };
}

pub struct FieldTypeValidator;

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

        for a in &feed.agencies {
            check_url!(errors, "agency.txt", "agency_url", a.agency_url);
            check_tz!(errors, "agency.txt", "agency_timezone", a.agency_timezone);
            check_opt_lang!(errors, "agency.txt", "agency_lang", a.agency_lang);
            check_opt_email!(errors, "agency.txt", "agency_email", a.agency_email);
            check_opt_phone!(errors, "agency.txt", "agency_phone", a.agency_phone);
            check_opt_url!(errors, "agency.txt", "agency_fare_url", a.agency_fare_url);
        }

        for s in &feed.stops {
            check_opt_url!(errors, "stops.txt", "stop_url", s.stop_url);
            check_opt_tz!(errors, "stops.txt", "stop_timezone", s.stop_timezone);
        }

        for r in &feed.routes {
            check_opt_url!(errors, "routes.txt", "route_url", r.route_url);
            check_opt_color!(errors, "routes.txt", "route_color", r.route_color);
            check_opt_color!(errors, "routes.txt", "route_text_color", r.route_text_color);
        }

        if let Some(ref fi) = feed.feed_info {
            check_url!(
                errors,
                "feed_info.txt",
                "feed_publisher_url",
                fi.feed_publisher_url
            );
            check_lang!(errors, "feed_info.txt", "feed_lang", fi.feed_lang);
            check_opt_lang!(errors, "feed_info.txt", "default_lang", fi.default_lang);
            check_opt_email!(
                errors,
                "feed_info.txt",
                "feed_contact_email",
                fi.feed_contact_email
            );
            check_opt_url!(
                errors,
                "feed_info.txt",
                "feed_contact_url",
                fi.feed_contact_url
            );
        }

        for fa in &feed.fare_attributes {
            check_currency!(
                errors,
                "fare_attributes.txt",
                "currency_type",
                fa.currency_type
            );
        }

        for t in &feed.translations {
            check_lang!(errors, "translations.txt", "language", t.language);
        }

        for a in &feed.attributions {
            check_opt_url!(
                errors,
                "attributions.txt",
                "attribution_url",
                a.attribution_url
            );
            check_opt_email!(
                errors,
                "attributions.txt",
                "attribution_email",
                a.attribution_email
            );
            check_opt_phone!(
                errors,
                "attributions.txt",
                "attribution_phone",
                a.attribution_phone
            );
        }

        errors
    }
}
