use std::io::BufRead;

use crate::models::{CurrencyCode, FareMediaId, FareProduct, FareProductId, RiderCategoryId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{optional_id, optional_str, required_id, required_parse};

const FILE: &str = "fare_products.txt";

pub fn parse(reader: impl BufRead) -> (Vec<FareProduct>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let fare_product_id =
            required_id::<FareProductId>(&row, "fare_product_id", FILE, line, &mut errors);
        let fare_product_name = optional_str(&row, "fare_product_name");
        let fare_media_id = optional_id::<FareMediaId>(&row, "fare_media_id");
        let amount = required_parse::<f64>(
            &row,
            "amount",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
            &mut errors,
        );
        let currency = required_id::<CurrencyCode>(&row, "currency", FILE, line, &mut errors);
        let rider_category_id = optional_id::<RiderCategoryId>(&row, "rider_category_id");

        records.push(FareProduct {
            fare_product_id,
            fare_product_name,
            fare_media_id,
            amount,
            currency,
            rider_category_id,
        });
    }

    (records, errors)
}
