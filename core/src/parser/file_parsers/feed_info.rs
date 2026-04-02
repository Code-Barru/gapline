use std::io::BufRead;

use crate::models::{Email, FeedInfo, GtfsDate, LanguageCode, Url};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{
    optional_id, optional_parse, optional_str, required_id, required_str,
};

const FILE: &str = "feed_info.txt";

pub fn parse(reader: impl BufRead) -> (Option<FeedInfo>, usize, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (None, 0, vec![]);
    };

    let Some((line, row)) = iter.next_row() else {
        return (None, 0, vec![]);
    };

    let mut errors = Vec::new();

    let feed_publisher_name = required_str(&row, "feed_publisher_name", FILE, line, &mut errors);
    let feed_publisher_url =
        required_id::<Url>(&row, "feed_publisher_url", FILE, line, &mut errors);
    let feed_lang = required_id::<LanguageCode>(&row, "feed_lang", FILE, line, &mut errors);
    let default_lang = optional_id::<LanguageCode>(&row, "default_lang");
    let feed_start_date = optional_parse::<GtfsDate>(
        &row,
        "feed_start_date",
        FILE,
        line,
        ParseErrorKind::InvalidDate,
        &mut errors,
    );
    let feed_end_date = optional_parse::<GtfsDate>(
        &row,
        "feed_end_date",
        FILE,
        line,
        ParseErrorKind::InvalidDate,
        &mut errors,
    );
    let feed_version = optional_str(&row, "feed_version");
    let feed_contact_email = optional_id::<Email>(&row, "feed_contact_email");
    let feed_contact_url = optional_id::<Url>(&row, "feed_contact_url");

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

    let mut line_count = 1;
    while iter.next_row().is_some() {
        line_count += 1;
    }

    (Some(info), line_count, errors)
}
