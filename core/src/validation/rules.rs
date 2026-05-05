use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError};

pub trait ValidationRule: Send + Sync {
    fn rule_id(&self) -> &'static str;
    fn section(&self) -> &'static str;
    fn severity(&self) -> Severity;
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError>;

    /// Grouping key used when rendering the progress bar. Defaults to
    /// `section`, but rules within the same GTFS section can override this
    /// to appear under a dedicated bar (e.g. geometric rules of section 7
    /// separate from temporal rules).
    fn progress_group(&self) -> &'static str {
        self.section()
    }
}
