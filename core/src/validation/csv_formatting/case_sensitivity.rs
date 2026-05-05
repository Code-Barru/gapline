//! Rule `case_sensitive_name` - rejects file names and column names that differ
//! from the canonical GTFS names only by case.

use crate::parser::{FeedSource, GtfsFiles};
use crate::validation::file_structure::helpers;
use crate::validation::{Severity, StructuralValidationRule, ValidationError};

/// All known GTFS file names for case-insensitive matching.
const ALL_GTFS_FILES: &[GtfsFiles; 31] = &[
    GtfsFiles::Agency,
    GtfsFiles::Stops,
    GtfsFiles::Routes,
    GtfsFiles::Trips,
    GtfsFiles::StopTimes,
    GtfsFiles::Calendar,
    GtfsFiles::CalendarDates,
    GtfsFiles::FareAttributes,
    GtfsFiles::FareRules,
    GtfsFiles::Timeframes,
    GtfsFiles::RiderCategories,
    GtfsFiles::FareMedia,
    GtfsFiles::FareProducts,
    GtfsFiles::FareLegRules,
    GtfsFiles::FareLegJoinRules,
    GtfsFiles::FareTransferRules,
    GtfsFiles::Areas,
    GtfsFiles::StopAreas,
    GtfsFiles::Networks,
    GtfsFiles::RouteNetworks,
    GtfsFiles::Shapes,
    GtfsFiles::Frequencies,
    GtfsFiles::Transfers,
    GtfsFiles::Pathways,
    GtfsFiles::Levels,
    GtfsFiles::LocationGroups,
    GtfsFiles::LocationGroupStops,
    GtfsFiles::BookingRules,
    GtfsFiles::Translations,
    GtfsFiles::FeedInfo,
    GtfsFiles::Attributions,
];

/// Checks that file names and column names use the exact canonical casing.
pub struct CaseSensitiveRule;

impl StructuralValidationRule for CaseSensitiveRule {
    fn rule_id(&self) -> &'static str {
        "case_sensitive_name"
    }

    fn section(&self) -> &'static str {
        "2"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, source: &FeedSource) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // --- File name casing ---
        for raw_name in source.raw_entry_names() {
            if raw_name.contains('/') {
                continue;
            }

            for &gtfs_file in ALL_GTFS_FILES {
                let canonical = gtfs_file.to_string();
                if raw_name != &canonical && raw_name.eq_ignore_ascii_case(&canonical) {
                    errors.push(
                        ValidationError::new(self.rule_id(), self.section(), self.severity())
                            .message(format!(
                                "File name has wrong casing: expected \"{canonical}\", got \"{raw_name}\""
                            ))
                            .file(raw_name.as_str())
                            .value(raw_name.as_str()),
                    );
                }
            }
        }

        // --- Column name casing ---
        for file in source.file_names() {
            let Ok(columns) = helpers::read_header(source, file) else {
                continue;
            };

            let expected = file.expected_columns();

            for col in &columns {
                let trimmed = col.trim();
                for &expected_col in expected {
                    if trimmed != expected_col && trimmed.eq_ignore_ascii_case(expected_col) {
                        errors.push(
                            ValidationError::new(
                                self.rule_id(),
                                self.section(),
                                self.severity(),
                            )
                            .message(format!(
                                "Column name has wrong casing: expected \"{expected_col}\", got \"{trimmed}\""
                            ))
                            .file(file.to_string())
                            .line(1)
                            .field(trimmed.to_string())
                            .value(trimmed.to_string()),
                        );
                    }
                }
            }
        }

        errors
    }
}
