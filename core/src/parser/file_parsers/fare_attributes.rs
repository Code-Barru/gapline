use std::io::BufRead;

use crate::models::{AgencyId, CurrencyCode, FareAttribute, FareId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{optional_id, optional_parse, required_id, required_parse};

const FILE: &str = "fare_attributes.txt";

pub fn parse(reader: impl BufRead) -> (Vec<FareAttribute>, Vec<ParseError>) {
    let Ok(iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (line, row) in iter {
        let (fare_id, mut e) = required_id::<FareId>(&row, "fare_id", FILE, line);
        errors.append(&mut e);
        let (price, mut e) =
            required_parse::<f64>(&row, "price", FILE, line, ParseErrorKind::InvalidFloat);
        errors.append(&mut e);
        let (currency_type, mut e) = required_id::<CurrencyCode>(&row, "currency_type", FILE, line);
        errors.append(&mut e);
        let (payment_method, mut e) = required_parse::<u8>(
            &row,
            "payment_method",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
        );
        errors.append(&mut e);
        let (transfers, mut e) = optional_parse::<u8>(
            &row,
            "transfers",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
        );
        errors.append(&mut e);
        let agency_id = optional_id::<AgencyId>(&row, "agency_id");
        let (transfer_duration, mut e) = optional_parse::<u32>(
            &row,
            "transfer_duration",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
        );
        errors.append(&mut e);

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
