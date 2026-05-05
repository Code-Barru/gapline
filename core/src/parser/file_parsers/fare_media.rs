use std::io::BufRead;

use crate::models::{FareMedia, FareMediaId, FareMediaType};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::ParseError;
use crate::parser::field_parsers::{optional_str, required_enum, required_id};

const FILE: &str = "fare_media.txt";

pub fn parse(reader: impl BufRead) -> (Vec<FareMedia>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let fare_media_id =
            required_id::<FareMediaId>(&row, "fare_media_id", FILE, line, &mut errors);
        let fare_media_name = optional_str(&row, "fare_media_name");
        let fare_media_type = required_enum(
            &row,
            "fare_media_type",
            FILE,
            line,
            FareMediaType::from_i32,
            FareMediaType::None,
            &mut errors,
        );

        records.push(FareMedia {
            fare_media_id,
            fare_media_name,
            fare_media_type,
        });
    }

    (records, errors)
}
