//! Transfer validation (section 7.9).
//!
//! Checks transfer distance coherence, self-transfers, and zero-time
//! transfers with `transfer_type=2`.

use std::collections::HashMap;

use crate::geo::haversine_meters;
use crate::models::GtfsFeed;
use crate::models::TransferType;
use crate::validation::schedule_time_validation::TransferThresholds;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "transfers.txt";
const SECTION: &str = "7";

/// Validates transfers for distance anomalies and logical inconsistencies.
pub struct TransferValidationRule {
    max_distance_m: f64,
    warning_distance_m: f64,
}

impl TransferValidationRule {
    #[must_use]
    pub fn new(thresholds: TransferThresholds) -> Self {
        Self {
            max_distance_m: thresholds.max_transfer_distance_m,
            warning_distance_m: thresholds.transfer_distance_warning_m,
        }
    }
}

impl ValidationRule for TransferValidationRule {
    fn rule_id(&self) -> &'static str {
        "transfer_validation"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let stops_by_id: HashMap<&str, _> =
            feed.stops.iter().map(|s| (s.stop_id.as_ref(), s)).collect();

        let mut errors = Vec::new();

        for (i, transfer) in feed.transfers.iter().enumerate() {
            let line = i + 2;

            let is_self = match (&transfer.from_stop_id, &transfer.to_stop_id) {
                (Some(from), Some(to)) => from.as_ref() == to.as_ref(),
                _ => false,
            };

            if is_self {
                let stop_id = transfer.from_stop_id.as_ref().unwrap();
                errors.push(
                    ValidationError::new("self_transfer", SECTION, Severity::Warning)
                        .message(format!("Transfer from stop '{stop_id}' to itself",))
                        .file(FILE)
                        .line(line)
                        .field("from_stop_id")
                        .value(stop_id.as_ref()),
                );
            }

            if transfer.transfer_type == TransferType::MinimumTime
                && transfer.min_transfer_time == Some(0)
            {
                errors.push(
                    ValidationError::new("zero_transfer_time", SECTION, Severity::Warning)
                        .message(
                            "Transfer with transfer_type=2 (minimum time) has \
                             min_transfer_time=0",
                        )
                        .file(FILE)
                        .line(line)
                        .field("min_transfer_time")
                        .value("0"),
                );
            }

            if is_self {
                continue;
            }
            let (Some(from_id), Some(to_id)) = (&transfer.from_stop_id, &transfer.to_stop_id)
            else {
                continue;
            };
            let (Some(from_stop), Some(to_stop)) = (
                stops_by_id.get(from_id.as_ref()),
                stops_by_id.get(to_id.as_ref()),
            ) else {
                continue;
            };
            let (Some(lat1), Some(lon1)) = (from_stop.stop_lat, from_stop.stop_lon) else {
                continue;
            };
            let (Some(lat2), Some(lon2)) = (to_stop.stop_lat, to_stop.stop_lon) else {
                continue;
            };

            let dist = haversine_meters(lat1.0, lon1.0, lat2.0, lon2.0);

            if dist > self.max_distance_m {
                errors.push(
                    ValidationError::new("transfer_distance_too_large", SECTION, Severity::Error)
                        .message(format!(
                            "Transfer from '{from_id}' to '{to_id}' is {dist:.0}m apart \
                         (threshold {:.0}m)",
                            self.max_distance_m,
                        ))
                        .file(FILE)
                        .line(line)
                        .field("from_stop_id")
                        .value(from_id.as_ref()),
                );
            } else if dist > self.warning_distance_m {
                errors.push(
                    ValidationError::new(
                        "transfer_distance_suspicious",
                        SECTION,
                        Severity::Warning,
                    )
                    .message(format!(
                        "Transfer from '{from_id}' to '{to_id}' is {dist:.0}m apart \
                         (warning threshold {:.0}m)",
                        self.warning_distance_m,
                    ))
                    .file(FILE)
                    .line(line)
                    .field("from_stop_id")
                    .value(from_id.as_ref()),
                );
            }
        }

        errors
    }
}
