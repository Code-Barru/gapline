//! FK rule: `fare_attributes.agency_id` → `agency.agency_id`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "fare_attributes.txt";
const SECTION: &str = "5";
const RULE_ID: &str = "foreign_key_violation";

/// If `agency_id` is non-empty in `fare_attributes.txt`, it must exist in agency.txt.
pub struct FareAttributesAgencyFkRule;

impl ValidationRule for FareAttributesAgencyFkRule {
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
        let valid_ids: HashSet<&str> = feed
            .agencies
            .iter()
            .filter_map(|a| a.agency_id.as_ref().map(AsRef::as_ref))
            .collect();

        feed.fare_attributes
            .iter()
            .enumerate()
            .filter_map(|(i, fa)| {
                let id = fa.agency_id.as_ref()?;
                if valid_ids.contains(id.as_ref()) {
                    return None;
                }
                let line = i + 2;
                Some(
                    ValidationError::new(RULE_ID, SECTION, Severity::Error)
                        .message(format!(
                            "agency_id '{id}' in fare_attributes.txt line {line} references non-existent agency in agency.txt"
                        ))
                        .file(FILE)
                        .line(line)
                        .field("agency_id")
                        .value(id.as_ref()),
                )
            })
            .collect()
    }
}
