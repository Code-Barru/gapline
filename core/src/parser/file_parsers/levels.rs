use std::io::BufRead;

use crate::models::{Level, LevelId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{optional_str, required_id, required_parse};

const FILE: &str = "levels.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Level>, Vec<ParseError>) {
    let Ok(iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (line, row) in iter {
        let (level_id, mut e) = required_id::<LevelId>(&row, "level_id", FILE, line);
        errors.append(&mut e);
        let (level_index, mut e) = required_parse::<f64>(
            &row,
            "level_index",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
        );
        errors.append(&mut e);
        let level_name = optional_str(&row, "level_name");

        records.push(Level {
            level_id,
            level_index,
            level_name,
        });
    }

    (records, errors)
}
