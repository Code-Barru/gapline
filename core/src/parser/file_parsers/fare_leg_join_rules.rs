use std::io::BufRead;

use crate::models::{FareLegJoinRule, NetworkId, StopId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::ParseError;
use crate::parser::field_parsers::{optional_id, required_id};

const FILE: &str = "fare_leg_join_rules.txt";

pub fn parse(reader: impl BufRead) -> (Vec<FareLegJoinRule>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let from_network_id =
            required_id::<NetworkId>(&row, "from_network_id", FILE, line, &mut errors);
        let to_network_id =
            required_id::<NetworkId>(&row, "to_network_id", FILE, line, &mut errors);
        let from_stop_id = optional_id::<StopId>(&row, "from_stop_id");
        let to_stop_id = optional_id::<StopId>(&row, "to_stop_id");

        records.push(FareLegJoinRule {
            from_network_id,
            to_network_id,
            from_stop_id,
            to_stop_id,
        });
    }

    (records, errors)
}
