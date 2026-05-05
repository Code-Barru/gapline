use std::io::BufRead;

use crate::models::{
    DurationLimitType, FareProductId, FareTransferRule, FareTransferType, LegGroupId,
};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{optional_enum, optional_id, optional_parse, required_enum};

const FILE: &str = "fare_transfer_rules.txt";

pub fn parse(reader: impl BufRead) -> (Vec<FareTransferRule>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let from_leg_group_id = optional_id::<LegGroupId>(&row, "from_leg_group_id");
        let to_leg_group_id = optional_id::<LegGroupId>(&row, "to_leg_group_id");
        let transfer_count = optional_parse::<i32>(
            &row,
            "transfer_count",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );
        let duration_limit = optional_parse::<u32>(
            &row,
            "duration_limit",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );
        let duration_limit_type = optional_enum(
            &row,
            "duration_limit_type",
            FILE,
            line,
            DurationLimitType::from_i32,
            &mut errors,
        );
        let fare_transfer_type = required_enum(
            &row,
            "fare_transfer_type",
            FILE,
            line,
            FareTransferType::from_i32,
            FareTransferType::FromLeg,
            &mut errors,
        );
        let fare_product_id = optional_id::<FareProductId>(&row, "fare_product_id");

        records.push(FareTransferRule {
            from_leg_group_id,
            to_leg_group_id,
            transfer_count,
            duration_limit,
            duration_limit_type,
            fare_transfer_type,
            fare_product_id,
        });
    }

    (records, errors)
}
