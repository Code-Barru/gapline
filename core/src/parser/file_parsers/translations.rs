use std::io::BufRead;

use crate::models::{LanguageCode, Translation};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::ParseError;
use crate::parser::field_parsers::{optional_str, required_id, required_str};

const FILE: &str = "translations.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Translation>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let table_name = required_str(&row, "table_name", FILE, line, &mut errors);
        let field_name = required_str(&row, "field_name", FILE, line, &mut errors);
        let language = required_id::<LanguageCode>(&row, "language", FILE, line, &mut errors);
        let translation = required_str(&row, "translation", FILE, line, &mut errors);
        let record_id = optional_str(&row, "record_id");
        let record_sub_id = optional_str(&row, "record_sub_id");
        let field_value = optional_str(&row, "field_value");

        records.push(Translation {
            table_name,
            field_name,
            language,
            translation,
            record_id,
            record_sub_id,
            field_value,
        });
    }

    (records, errors)
}
