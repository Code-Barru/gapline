use std::io::BufRead;

use crate::models::{
    AgencyId, Color, ContinuousDropOff, ContinuousPickup, Route, RouteId, RouteType, Url,
};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::ParseError;
use crate::parser::field_parsers::{
    optional_enum, optional_id, optional_parse, optional_str, optional_wrapper, required_enum,
    required_id,
};

const FILE: &str = "routes.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Route>, Vec<ParseError>) {
    let Ok(iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (line, row) in iter {
        let (route_id, mut e) = required_id::<RouteId>(&row, "route_id", FILE, line);
        errors.append(&mut e);
        let agency_id = optional_id::<AgencyId>(&row, "agency_id");
        let route_short_name = optional_str(&row, "route_short_name");
        let route_long_name = optional_str(&row, "route_long_name");
        let route_desc = optional_str(&row, "route_desc");
        let (route_type, mut e) = required_enum(
            &row,
            "route_type",
            FILE,
            line,
            RouteType::from_i32,
            RouteType::Bus,
        );
        errors.append(&mut e);
        let route_url = optional_wrapper::<Url>(&row, "route_url");
        let route_color = optional_wrapper::<Color>(&row, "route_color");
        let route_text_color = optional_wrapper::<Color>(&row, "route_text_color");
        let (route_sort_order, mut e) = optional_parse(
            &row,
            "route_sort_order",
            FILE,
            line,
            crate::parser::error::ParseErrorKind::InvalidInteger,
        );
        errors.append(&mut e);
        let (continuous_pickup, mut e) = optional_enum(
            &row,
            "continuous_pickup",
            FILE,
            line,
            ContinuousPickup::from_i32,
        );
        errors.append(&mut e);
        let (continuous_drop_off, mut e) = optional_enum(
            &row,
            "continuous_drop_off",
            FILE,
            line,
            ContinuousDropOff::from_i32,
        );
        errors.append(&mut e);
        let network_id = optional_str(&row, "network_id");

        records.push(Route {
            route_id,
            agency_id,
            route_short_name,
            route_long_name,
            route_desc,
            route_type,
            route_url,
            route_color,
            route_text_color,
            route_sort_order,
            continuous_pickup,
            continuous_drop_off,
            network_id,
        });
    }

    (records, errors)
}
