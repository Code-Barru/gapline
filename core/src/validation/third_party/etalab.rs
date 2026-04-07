use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "feed_info.txt";
const SECTION: &str = "13";
const RULE_ID: &str = "etalab_missing_contact";

/// Flags feeds missing `feed_contact_email` in `feed_info.txt`.
pub struct EtalabMissingContactRule;

impl ValidationRule for EtalabMissingContactRule {
    fn rule_id(&self) -> &'static str {
        RULE_ID
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let Some(info) = &feed.feed_info else {
            return Vec::new();
        };

        if info.feed_contact_email.is_none() {
            vec![
                ValidationError::new(RULE_ID, SECTION, Severity::Warning)
                    .message("feed_contact_email is recommended for contact purposes")
                    .file(FILE)
                    .line(2)
                    .field("feed_contact_email"),
            ]
        } else {
            Vec::new()
        }
    }
}
