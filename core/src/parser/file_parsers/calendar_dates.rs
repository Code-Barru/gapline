use std::io::BufRead;

use crate::models::{CalendarDate, ExceptionType, GtfsDate, ServiceId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{required_enum, required_id, required_parse};

const FILE: &str = "calendar_dates.txt";

pub fn parse(reader: impl BufRead) -> (Vec<CalendarDate>, Vec<ParseError>) {
    let Ok(iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (line, row) in iter {
        let (service_id, mut e) = required_id::<ServiceId>(&row, "service_id", FILE, line);
        errors.append(&mut e);
        let (date, mut e) =
            required_parse::<GtfsDate>(&row, "date", FILE, line, ParseErrorKind::InvalidDate);
        errors.append(&mut e);
        let (exception_type, mut e) = required_enum(
            &row,
            "exception_type",
            FILE,
            line,
            ExceptionType::from_i32,
            ExceptionType::Added,
        );
        errors.append(&mut e);

        records.push(CalendarDate {
            service_id,
            date,
            exception_type,
        });
    }

    (records, errors)
}
