use std::io::BufRead;

use crate::models::{NetworkId, RouteId, RouteNetwork};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::ParseError;
use crate::parser::field_parsers::required_id;

const FILE: &str = "route_networks.txt";

pub fn parse(reader: impl BufRead) -> (Vec<RouteNetwork>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let network_id = required_id::<NetworkId>(&row, "network_id", FILE, line, &mut errors);
        let route_id = required_id::<RouteId>(&row, "route_id", FILE, line, &mut errors);

        records.push(RouteNetwork {
            network_id,
            route_id,
        });
    }

    (records, errors)
}
