use std::io::BufRead;

use crate::models::{Calendar, GtfsDate, ServiceId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{bool_field, required_id, required_parse};

const FILE: &str = "calendar.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Calendar>, Vec<ParseError>) {
    let Ok(iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (line, row) in iter {
        let (service_id, mut e) = required_id::<ServiceId>(&row, "service_id", FILE, line);
        errors.append(&mut e);
        let monday = bool_field(&row, "monday");
        let tuesday = bool_field(&row, "tuesday");
        let wednesday = bool_field(&row, "wednesday");
        let thursday = bool_field(&row, "thursday");
        let friday = bool_field(&row, "friday");
        let saturday = bool_field(&row, "saturday");
        let sunday = bool_field(&row, "sunday");
        let (start_date, mut e) =
            required_parse::<GtfsDate>(&row, "start_date", FILE, line, ParseErrorKind::InvalidDate);
        errors.append(&mut e);
        let (end_date, mut e) =
            required_parse::<GtfsDate>(&row, "end_date", FILE, line, ParseErrorKind::InvalidDate);
        errors.append(&mut e);

        records.push(Calendar {
            service_id,
            monday,
            tuesday,
            wednesday,
            thursday,
            friday,
            saturday,
            sunday,
            start_date,
            end_date,
        });
    }

    (records, errors)
}
