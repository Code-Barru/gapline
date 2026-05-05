//! Rule `missing_required_file` - detects absence of always-required GTFS files.

use crate::parser::FeedSource;
use crate::validation::file_structure::StructuralValidationRule;
use crate::validation::file_structure::gtfs_spec::REQUIRED_FILES;
use crate::validation::{Severity, ValidationError};

/// Checks that all always-required GTFS files are present in the feed.
///
/// Required files: `agency.txt`, `routes.txt`, `trips.txt`, `stop_times.txt`.
/// Produces one `ERROR` per missing file.
pub struct MissingRequiredFileRule;

impl StructuralValidationRule for MissingRequiredFileRule {
    fn rule_id(&self) -> &'static str {
        "missing_required_file"
    }

    fn section(&self) -> &'static str {
        "1"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, source: &FeedSource) -> Vec<ValidationError> {
        let present = source.file_names();

        REQUIRED_FILES
            .iter()
            .filter(|required| !present.contains(required))
            .map(|missing| {
                let name = missing.to_string();
                ValidationError::new(self.rule_id(), self.section(), self.severity())
                    .message(format!("Required file {name} is missing"))
                    .file(name)
            })
            .collect()
    }
}
