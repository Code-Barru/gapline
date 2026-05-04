use std::collections::HashSet;

use crate::models::{DropOffType, GtfsFeed, PickupType};
use crate::validation::{Severity, ValidationError, ValidationRule};

const SECTION: &str = "9";
const STOP_TIMES: &str = "stop_times.txt";
const BOOKING_RULES: &str = "booking_rules.txt";
const LOCATION_GROUPS: &str = "location_groups.txt";

pub struct WindowOrderRule;

impl ValidationRule for WindowOrderRule {
    fn rule_id(&self) -> &'static str {
        "flex_invalid_window"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_flex() {
            return Vec::new();
        }
        let mut errors = Vec::new();
        for (i, st) in feed.stop_times.iter().enumerate() {
            let (Some(start), Some(end)) = (
                st.start_pickup_drop_off_window,
                st.end_pickup_drop_off_window,
            ) else {
                continue;
            };
            if end.total_seconds <= start.total_seconds {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Error)
                        .message("end_pickup_drop_off_window must be greater than start")
                        .file(STOP_TIMES)
                        .line(i + 2)
                        .field("end_pickup_drop_off_window")
                        .value(end.to_string()),
                );
            }
        }
        errors
    }
}

pub struct PriorNoticeMinPositiveRule;

impl ValidationRule for PriorNoticeMinPositiveRule {
    fn rule_id(&self) -> &'static str {
        "flex_invalid_prior_notice"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_flex() {
            return Vec::new();
        }
        let mut errors = Vec::new();
        for (i, br) in feed.booking_rules.iter().enumerate() {
            if br.prior_notice_duration_min == Some(0) {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Error)
                        .message("prior_notice_duration_min must be > 0")
                        .file(BOOKING_RULES)
                        .line(i + 2)
                        .field("prior_notice_duration_min")
                        .value("0"),
                );
            }
        }
        errors
    }
}

pub struct PriorNoticeMinMaxRule;

impl ValidationRule for PriorNoticeMinMaxRule {
    fn rule_id(&self) -> &'static str {
        "flex_invalid_prior_notice_range"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_flex() {
            return Vec::new();
        }
        let mut errors = Vec::new();
        for (i, br) in feed.booking_rules.iter().enumerate() {
            let (Some(min), Some(max)) =
                (br.prior_notice_duration_min, br.prior_notice_duration_max)
            else {
                continue;
            };
            if max < min {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Error)
                        .message("prior_notice_duration_max must be >= min")
                        .file(BOOKING_RULES)
                        .line(i + 2)
                        .field("prior_notice_duration_max")
                        .value(max.to_string()),
                );
            }
        }
        errors
    }
}

pub struct PriorNoticeLastDayTimeRule;

impl ValidationRule for PriorNoticeLastDayTimeRule {
    fn rule_id(&self) -> &'static str {
        "flex_incomplete_prior_notice_last"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_flex() {
            return Vec::new();
        }
        let mut errors = Vec::new();
        for (i, br) in feed.booking_rules.iter().enumerate() {
            if br.prior_notice_last_day.is_some() && br.prior_notice_last_time.is_none() {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Warning)
                        .message("prior_notice_last_day set without prior_notice_last_time")
                        .file(BOOKING_RULES)
                        .line(i + 2)
                        .field("prior_notice_last_time"),
                );
            }
        }
        errors
    }
}

pub struct EmptyLocationGroupRule;

impl ValidationRule for EmptyLocationGroupRule {
    fn rule_id(&self) -> &'static str {
        "flex_empty_location_group"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_flex() {
            return Vec::new();
        }
        let populated: HashSet<&str> = feed
            .location_group_stops
            .iter()
            .map(|lgs| lgs.location_group_id.as_ref())
            .collect();
        let mut errors = Vec::new();
        for (i, lg) in feed.location_groups.iter().enumerate() {
            if !populated.contains(lg.location_group_id.as_ref()) {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Warning)
                        .message("location_group has no entries in location_group_stops.txt")
                        .file(LOCATION_GROUPS)
                        .line(i + 2)
                        .field("location_group_id")
                        .value(lg.location_group_id.to_string()),
                );
            }
        }
        errors
    }
}

pub struct MeanDurationFactorPositiveRule;

impl ValidationRule for MeanDurationFactorPositiveRule {
    fn rule_id(&self) -> &'static str {
        "flex_invalid_duration_factor"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_flex() {
            return Vec::new();
        }
        let mut errors = Vec::new();
        for (i, st) in feed.stop_times.iter().enumerate() {
            if let Some(mean) = st.mean_duration_factor
                && mean < 0.0
            {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Error)
                        .message("mean_duration_factor must be >= 0")
                        .file(STOP_TIMES)
                        .line(i + 2)
                        .field("mean_duration_factor")
                        .value(mean.to_string()),
                );
            }
        }
        errors
    }
}

pub struct SafeDurationFactorRule;

impl ValidationRule for SafeDurationFactorRule {
    fn rule_id(&self) -> &'static str {
        "flex_unsafe_duration"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_flex() {
            return Vec::new();
        }
        let mut errors = Vec::new();
        for (i, st) in feed.stop_times.iter().enumerate() {
            let (Some(mean), Some(safe)) = (st.mean_duration_factor, st.safe_duration_factor)
            else {
                continue;
            };
            if safe < mean {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Warning)
                        .message("safe_duration_factor must be >= mean_duration_factor")
                        .file(STOP_TIMES)
                        .line(i + 2)
                        .field("safe_duration_factor")
                        .value(safe.to_string()),
                );
            }
        }
        errors
    }
}

pub struct ScheduledWithBookingRuleRule;

impl ValidationRule for ScheduledWithBookingRuleRule {
    fn rule_id(&self) -> &'static str {
        "flex_inconsistent_booking"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_flex() {
            return Vec::new();
        }
        let mut errors = Vec::new();
        for (i, st) in feed.stop_times.iter().enumerate() {
            let line = i + 2;
            // GTFS spec: absent pickup_type/drop_off_type defaults to Regular.
            let pickup_regular = matches!(st.pickup_type, None | Some(PickupType::Regular));
            if pickup_regular && st.pickup_booking_rule_id.is_some() {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Warning)
                        .message("pickup_booking_rule_id set with scheduled pickup_type")
                        .file(STOP_TIMES)
                        .line(line)
                        .field("pickup_booking_rule_id"),
                );
            }
            let drop_off_regular = matches!(st.drop_off_type, None | Some(DropOffType::Regular));
            if drop_off_regular && st.drop_off_booking_rule_id.is_some() {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Warning)
                        .message("drop_off_booking_rule_id set with scheduled drop_off_type")
                        .file(STOP_TIMES)
                        .line(line)
                        .field("drop_off_booking_rule_id"),
                );
            }
        }
        errors
    }
}
