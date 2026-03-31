use std::io::BufRead;

use crate::models::{RouteId, StopId, Transfer, TransferType, TripId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{optional_id, optional_parse, required_enum};

const FILE: &str = "transfers.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Transfer>, Vec<ParseError>) {
    let Ok(iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (line, row) in iter {
        let from_stop_id = optional_id::<StopId>(&row, "from_stop_id");
        let to_stop_id = optional_id::<StopId>(&row, "to_stop_id");
        let from_route_id = optional_id::<RouteId>(&row, "from_route_id");
        let to_route_id = optional_id::<RouteId>(&row, "to_route_id");
        let from_trip_id = optional_id::<TripId>(&row, "from_trip_id");
        let to_trip_id = optional_id::<TripId>(&row, "to_trip_id");
        let (transfer_type, mut e) = required_enum(
            &row,
            "transfer_type",
            FILE,
            line,
            TransferType::from_i32,
            TransferType::Recommended,
        );
        errors.append(&mut e);
        let (min_transfer_time, mut e) = optional_parse::<u32>(
            &row,
            "min_transfer_time",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
        );
        errors.append(&mut e);

        records.push(Transfer {
            from_stop_id,
            to_stop_id,
            from_route_id,
            to_route_id,
            from_trip_id,
            to_trip_id,
            transfer_type,
            min_transfer_time,
        });
    }

    (records, errors)
}
