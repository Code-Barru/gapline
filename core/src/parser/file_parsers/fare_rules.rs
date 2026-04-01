use std::io::BufRead;

use crate::models::{FareId, FareRule, RouteId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::ParseError;
use crate::parser::field_parsers::{optional_id, optional_str, required_id};

const FILE: &str = "fare_rules.txt";

pub fn parse(reader: impl BufRead) -> (Vec<FareRule>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let fare_id = required_id::<FareId>(&row, "fare_id", FILE, line, &mut errors);
        let route_id = optional_id::<RouteId>(&row, "route_id");
        let origin_id = optional_str(&row, "origin_id");
        let destination_id = optional_str(&row, "destination_id");
        let contains_id = optional_str(&row, "contains_id");

        records.push(FareRule {
            fare_id,
            route_id,
            origin_id,
            destination_id,
            contains_id,
        });
    }

    (records, errors)
}
