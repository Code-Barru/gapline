use std::io::BufRead;

use crate::models::{Network, NetworkId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::ParseError;
use crate::parser::field_parsers::{optional_str, required_id};

const FILE: &str = "networks.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Network>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let network_id = required_id::<NetworkId>(&row, "network_id", FILE, line, &mut errors);
        let network_name = optional_str(&row, "network_name");

        records.push(Network {
            network_id,
            network_name,
        });
    }

    (records, errors)
}
