//! Field definition validation for the GTFS-Flex columns of `stop_times.txt`.

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "stop_times.txt";
const SECTION: &str = "9";
const RULE_ID: &str = "field_definition_stop_times_flex";

/// Validates conditional constraints on the Flex columns of `stop_times.txt`.
///
/// - `start_pickup_drop_off_window` and `end_pickup_drop_off_window` must both
///   be set or both be unset.
/// - `pickup_booking_rule_id` / `drop_off_booking_rule_id` require a window.
pub struct StopTimesFlexFieldDefinitionRule;

impl ValidationRule for StopTimesFlexFieldDefinitionRule {
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

        for (i, st) in feed.stop_times.iter().enumerate() {
            let line = i + 2;
            let has_start = st.start_pickup_drop_off_window.is_some();
            let has_end = st.end_pickup_drop_off_window.is_some();

            if has_start && !has_end {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message("end_pickup_drop_off_window is required when start is set")
                        .file(FILE)
                        .line(line)
                        .field("end_pickup_drop_off_window"),
                );
            }
            if has_end && !has_start {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message("start_pickup_drop_off_window is required when end is set")
                        .file(FILE)
                        .line(line)
                        .field("start_pickup_drop_off_window"),
                );
            }

            let has_booking_rule =
                st.pickup_booking_rule_id.is_some() || st.drop_off_booking_rule_id.is_some();
            if has_booking_rule && !has_start {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message(
                            "pickup/drop_off_booking_rule_id requires a pickup/drop-off window",
                        )
                        .file(FILE)
                        .line(line)
                        .field("start_pickup_drop_off_window"),
                );
            }
        }

        errors
    }
}
