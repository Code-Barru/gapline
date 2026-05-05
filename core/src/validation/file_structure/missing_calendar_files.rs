//! Rule `missing_calendar_and_calendar_date_files` - both calendar files are absent.

use crate::parser::{FeedSource, GtfsFiles};
use crate::validation::file_structure::StructuralValidationRule;
use crate::validation::{Severity, ValidationError};

/// Checks that at least one of `calendar.txt` or `calendar_dates.txt` is present.
///
/// The GTFS spec requires that at least one of these two files exists.
/// Produces a single `ERROR` when **both** are missing.
pub struct MissingCalendarFilesRule;

impl StructuralValidationRule for MissingCalendarFilesRule {
    fn rule_id(&self) -> &'static str {
        "missing_calendar_and_calendar_date_files"
    }

    fn section(&self) -> &'static str {
        "1"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, source: &FeedSource) -> Vec<ValidationError> {
        let present = source.file_names();
        let has_calendar = present.contains(&GtfsFiles::Calendar);
        let has_calendar_dates = present.contains(&GtfsFiles::CalendarDates);

        if !has_calendar && !has_calendar_dates {
            vec![
                ValidationError::new(self.rule_id(), self.section(), self.severity())
                    .message("Neither calendar.txt nor calendar_dates.txt is present; at least one is required"),
            ]
        } else {
            vec![]
        }
    }
}
