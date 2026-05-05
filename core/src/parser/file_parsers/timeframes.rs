use std::io::BufRead;

use crate::models::{GtfsTime, ServiceId, Timeframe, TimeframeId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{required_id, required_parse};

const FILE: &str = "timeframes.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Timeframe>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let timeframe_group_id =
            required_id::<TimeframeId>(&row, "timeframe_group_id", FILE, line, &mut errors);
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
        let service_id = required_id::<ServiceId>(&row, "service_id", FILE, line, &mut errors);

        records.push(Timeframe {
            timeframe_group_id,
            start_time,
            end_time,
            service_id,
        });
    }

    (records, errors)
}
