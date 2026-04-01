use std::io::BufRead;

use crate::models::{ExactTimes, Frequency, GtfsTime, TripId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{optional_enum, required_id, required_parse};

const FILE: &str = "frequencies.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Frequency>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let trip_id = required_id::<TripId>(&row, "trip_id", FILE, line, &mut errors);
        let start_time = required_parse::<GtfsTime>(
            &row,
            "start_time",
            FILE,
            line,
            ParseErrorKind::InvalidTime,
            &mut errors,
        );
        let end_time = required_parse::<GtfsTime>(
            &row,
            "end_time",
            FILE,
            line,
            ParseErrorKind::InvalidTime,
            &mut errors,
        );
        let headway_secs = required_parse::<u32>(
            &row,
            "headway_secs",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );
        let exact_times = optional_enum(
            &row,
            "exact_times",
            FILE,
            line,
            ExactTimes::from_i32,
            &mut errors,
        );

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
