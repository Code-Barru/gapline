use std::io::BufRead;

use crate::models::{LanguageCode, Translation};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::ParseError;
use crate::parser::field_parsers::{optional_str, required_str, required_wrapper};

const FILE: &str = "translations.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Translation>, Vec<ParseError>) {
    let Ok(iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (line, row) in iter {
        let (table_name, mut e) = required_str(&row, "table_name", FILE, line);
        errors.append(&mut e);
        let (field_name, mut e) = required_str(&row, "field_name", FILE, line);
        errors.append(&mut e);
        let (language, mut e) = required_wrapper::<LanguageCode>(&row, "language", FILE, line);
        errors.append(&mut e);
        let (translation, mut e) = required_str(&row, "translation", FILE, line);
        errors.append(&mut e);
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
