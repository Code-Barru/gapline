use std::io::BufRead;

use crate::models::{CalendarDate, ExceptionType, GtfsDate, ServiceId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{required_enum, required_id, required_parse};

const FILE: &str = "calendar_dates.txt";

pub fn parse(reader: impl BufRead) -> (Vec<CalendarDate>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let service_id = required_id::<ServiceId>(&row, "service_id", FILE, line, &mut errors);
        let date = required_parse::<GtfsDate>(
            &row,
            "date",
            FILE,
            line,
            ParseErrorKind::InvalidDate,
            &mut errors,
        );
        let exception_type = required_enum(
            &row,
            "exception_type",
            FILE,
            line,
            ExceptionType::from_i32,
            ExceptionType::Added,
            &mut errors,
        );

        records.push(CalendarDate {
            service_id,
            date,
            exception_type,
        });
    }

    (records, errors)
}
