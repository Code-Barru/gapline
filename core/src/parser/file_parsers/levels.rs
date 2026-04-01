use std::io::BufRead;

use crate::models::{Level, LevelId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{optional_str, required_id, required_parse};

const FILE: &str = "levels.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Level>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let level_id = required_id::<LevelId>(&row, "level_id", FILE, line, &mut errors);
        let level_index = required_parse::<f64>(
            &row,
            "level_index",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
            &mut errors,
        );
        let level_name = optional_str(&row, "level_name");

        records.push(Level {
            level_id,
            level_index,
            level_name,
        });
    }

    (records, errors)
}
