use std::io::BufRead;

use crate::models::{Agency, AgencyId, Email, LanguageCode, Phone, Timezone, Url};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::ParseError;
use crate::parser::field_parsers::{optional_id, required_id, required_str};

const FILE: &str = "agency.txt";

pub fn parse(reader: impl BufRead) -> (Vec<Agency>, Vec<ParseError>) {
    let Ok(iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (line, row) in iter {
        let agency_id = optional_id::<AgencyId>(&row, "agency_id");
        let (agency_name, mut e) = required_str(&row, "agency_name", FILE, line);
        errors.append(&mut e);
        let (agency_url, mut e) = required_id::<Url>(&row, "agency_url", FILE, line);
        errors.append(&mut e);
        let (agency_timezone, mut e) = required_id::<Timezone>(&row, "agency_timezone", FILE, line);
        errors.append(&mut e);
        let agency_lang = optional_id::<LanguageCode>(&row, "agency_lang");
        let agency_phone = optional_id::<Phone>(&row, "agency_phone");
        let agency_fare_url = optional_id::<Url>(&row, "agency_fare_url");
        let agency_email = optional_id::<Email>(&row, "agency_email");

        records.push(Agency {
            agency_id,
            agency_name,
            agency_url,
            agency_timezone,
            agency_lang,
            agency_phone,
            agency_fare_url,
            agency_email,
        });
    }

    (records, errors)
}
