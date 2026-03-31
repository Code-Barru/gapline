use std::io::BufRead;

use crate::models::{Latitude, Longitude, Shape, ShapeId};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{optional_parse, required_id, required_parse};

const FILE: &str = "shapes.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Shape>, Vec<ParseError>) {
    let Ok(iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (line, row) in iter {
        let (shape_id, mut e) = required_id::<ShapeId>(&row, "shape_id", FILE, line);
        errors.append(&mut e);
        let (lat, mut e) = required_parse::<f64>(
            &row,
            "shape_pt_lat",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
        );
        errors.append(&mut e);
        let (lon, mut e) = required_parse::<f64>(
            &row,
            "shape_pt_lon",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
        );
        errors.append(&mut e);
        let (shape_pt_sequence, mut e) = required_parse::<u32>(
            &row,
            "shape_pt_sequence",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
        );
        errors.append(&mut e);
        let (shape_dist_traveled, mut e) = optional_parse::<f64>(
            &row,
            "shape_dist_traveled",
            FILE,
            line,
            ParseErrorKind::InvalidFloat,
        );
        errors.append(&mut e);

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
