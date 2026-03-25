//! Rule `unknown_file` — reports files in the archive not recognized by the GTFS spec.

use crate::parser::{FeedSource, GtfsFiles};
use crate::validation::file_structure::StructuralValidationRule;
use crate::validation::{Severity, ValidationError};

/// Reports files in the feed that are not part of the GTFS specification.
///
/// Compares [`FeedSource::raw_entry_names`] against known [`GtfsFiles`] variants.
/// Files in subdirectories are stripped of their directory prefix before matching.
/// Produces one `INFO` per unknown file.
pub struct UnknownFileRule;

impl StructuralValidationRule for UnknownFileRule {
    fn rule_id(&self) -> &'static str {
        "unknown_file"
    }

    fn section(&self) -> &'static str {
        "1"
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn validate(&self, source: &FeedSource) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for raw_name in source.raw_entry_names() {
            // Strip any directory prefix to get the base file name.
            let base_name = raw_name
                .rsplit_once('/')
                .map_or(raw_name.as_str(), |(_, name)| name);

            if GtfsFiles::try_from(base_name).is_err() {
                errors.push(
                    ValidationError::new(self.rule_id(), self.section(), self.severity())
                        .message(format!("Unknown file in feed: {raw_name}"))
                        .file(raw_name),
                );
            }
        }

        errors
    }
}
