use std::io::BufRead;

use crate::models::{
    BikesAllowed, DirectionId, RouteId, ServiceId, ShapeId, Trip, TripId, WheelchairAccessible,
};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::ParseError;
use crate::parser::field_parsers::{optional_enum, optional_id, optional_str, required_id};

const FILE: &str = "trips.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Trip>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let route_id = required_id::<RouteId>(&row, "route_id", FILE, line, &mut errors);
        let service_id = required_id::<ServiceId>(&row, "service_id", FILE, line, &mut errors);
        let trip_id = required_id::<TripId>(&row, "trip_id", FILE, line, &mut errors);
        let trip_headsign = optional_str(&row, "trip_headsign");
        let trip_short_name = optional_str(&row, "trip_short_name");
        let direction_id = optional_enum(
            &row,
            "direction_id",
            FILE,
            line,
            DirectionId::from_i32,
            &mut errors,
        );
        let block_id = optional_str(&row, "block_id");
        let shape_id = optional_id::<ShapeId>(&row, "shape_id");
        let wheelchair_accessible = optional_enum(
            &row,
            "wheelchair_accessible",
            FILE,
            line,
            WheelchairAccessible::from_i32,
            &mut errors,
        );
        let bikes_allowed = optional_enum(
            &row,
            "bikes_allowed",
            FILE,
            line,
            BikesAllowed::from_i32,
            &mut errors,
        );

        records.push(Trip {
            route_id,
            service_id,
            trip_id,
            trip_headsign,
            trip_short_name,
            direction_id,
            block_id,
            shape_id,
            wheelchair_accessible,
            bikes_allowed,
        });
    }

    (records, errors)
}
