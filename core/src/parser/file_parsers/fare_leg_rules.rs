use std::io::BufRead;

use crate::models::{AreaId, FareLegRule, FareProductId, LegGroupId, NetworkId, TimeframeId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{optional_id, optional_parse, required_id};

const FILE: &str = "fare_leg_rules.txt";

pub fn parse(reader: impl BufRead) -> (Vec<FareLegRule>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let leg_group_id = optional_id::<LegGroupId>(&row, "leg_group_id");
        let network_id = optional_id::<NetworkId>(&row, "network_id");
        let from_area_id = optional_id::<AreaId>(&row, "from_area_id");
        let to_area_id = optional_id::<AreaId>(&row, "to_area_id");
        let from_timeframe_group_id = optional_id::<TimeframeId>(&row, "from_timeframe_group_id");
        let to_timeframe_group_id = optional_id::<TimeframeId>(&row, "to_timeframe_group_id");
        let fare_product_id =
            required_id::<FareProductId>(&row, "fare_product_id", FILE, line, &mut errors);
        let rule_priority = optional_parse::<u32>(
            &row,
            "rule_priority",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );

        records.push(FareLegRule {
            leg_group_id,
            network_id,
            from_area_id,
            to_area_id,
            from_timeframe_group_id,
            to_timeframe_group_id,
            fare_product_id,
            rule_priority,
        });
    }

    (records, errors)
}
