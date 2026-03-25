//! Rules in this module operate on a [`FeedSource`] (raw file access) rather than
//! a fully parsed [`GtfsFeed`]. They verify the structural integrity of the feed
//! (required files, CSV headers, row lengths, etc.) **before** any data is loaded
//! into memory.
//!
//! If any rule with [`Severity::Error`] fires, the structural gate blocks feed
//! loading entirely.

mod csv_parsing_failed;
mod duplicated_column;
mod empty_column_name;
mod empty_file;
mod empty_row;
pub mod gtfs_spec;
pub(crate) mod helpers;
mod invalid_input_files_in_subfolder;
mod invalid_row_length;
mod leading_or_trailing_whitespaces;
mod missing_calendar_files;
mod missing_recommended_file;
mod missing_required_file;
mod new_line_in_value;
mod too_many_rows;
mod unknown_column;
mod unknown_file;

pub use csv_parsing_failed::CsvParsingFailedRule;
pub use duplicated_column::DuplicatedColumnRule;
pub use empty_column_name::EmptyColumnNameRule;
pub use empty_file::EmptyFileRule;
pub use empty_row::EmptyRowRule;
pub use invalid_input_files_in_subfolder::InvalidInputFilesInSubfolderRule;
pub use invalid_row_length::InvalidRowLengthRule;
pub use leading_or_trailing_whitespaces::LeadingOrTrailingWhitespacesRule;
pub use missing_calendar_files::MissingCalendarFilesRule;
pub use missing_recommended_file::MissingRecommendedFileRule;
pub use missing_required_file::MissingRequiredFileRule;
pub use new_line_in_value::NewLineInValueRule;
pub use too_many_rows::TooManyRowsRule;
pub use unknown_column::UnknownColumnRule;
pub use unknown_file::UnknownFileRule;

use crate::parser::FeedSource;
use crate::validation::{Severity, ValidationError};

/// Trait for validation rules that operate on the raw feed structure.
///
/// Unlike [`ValidationRule`](super::ValidationRule) which takes a parsed
/// `GtfsFeed`, structural rules only need access to file names and raw CSV
/// content via [`FeedSource`]. This separation enforces the architectural gate:
/// structural validation runs first, and only if it passes does CSV parsing and
/// data loading begin.
///
/// Implementations must be [`Send`] + [`Sync`] for parallel execution with
/// [rayon](https://docs.rs/rayon).
///
/// # Implementing a Rule
///
/// ```no_run
/// use headway_core::parser::FeedSource;
/// use headway_core::validation::file_structure::StructuralValidationRule;
/// use headway_core::validation::{ValidationError, Severity};
///
/// pub struct MissingRequiredFileRule;
///
/// impl StructuralValidationRule for MissingRequiredFileRule {
///     fn rule_id(&self) -> &'static str { "missing_required_file" }
///     fn section(&self) -> &'static str { "1" }
///     fn severity(&self) -> Severity { Severity::Error }
///
///     fn validate(&self, source: &FeedSource) -> Vec<ValidationError> {
///         // Check file_names() for required files...
///         vec![]
///     }
/// }
/// ```
pub trait StructuralValidationRule: Send + Sync {
    /// Unique identifier for this rule (e.g. `"missing_required_file"`).
    ///
    /// Must match the error code from the GTFS specification reference document.
    fn rule_id(&self) -> &'static str;

    /// GTFS specification section that defines this rule (e.g. `"1"`).
    fn section(&self) -> &'static str;

    /// Default severity level for findings produced by this rule.
    fn severity(&self) -> Severity;

    /// Runs the structural validation check against the raw feed source.
    ///
    /// Returns an empty `Vec` if no issues are found, or one or more
    /// [`ValidationError`]s with full diagnostic context otherwise.
    fn validate(&self, source: &FeedSource) -> Vec<ValidationError>;
}
