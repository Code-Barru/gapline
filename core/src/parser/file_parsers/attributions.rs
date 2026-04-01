use std::io::BufRead;

use crate::models::{AgencyId, Attribution, Email, Phone, RouteId, TripId, Url};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{optional_id, optional_parse, optional_str, required_str};

const FILE: &str = "attributions.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Attribution>, Vec<ParseError>) {
    let Ok(iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (line, row) in iter {
        let attribution_id = optional_str(&row, "attribution_id");
        let agency_id = optional_id::<AgencyId>(&row, "agency_id");
        let route_id = optional_id::<RouteId>(&row, "route_id");
        let trip_id = optional_id::<TripId>(&row, "trip_id");
        let (organization_name, mut e) = required_str(&row, "organization_name", FILE, line);
        errors.append(&mut e);
        let (is_producer, mut e) = optional_parse::<u8>(
            &row,
            "is_producer",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
        );
        errors.append(&mut e);
        let (is_operator, mut e) = optional_parse::<u8>(
            &row,
            "is_operator",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
        );
        errors.append(&mut e);
        let (is_authority, mut e) = optional_parse::<u8>(
            &row,
            "is_authority",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
        );
        errors.append(&mut e);
        let attribution_url = optional_id::<Url>(&row, "attribution_url");
        let attribution_email = optional_id::<Email>(&row, "attribution_email");
        let attribution_phone = optional_id::<Phone>(&row, "attribution_phone");

        records.push(Attribution {
            attribution_id,
            agency_id,
            route_id,
            trip_id,
            organization_name,
            is_producer,
            is_operator,
            is_authority,
            attribution_url,
            attribution_email,
            attribution_phone,
        });
    }

    (records, errors)
}
