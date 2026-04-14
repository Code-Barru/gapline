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
mod missing_calendar_files;
mod missing_recommended_file;
mod missing_required_column;
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
pub use missing_calendar_files::MissingCalendarFilesRule;
pub use missing_recommended_file::MissingRecommendedFileRule;
pub use missing_required_column::MissingRequiredColumnRule;
pub use missing_required_file::MissingRequiredFileRule;
pub use new_line_in_value::NewLineInValueRule;
pub use too_many_rows::TooManyRowsRule;
pub use unknown_column::UnknownColumnRule;
pub use unknown_file::UnknownFileRule;

// Re-export the shared trait for backward compatibility.
pub use crate::validation::StructuralValidationRule;

/// Returns every pre-parsing rule owned by this module, ready to be appended
/// to `ValidationEngine::pre_rules`. `max_rows` parameterizes
/// [`TooManyRowsRule`] which is the only configurable structural rule here.
#[must_use]
pub fn pre_rules(max_rows: Option<usize>) -> Vec<Box<dyn StructuralValidationRule>> {
    vec![
        Box::new(MissingRequiredFileRule),
        Box::new(MissingRecommendedFileRule),
        Box::new(MissingCalendarFilesRule),
        Box::new(EmptyFileRule),
        Box::new(EmptyColumnNameRule),
        Box::new(DuplicatedColumnRule),
        Box::new(InvalidRowLengthRule),
        Box::new(InvalidInputFilesInSubfolderRule),
        Box::new(CsvParsingFailedRule),
        Box::new(TooManyRowsRule::new(max_rows)),
        Box::new(EmptyRowRule),
        Box::new(UnknownFileRule),
        Box::new(UnknownColumnRule),
        Box::new(MissingRequiredColumnRule),
    ]
}
