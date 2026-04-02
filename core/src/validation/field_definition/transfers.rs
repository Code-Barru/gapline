//! Field definition validation for `transfers.txt`.

use crate::models::GtfsFeed;
use crate::models::TransferType;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "transfers.txt";
const SECTION: &str = "4";
const RULE_ID: &str = "field_definition_transfers";

/// Validates conditional field constraints for `transfers.txt`.
///
/// - `from_stop_id` is required.
/// - `to_stop_id` is required.
/// - `min_transfer_time` is required when `transfer_type` is `MinimumTime` (2).
pub struct TransfersFieldDefinitionRule;

impl ValidationRule for TransfersFieldDefinitionRule {
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
        if !feed.has_file("transfers.txt") {
            return Vec::new();
        }

        let mut errors = Vec::new();

        for (i, transfer) in feed.transfers.iter().enumerate() {
            let line = i + 2;

            if transfer
                .from_stop_id
                .as_ref()
                .is_none_or(|id| id.as_ref().is_empty())
            {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message("from_stop_id is required")
                        .file(FILE)
                        .line(line)
                        .field("from_stop_id"),
                );
            }

            if transfer
                .to_stop_id
                .as_ref()
                .is_none_or(|id| id.as_ref().is_empty())
            {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message("to_stop_id is required")
                        .file(FILE)
                        .line(line)
                        .field("to_stop_id"),
                );
            }

            if transfer.transfer_type == TransferType::MinimumTime
                && transfer.min_transfer_time.is_none()
            {
                errors.push(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message(
                            "min_transfer_time is required when transfer_type is 2 (MinimumTime)",
                        )
                        .file(FILE)
                        .line(line)
                        .field("min_transfer_time"),
                );
            }
        }

        errors
    }
}
