use std::io::BufRead;

use crate::models::{Latitude, Longitude, Shape, ShapeId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{optional_parse, required_id, required_parse};

const FILE: &str = "shapes.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Shape>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let shape_id = required_id::<ShapeId>(&row, "shape_id", FILE, line, &mut errors);
        let lat = required_parse::<f64>(
            &row,
            "shape_pt_lat",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
            &mut errors,
        );
        let lon = required_parse::<f64>(
            &row,
            "shape_pt_lon",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
            &mut errors,
        );
        let shape_pt_sequence = required_parse::<u32>(
            &row,
            "shape_pt_sequence",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );
        let shape_dist_traveled = optional_parse::<f64>(
            &row,
            "shape_dist_traveled",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
            &mut errors,
        );

        records.push(Shape {
            shape_id,
            shape_pt_lat: Latitude(lat),
            shape_pt_lon: Longitude(lon),
            shape_pt_sequence,
            shape_dist_traveled,
        });
    }

    (records, errors)
}
