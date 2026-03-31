use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError};

pub trait ValidationRule: Send + Sync {
    fn rule_id(&self) -> &'static str;
    fn section(&self) -> &'static str;
    fn severity(&self) -> Severity;
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError>;
}
