//! Field definition validation for Fares v2 files.
//!
//! Conditional / cross-field constraints not enforced by the parser:
//! - `fare_transfer_rules`: `duration_limit` and `duration_limit_type` are mutually required.
//! - `rider_categories`: `min_age` must not exceed `max_age` when both set.
//! - `timeframes`: `start_time` must precede `end_time`.
//! - `fare_leg_rules`: at least one matching criterion must be set (Warning otherwise).

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const SECTION: &str = "10";
const RULE_ID: &str = "field_definition_fares_v2";

pub struct FaresV2FieldDefinitionRule;

impl ValidationRule for FaresV2FieldDefinitionRule {
    fn rule_id(&self) -> &'static str {
        RULE_ID
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        check_fare_transfer_rules(feed, &mut errors);
        check_rider_categories(feed, &mut errors);
        check_timeframes(feed, &mut errors);
        check_fare_leg_rules(feed, &mut errors);
        errors
    }
}

fn check_fare_transfer_rules(feed: &GtfsFeed, errors: &mut Vec<ValidationError>) {
    const FILE: &str = "fare_transfer_rules.txt";
    for (i, ftr) in feed.fare_transfer_rules.iter().enumerate() {
        let line = i + 2;
        let has_limit = ftr.duration_limit.is_some();
        let has_type = ftr.duration_limit_type.is_some();
        if has_limit && !has_type {
            errors.push(
                ValidationError::new(RULE_ID, SECTION, Severity::Error)
                    .message("duration_limit_type is required when duration_limit is set")
                    .file(FILE)
                    .line(line)
                    .field("duration_limit_type"),
            );
        }
        if has_type && !has_limit {
            errors.push(
                ValidationError::new(RULE_ID, SECTION, Severity::Error)
                    .message("duration_limit is required when duration_limit_type is set")
                    .file(FILE)
                    .line(line)
                    .field("duration_limit"),
            );
        }
    }
}

fn check_rider_categories(feed: &GtfsFeed, errors: &mut Vec<ValidationError>) {
    const FILE: &str = "rider_categories.txt";
    for (i, rc) in feed.rider_categories.iter().enumerate() {
        let line = i + 2;
        if let (Some(min), Some(max)) = (rc.min_age, rc.max_age)
            && min > max
        {
            errors.push(
                ValidationError::new(RULE_ID, SECTION, Severity::Error)
                    .message(format!("min_age {min} exceeds max_age {max}"))
                    .file(FILE)
                    .line(line)
                    .field("min_age")
                    .value(min.to_string()),
            );
        }
    }
}

fn check_timeframes(feed: &GtfsFeed, errors: &mut Vec<ValidationError>) {
    const FILE: &str = "timeframes.txt";
    for (i, tf) in feed.timeframes.iter().enumerate() {
        let line = i + 2;
        if tf.start_time >= tf.end_time {
            errors.push(
                ValidationError::new(RULE_ID, SECTION, Severity::Error)
                    .message(format!(
                        "start_time {} is not before end_time {}",
                        tf.start_time, tf.end_time
                    ))
                    .file(FILE)
                    .line(line)
                    .field("start_time")
                    .value(tf.start_time.to_string()),
            );
        }
    }
}

fn check_fare_leg_rules(feed: &GtfsFeed, errors: &mut Vec<ValidationError>) {
    const FILE: &str = "fare_leg_rules.txt";
    for (i, flr) in feed.fare_leg_rules.iter().enumerate() {
        let line = i + 2;
        let has_criterion = flr.network_id.is_some()
            || flr.from_area_id.is_some()
            || flr.to_area_id.is_some()
            || flr.from_timeframe_group_id.is_some()
            || flr.to_timeframe_group_id.is_some();
        if !has_criterion {
            errors.push(
                ValidationError::new(RULE_ID, SECTION, Severity::Warning)
                    .message(
                        "fare_leg_rule has no matching criterion (network_id, area, or timeframe); applies unconditionally",
                    )
                    .file(FILE)
                    .line(line)
                    .field("network_id"),
            );
        }
    }
}
