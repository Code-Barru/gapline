//! Field definition validation for `booking_rules.txt`.

use crate::models::{BookingType, GtfsFeed};
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "booking_rules.txt";
const SECTION: &str = "9";
const RULE_ID: &str = "field_definition_booking_rules";

/// Validates conditional field constraints for `booking_rules.txt`.
///
/// - `prior_notice_duration_min` required when `booking_type` is 1 or 2.
/// - `prior_notice_duration_min` has no effect when `booking_type` is 0
///   (Warning).
pub struct BookingRulesFieldDefinitionRule;

impl ValidationRule for BookingRulesFieldDefinitionRule {
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

        for (i, br) in feed.booking_rules.iter().enumerate() {
            let line = i + 2;
            // None: parser already reported missing/invalid; skip to avoid
            // false positives from a substituted default.
            let Some(bt) = br.booking_type else { continue };
            let has_min = br.prior_notice_duration_min.is_some();

            if matches!(bt, BookingType::SameDay | BookingType::PriorDays) && !has_min {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message(
                            "prior_notice_duration_min is required when booking_type is 1 or 2",
                        )
                        .file(FILE)
                        .line(line)
                        .field("prior_notice_duration_min"),
                );
            }
            if matches!(bt, BookingType::RealTime) && has_min {
                let min = br.prior_notice_duration_min.unwrap();
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Warning)
                        .message("prior_notice_duration_min has no effect when booking_type is 0")
                        .file(FILE)
                        .line(line)
                        .field("prior_notice_duration_min")
                        .value(min.to_string()),
                );
            }
        }

        errors
    }
}
