use std::io::BufRead;

use crate::models::{LocationGroup, LocationGroupId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::ParseError;
use crate::parser::field_parsers::{optional_str, required_id};

const FILE: &str = "location_groups.txt";

pub fn parse(reader: impl BufRead) -> (Vec<LocationGroup>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let location_group_id =
            required_id::<LocationGroupId>(&row, "location_group_id", FILE, line, &mut errors);
        let location_group_name = optional_str(&row, "location_group_name");

        records.push(LocationGroup {
            location_group_id,
            location_group_name,
        });
    }

    (records, errors)
}
