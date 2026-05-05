use std::io::BufRead;

use crate::models::{AreaId, StopArea, StopId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::ParseError;
use crate::parser::field_parsers::required_id;

const FILE: &str = "stop_areas.txt";

pub fn parse(reader: impl BufRead) -> (Vec<StopArea>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let area_id = required_id::<AreaId>(&row, "area_id", FILE, line, &mut errors);
        let stop_id = required_id::<StopId>(&row, "stop_id", FILE, line, &mut errors);

        records.push(StopArea { area_id, stop_id });
    }

    (records, errors)
}
