use std::io::BufRead;

use crate::models::{Calendar, GtfsDate, ServiceId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{bool_field, required_id, required_parse};

const FILE: &str = "calendar.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Calendar>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let service_id = required_id::<ServiceId>(&row, "service_id", FILE, line, &mut errors);
        let monday = bool_field(&row, "monday");
        let tuesday = bool_field(&row, "tuesday");
        let wednesday = bool_field(&row, "wednesday");
        let thursday = bool_field(&row, "thursday");
        let friday = bool_field(&row, "friday");
        let saturday = bool_field(&row, "saturday");
        let sunday = bool_field(&row, "sunday");
        let start_date = required_parse::<GtfsDate>(
            &row,
            "start_date",
            FILE,
            line,
            ParseErrorKind::InvalidDate,
            &mut errors,
        );
        let end_date = required_parse::<GtfsDate>(
            &row,
            "end_date",
            FILE,
            line,
            ParseErrorKind::InvalidDate,
            &mut errors,
        );

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
