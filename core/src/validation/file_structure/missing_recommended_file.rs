//! Rule `missing_recommended_file` - warns when recommended files are absent.

use crate::parser::FeedSource;
use crate::validation::file_structure::StructuralValidationRule;
use crate::validation::file_structure::gtfs_spec::RECOMMENDED_FILES;
use crate::validation::{Severity, ValidationError};

/// Checks that recommended GTFS files are present in the feed.
///
/// Recommended files: `feed_info.txt`, `shapes.txt`.
/// Produces one `WARNING` per missing file.
pub struct MissingRecommendedFileRule;

impl StructuralValidationRule for MissingRecommendedFileRule {
    fn rule_id(&self) -> &'static str {
        "missing_recommended_file"
    }

    fn section(&self) -> &'static str {
        "1"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, source: &FeedSource) -> Vec<ValidationError> {
        let present = source.file_names();

        RECOMMENDED_FILES
            .iter()
            .filter(|rec| !present.contains(rec))
            .map(|missing| {
                let name = missing.to_string();
                ValidationError::new(self.rule_id(), self.section(), self.severity())
                    .message(format!("Recommended file {name} is missing"))
                    .file(name)
            })
            .collect()
    }
}
