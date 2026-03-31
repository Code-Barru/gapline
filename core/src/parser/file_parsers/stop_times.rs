use std::io::BufRead;

use crate::models::{
    ContinuousDropOff, ContinuousPickup, DropOffType, GtfsTime, PickupType, StopId, StopTime,
    Timepoint, TripId,
};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{
    optional_enum, optional_parse, optional_str, required_id, required_parse,
};

const FILE: &str = "stop_times.txt";

pub fn parse(reader: impl BufRead) -> (Vec<StopTime>, Vec<ParseError>) {
    let Ok(iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (line, row) in iter {
        let (trip_id, mut e) = required_id::<TripId>(&row, "trip_id", FILE, line);
        errors.append(&mut e);
        let (arrival_time, mut e) = optional_parse::<GtfsTime>(
            &row,
            "arrival_time",
            FILE,
            line,
            ParseErrorKind::InvalidTime,
        );
        errors.append(&mut e);
        let (departure_time, mut e) = optional_parse::<GtfsTime>(
            &row,
            "departure_time",
            FILE,
            line,
            ParseErrorKind::InvalidTime,
        );
        errors.append(&mut e);
        let (stop_id, mut e) = required_id::<StopId>(&row, "stop_id", FILE, line);
        errors.append(&mut e);
        let (stop_sequence, mut e) = required_parse::<u32>(
            &row,
            "stop_sequence",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
        );
        errors.append(&mut e);
        let stop_headsign = optional_str(&row, "stop_headsign");
        let (pickup_type, mut e) =
            optional_enum(&row, "pickup_type", FILE, line, PickupType::from_i32);
        errors.append(&mut e);
        let (drop_off_type, mut e) =
            optional_enum(&row, "drop_off_type", FILE, line, DropOffType::from_i32);
        errors.append(&mut e);
        let (continuous_pickup, mut e) = optional_enum(
            &row,
            "continuous_pickup",
            FILE,
            line,
            ContinuousPickup::from_i32,
        );
        errors.append(&mut e);
        let (continuous_drop_off, mut e) = optional_enum(
            &row,
            "continuous_drop_off",
            FILE,
            line,
            ContinuousDropOff::from_i32,
        );
        errors.append(&mut e);
        let (shape_dist_traveled, mut e) = optional_parse::<f64>(
            &row,
            "shape_dist_traveled",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
        );
        errors.append(&mut e);
        let (timepoint, mut e) = optional_enum(&row, "timepoint", FILE, line, Timepoint::from_i32);
        errors.append(&mut e);

        records.push(StopTime {
            trip_id,
            arrival_time,
            departure_time,
            stop_id,
            stop_sequence,
            stop_headsign,
            pickup_type,
            drop_off_type,
            continuous_pickup,
            continuous_drop_off,
            shape_dist_traveled,
            timepoint,
        });
    }

    (records, errors)
}
