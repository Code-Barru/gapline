use std::io::BufRead;

use crate::models::{ExactTimes, Frequency, GtfsTime, TripId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{optional_enum, required_id, required_parse};

const FILE: &str = "frequencies.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Frequency>, Vec<ParseError>) {
    let Ok(iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (line, row) in iter {
        let (trip_id, mut e) = required_id::<TripId>(&row, "trip_id", FILE, line);
        errors.append(&mut e);
        let (start_time, mut e) =
            required_parse::<GtfsTime>(&row, "start_time", FILE, line, ParseErrorKind::InvalidTime);
        errors.append(&mut e);
        let (end_time, mut e) =
            required_parse::<GtfsTime>(&row, "end_time", FILE, line, ParseErrorKind::InvalidTime);
        errors.append(&mut e);
        let (headway_secs, mut e) = required_parse::<u32>(
            &row,
            "headway_secs",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
        );
        errors.append(&mut e);
        let (exact_times, mut e) =
            optional_enum(&row, "exact_times", FILE, line, ExactTimes::from_i32);
        errors.append(&mut e);

        records.push(Frequency {
            trip_id,
            start_time,
            end_time,
            headway_secs,
            exact_times,
        });
    }

    (records, errors)
}
