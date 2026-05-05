use std::io::BufRead;

use crate::models::{Area, AreaId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::ParseError;
use crate::parser::field_parsers::{optional_str, required_id};

const FILE: &str = "areas.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Area>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let area_id = required_id::<AreaId>(&row, "area_id", FILE, line, &mut errors);
        let area_name = optional_str(&row, "area_name");

        records.push(Area { area_id, area_name });
    }

    (records, errors)
}
