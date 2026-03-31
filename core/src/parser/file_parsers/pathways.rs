use std::io::BufRead;

use crate::models::{IsBidirectional, Pathway, PathwayId, PathwayMode, StopId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{optional_parse, optional_str, required_enum, required_id};

const FILE: &str = "pathways.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Pathway>, Vec<ParseError>) {
    let Ok(iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (line, row) in iter {
        let (pathway_id, mut e) = required_id::<PathwayId>(&row, "pathway_id", FILE, line);
        errors.append(&mut e);
        let (from_stop_id, mut e) = required_id::<StopId>(&row, "from_stop_id", FILE, line);
        errors.append(&mut e);
        let (to_stop_id, mut e) = required_id::<StopId>(&row, "to_stop_id", FILE, line);
        errors.append(&mut e);
        let (pathway_mode, mut e) = required_enum(
            &row,
            "pathway_mode",
            FILE,
            line,
            PathwayMode::from_i32,
            PathwayMode::Walkway,
        );
        errors.append(&mut e);
        let (is_bidirectional, mut e) = required_enum(
            &row,
            "is_bidirectional",
            FILE,
            line,
            IsBidirectional::from_i32,
            IsBidirectional::Unidirectional,
        );
        errors.append(&mut e);
        let (length, mut e) =
            optional_parse::<f64>(&row, "length", FILE, line, ParseErrorKind::InvalidFloat);
        errors.append(&mut e);
        let (traversal_time, mut e) = optional_parse::<u32>(
            &row,
            "traversal_time",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
        );
        errors.append(&mut e);
        let (stair_count, mut e) = optional_parse::<i32>(
            &row,
            "stair_count",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
        );
        errors.append(&mut e);
        let (max_slope, mut e) =
            optional_parse::<f64>(&row, "max_slope", FILE, line, ParseErrorKind::InvalidFloat);
        errors.append(&mut e);
        let (min_width, mut e) =
            optional_parse::<f64>(&row, "min_width", FILE, line, ParseErrorKind::InvalidFloat);
        errors.append(&mut e);
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
