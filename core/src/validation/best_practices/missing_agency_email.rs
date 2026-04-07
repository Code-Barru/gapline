use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "agency.txt";
const SECTION: &str = "8";
const RULE_ID: &str = "missing_agency_email";

/// Flags agencies that do not provide an `agency_email`.
pub struct MissingAgencyEmailRule;

impl ValidationRule for MissingAgencyEmailRule {
    fn rule_id(&self) -> &'static str {
        RULE_ID
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        feed.agencies
            .iter()
            .enumerate()
            .filter(|(_, agency)| agency.agency_email.is_none())
            .map(|(i, _)| {
                ValidationError::new(RULE_ID, SECTION, Severity::Info)
                    .message("agency_email is recommended for contact purposes")
                    .file(FILE)
                    .line(i + 2)
                    .field("agency_email")
            })
            .collect()
    }
}
