use std::io::BufRead;

use crate::models::{AgencyId, CurrencyCode, FareAttribute, FareId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{optional_id, optional_parse, required_id, required_parse};

const FILE: &str = "fare_attributes.txt";

pub fn parse(reader: impl BufRead) -> (Vec<FareAttribute>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let fare_id = required_id::<FareId>(&row, "fare_id", FILE, line, &mut errors);
        let price = required_parse::<f64>(
            &row,
            "price",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
            &mut errors,
        );
        let currency_type =
            required_id::<CurrencyCode>(&row, "currency_type", FILE, line, &mut errors);
        let payment_method = required_parse::<u8>(
            &row,
            "payment_method",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );
        let transfers = optional_parse::<u8>(
            &row,
            "transfers",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );
        let agency_id = optional_id::<AgencyId>(&row, "agency_id");
        let transfer_duration = optional_parse::<u32>(
            &row,
            "transfer_duration",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );

        records.push(FareAttribute {
            fare_id,
            price,
            currency_type,
            payment_method,
            transfers,
            agency_id,
            transfer_duration,
        });
    }

    (records, errors)
}
