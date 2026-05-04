use std::io::BufRead;

use crate::models::{
    BookingRuleId, ContinuousDropOff, ContinuousPickup, DropOffType, GtfsTime, PickupType, StopId,
    StopTime, Timepoint, TripId,
};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{
    optional_enum, optional_id, optional_parse, optional_str, required_id, required_parse,
};

const FILE: &str = "stop_times.txt";

#[allow(clippy::too_many_lines)]
pub fn parse(reader: impl BufRead) -> (Vec<StopTime>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let trip_id = required_id::<TripId>(&row, "trip_id", FILE, line, &mut errors);
        let arrival_time = optional_parse::<GtfsTime>(
            &row,
            "arrival_time",
            FILE,
            line,
            ParseErrorKind::InvalidTime,
            &mut errors,
        );
        let departure_time = optional_parse::<GtfsTime>(
            &row,
            "departure_time",
            FILE,
            line,
            ParseErrorKind::InvalidTime,
            &mut errors,
        );
        let stop_id = required_id::<StopId>(&row, "stop_id", FILE, line, &mut errors);
        let stop_sequence = required_parse::<u32>(
            &row,
            "stop_sequence",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );
        let stop_headsign = optional_str(&row, "stop_headsign");
        let pickup_type = optional_enum(
            &row,
            "pickup_type",
            FILE,
            line,
            PickupType::from_i32,
            &mut errors,
        );
        let drop_off_type = optional_enum(
            &row,
            "drop_off_type",
            FILE,
            line,
            DropOffType::from_i32,
            &mut errors,
        );
        let continuous_pickup = optional_enum(
            &row,
            "continuous_pickup",
            FILE,
            line,
            ContinuousPickup::from_i32,
            &mut errors,
        );
        let continuous_drop_off = optional_enum(
            &row,
            "continuous_drop_off",
            FILE,
            line,
            ContinuousDropOff::from_i32,
            &mut errors,
        );
        let shape_dist_traveled = optional_parse::<f64>(
            &row,
            "shape_dist_traveled",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
            &mut errors,
        );
        let timepoint = optional_enum(
            &row,
            "timepoint",
            FILE,
            line,
            Timepoint::from_i32,
            &mut errors,
        );
        let start_pickup_drop_off_window = optional_parse::<GtfsTime>(
            &row,
            "start_pickup_drop_off_window",
            FILE,
            line,
            ParseErrorKind::InvalidTime,
            &mut errors,
        );
        let end_pickup_drop_off_window = optional_parse::<GtfsTime>(
            &row,
            "end_pickup_drop_off_window",
            FILE,
            line,
            ParseErrorKind::InvalidTime,
            &mut errors,
        );
        let pickup_booking_rule_id = optional_id::<BookingRuleId>(&row, "pickup_booking_rule_id");
        let drop_off_booking_rule_id =
            optional_id::<BookingRuleId>(&row, "drop_off_booking_rule_id");
        let mean_duration_factor = optional_parse::<f64>(
            &row,
            "mean_duration_factor",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
            &mut errors,
        );
        let mean_duration_offset = optional_parse::<f64>(
            &row,
            "mean_duration_offset",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
            &mut errors,
        );
        let safe_duration_factor = optional_parse::<f64>(
            &row,
            "safe_duration_factor",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
            &mut errors,
        );
        let safe_duration_offset = optional_parse::<f64>(
            &row,
            "safe_duration_offset",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
            &mut errors,
        );

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
            start_pickup_drop_off_window,
            end_pickup_drop_off_window,
            pickup_booking_rule_id,
            drop_off_booking_rule_id,
            mean_duration_factor,
            mean_duration_offset,
            safe_duration_factor,
            safe_duration_offset,
        });
    }

    (records, errors)
}
