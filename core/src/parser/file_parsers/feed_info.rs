use std::io::BufRead;

use crate::models::{Email, FeedInfo, GtfsDate, LanguageCode, Url};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{
    optional_parse, optional_str, optional_wrapper, required_str, required_wrapper,
};

const FILE: &str = "feed_info.txt";

pub fn parse(reader: impl BufRead) -> (Option<FeedInfo>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (None, vec![]);
    };

    let Some((line, row)) = iter.next() else {
        return (None, vec![]);
    };

    let mut errors = Vec::new();

    let (feed_publisher_name, mut e) = required_str(&row, "feed_publisher_name", FILE, line);
    errors.append(&mut e);
    let (feed_publisher_url, mut e) =
        required_wrapper::<Url>(&row, "feed_publisher_url", FILE, line);
    errors.append(&mut e);
    let (feed_lang, mut e) = required_wrapper::<LanguageCode>(&row, "feed_lang", FILE, line);
    errors.append(&mut e);
    let default_lang = optional_wrapper::<LanguageCode>(&row, "default_lang");
    let (feed_start_date, mut e) = optional_parse::<GtfsDate>(
        &row,
        "feed_start_date",
        FILE,
        line,
        ParseErrorKind::InvalidDate,
    );
    errors.append(&mut e);
    let (feed_end_date, mut e) = optional_parse::<GtfsDate>(
        &row,
        "feed_end_date",
        FILE,
        line,
        ParseErrorKind::InvalidDate,
    );
    errors.append(&mut e);
    let feed_version = optional_str(&row, "feed_version");
    let feed_contact_email = optional_wrapper::<Email>(&row, "feed_contact_email");
    let feed_contact_url = optional_wrapper::<Url>(&row, "feed_contact_url");

    let info = FeedInfo {
        feed_publisher_name,
        feed_publisher_url,
        feed_lang,
        default_lang,
        feed_start_date,
        feed_end_date,
        feed_version,
        feed_contact_email,
        feed_contact_url,
    };

    (Some(info), errors)
}
