use std::io::BufRead;

use crate::models::{BookingRule, BookingRuleId, BookingType, GtfsTime, Phone, ServiceId, Url};
use crate::parser::csv_parser::parse_csv;
use crate::parser::error::{ParseError, ParseErrorKind};
use crate::parser::field_parsers::{
    optional_id, optional_parse, optional_str, required_enum, required_id,
};

const FILE: &str = "booking_rules.txt";

pub fn parse(reader: impl BufRead) -> (Vec<BookingRule>, Vec<ParseError>) {
    let Ok(mut iter) = parse_csv(reader) else {
        return (vec![], vec![]);
    };

    let mut records = Vec::new();
    let mut errors = Vec::new();

    while let Some((line, row)) = iter.next_row() {
        let booking_rule_id =
            required_id::<BookingRuleId>(&row, "booking_rule_id", FILE, line, &mut errors);
        let booking_type = required_enum(
            &row,
            "booking_type",
            FILE,
            line,
            BookingType::from_i32,
            BookingType::RealTime,
            &mut errors,
        );
        let prior_notice_duration_min = optional_parse::<u32>(
            &row,
            "prior_notice_duration_min",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );
        let prior_notice_duration_max = optional_parse::<u32>(
            &row,
            "prior_notice_duration_max",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );
        let prior_notice_last_day = optional_parse::<u32>(
            &row,
            "prior_notice_last_day",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );
        let prior_notice_last_time = optional_parse::<GtfsTime>(
            &row,
            "prior_notice_last_time",
            FILE,
            line,
            ParseErrorKind::InvalidTime,
            &mut errors,
        );
        let prior_notice_start_day = optional_parse::<u32>(
            &row,
            "prior_notice_start_day",
            FILE,
            line,
            ParseErrorKind::InvalidInteger,
            &mut errors,
        );
        let prior_notice_start_time = optional_parse::<GtfsTime>(
            &row,
            "prior_notice_start_time",
            FILE,
            line,
            ParseErrorKind::InvalidTime,
            &mut errors,
        );
        let prior_notice_service_id = optional_id::<ServiceId>(&row, "prior_notice_service_id");
        let message = optional_str(&row, "message");
        let pickup_message = optional_str(&row, "pickup_message");
        let drop_off_message = optional_str(&row, "drop_off_message");
        let phone_number = optional_id::<Phone>(&row, "phone_number");
        let info_url = optional_id::<Url>(&row, "info_url");
        let booking_url = optional_id::<Url>(&row, "booking_url");

        records.push(BookingRule {
            booking_rule_id,
            booking_type,
            prior_notice_duration_min,
            prior_notice_duration_max,
            prior_notice_last_day,
            prior_notice_last_time,
            prior_notice_start_day,
            prior_notice_start_time,
            prior_notice_service_id,
            message,
            pickup_message,
            drop_off_message,
            phone_number,
            info_url,
            booking_url,
        });
    }

    (records, errors)
}
