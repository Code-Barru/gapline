use std::io::BufRead;

use crate::models::{
    BikesAllowed, DirectionId, RouteId, ServiceId, ShapeId, Trip, TripId, WheelchairAccessible,
};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::ParseError;
use crate::parser::field_parsers::{optional_enum, optional_id, optional_str, required_id};

const FILE: &str = "trips.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Trip>, Vec<ParseError>) {
    let Ok(iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (line, row) in iter {
        let (route_id, mut e) = required_id::<RouteId>(&row, "route_id", FILE, line);
        errors.append(&mut e);
        let (service_id, mut e) = required_id::<ServiceId>(&row, "service_id", FILE, line);
        errors.append(&mut e);
        let (trip_id, mut e) = required_id::<TripId>(&row, "trip_id", FILE, line);
        errors.append(&mut e);
        let trip_headsign = optional_str(&row, "trip_headsign");
        let trip_short_name = optional_str(&row, "trip_short_name");
        let (direction_id, mut e) =
            optional_enum(&row, "direction_id", FILE, line, DirectionId::from_i32);
        errors.append(&mut e);
        let block_id = optional_str(&row, "block_id");
        let shape_id = optional_id::<ShapeId>(&row, "shape_id");
        let (wheelchair_accessible, mut e) = optional_enum(
            &row,
            "wheelchair_accessible",
            FILE,
            line,
            WheelchairAccessible::from_i32,
        );
        errors.append(&mut e);
        let (bikes_allowed, mut e) =
            optional_enum(&row, "bikes_allowed", FILE, line, BikesAllowed::from_i32);
        errors.append(&mut e);

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
