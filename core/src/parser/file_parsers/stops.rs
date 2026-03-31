use std::io::BufRead;

use crate::models::{
    Latitude, LevelId, LocationType, Longitude, Stop, StopId, Timezone, Url, WheelchairAccessible,
};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{
    optional_enum, optional_id, optional_parse, optional_str, optional_wrapper, required_id,
};

const FILE: &str = "stops.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Stop>, Vec<ParseError>) {
    let Ok(iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (line, row) in iter {
        let (stop_id, mut e) = required_id::<StopId>(&row, "stop_id", FILE, line);
        errors.append(&mut e);
        let stop_code = optional_str(&row, "stop_code");
        let stop_name = optional_str(&row, "stop_name");
        let tts_stop_name = optional_str(&row, "tts_stop_name");
        let stop_desc = optional_str(&row, "stop_desc");
        let (stop_lat, mut e) =
            optional_parse::<f64>(&row, "stop_lat", FILE, line, ParseErrorKind::InvalidFloat);
        errors.append(&mut e);
        let (stop_lon, mut e) =
            optional_parse::<f64>(&row, "stop_lon", FILE, line, ParseErrorKind::InvalidFloat);
        errors.append(&mut e);
        let zone_id = optional_str(&row, "zone_id");
        let stop_url = optional_wrapper::<Url>(&row, "stop_url");
        let (location_type, mut e) =
            optional_enum(&row, "location_type", FILE, line, LocationType::from_i32);
        errors.append(&mut e);
        let parent_station = optional_id::<StopId>(&row, "parent_station");
        let stop_timezone = optional_wrapper::<Timezone>(&row, "stop_timezone");
        let (wheelchair_boarding, mut e) = optional_enum(
            &row,
            "wheelchair_boarding",
            FILE,
            line,
            WheelchairAccessible::from_i32,
        );
        errors.append(&mut e);
        let level_id = optional_id::<LevelId>(&row, "level_id");
        let platform_code = optional_str(&row, "platform_code");

        records.push(Stop {
            stop_id,
            stop_code,
            stop_name,
            tts_stop_name,
            stop_desc,
            stop_lat: stop_lat.map(Latitude),
            stop_lon: stop_lon.map(Longitude),
            zone_id,
            stop_url,
            location_type,
            parent_station,
            stop_timezone,
            wheelchair_boarding,
            level_id,
            platform_code,
        });
    }

    (records, errors)
}
