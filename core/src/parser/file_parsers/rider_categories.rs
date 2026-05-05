use std::io::BufRead;

use crate::models::{RiderCategory, RiderCategoryId, Url};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{optional_id, optional_parse, required_id, required_str};

const FILE: &str = "rider_categories.txt";

pub fn parse(reader: impl BufRead) -> (Vec<RiderCategory>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let rider_category_id =
            required_id::<RiderCategoryId>(&row, "rider_category_id", FILE, line, &mut errors);
        let rider_category_name =
            required_str(&row, "rider_category_name", FILE, line, &mut errors);
        let min_age = optional_parse::<u32>(
            &row,
            "min_age",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );
        let max_age = optional_parse::<u32>(
            &row,
            "max_age",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );
        let eligibility_url = optional_id::<Url>(&row, "eligibility_url");

        records.push(RiderCategory {
            rider_category_id,
            rider_category_name,
            min_age,
            max_age,
            eligibility_url,
        });
    }

    (records, errors)
}
