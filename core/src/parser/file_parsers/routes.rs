use std::io::BufRead;

use crate::models::{
    AgencyId, Color, ContinuousDropOff, ContinuousPickup, Route, RouteId, RouteType, Url,
};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{
    optional_enum, optional_id, optional_parse, optional_str, required_enum, required_id,
};

const FILE: &str = "routes.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Route>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let route_id = required_id::<RouteId>(&row, "route_id", FILE, line, &mut errors);
        let agency_id = optional_id::<AgencyId>(&row, "agency_id");
        let route_short_name = optional_str(&row, "route_short_name");
        let route_long_name = optional_str(&row, "route_long_name");
        let route_desc = optional_str(&row, "route_desc");
        let route_type = required_enum(
            &row,
            "route_type",
            FILE,
            line,
            RouteType::from_i32,
            RouteType::Bus,
            &mut errors,
        );
        let route_url = optional_id::<Url>(&row, "route_url");
        let route_color = optional_id::<Color>(&row, "route_color");
        let route_text_color = optional_id::<Color>(&row, "route_text_color");
        let route_sort_order = optional_parse(
            &row,
            "route_sort_order",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );
        let continuous_pickup = optional_enum(
            &row,
            "continuous_pickup",
            FILE,
            line,
            ContinuousPickup::from_i32,
            &mut errors,
        );
        let continuous_drop_off = optional_enum(
            &row,
            "continuous_drop_off",
            FILE,
            line,
            ContinuousDropOff::from_i32,
            &mut errors,
        );
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
