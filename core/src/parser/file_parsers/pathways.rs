use std::io::BufRead;

use crate::models::{IsBidirectional, Pathway, PathwayId, PathwayMode, StopId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{optional_parse, optional_str, required_enum, required_id};

const FILE: &str = "pathways.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Pathway>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let pathway_id = required_id::<PathwayId>(&row, "pathway_id", FILE, line, &mut errors);
        let from_stop_id = required_id::<StopId>(&row, "from_stop_id", FILE, line, &mut errors);
        let to_stop_id = required_id::<StopId>(&row, "to_stop_id", FILE, line, &mut errors);
        let pathway_mode = required_enum(
            &row,
            "pathway_mode",
            FILE,
            line,
            PathwayMode::from_i32,
            PathwayMode::Walkway,
            &mut errors,
        );
        let is_bidirectional = required_enum(
            &row,
            "is_bidirectional",
            FILE,
            line,
            IsBidirectional::from_i32,
            IsBidirectional::Unidirectional,
            &mut errors,
        );
        let length = optional_parse::<f64>(
            &row,
            "length",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
            &mut errors,
        );
        let traversal_time = optional_parse::<u32>(
            &row,
            "traversal_time",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );
        let stair_count = optional_parse::<i32>(
            &row,
            "stair_count",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );
        let max_slope = optional_parse::<f64>(
            &row,
            "max_slope",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
            &mut errors,
        );
        let min_width = optional_parse::<f64>(
            &row,
            "min_width",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
            &mut errors,
        );
        let signposted_as = optional_str(&row, "signposted_as");
        let reversed_signposted_as = optional_str(&row, "reversed_signposted_as");

        records.push(Pathway {
            pathway_id,
            from_stop_id,
            to_stop_id,
            pathway_mode,
            is_bidirectional,
            length,
            traversal_time,
            stair_count,
            max_slope,
            min_width,
            signposted_as,
            reversed_signposted_as,
        });
    }

    (records, errors)
}
