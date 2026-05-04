//! Low-level helpers used by every per-file GTFS parser.
//!
//! Each function reads a single field from a [`CsvRow`], converts it to the
//! target type, and — on failure — pushes a structured [`ParseError`] into the
//! caller's error buffer before returning a default value. This keeps the
//! per-file parsers (`core/src/parser/file_parsers/*.rs`) uniform: they walk
//! the rows, call one helper per column, and collect every error in a single
//! pass instead of short-circuiting on the first problem.
//!
//! All `required_*` helpers emit a `MissingRequired` error when the column is
//! absent or empty. All `optional_*` helpers return `None` silently in that
//! case. Conversion failures (parse/enum) emit the caller-supplied
//! [`ParseErrorKind`] so different columns can be reported with appropriate
//! taxonomy.

use std::str::FromStr;

use crate::parser::csv_parser::CsvRow;
use crate::parser::error::{ParseError, ParseErrorKind};

/// Returns the raw `&str` value for `field` from `row`, or `None` if missing/empty.
#[inline]
#[must_use]
pub fn get<'a>(row: &'a CsvRow, field: &str) -> Option<&'a str> {
    row.get(field)
}

fn push_error(
    errors: &mut Vec<ParseError>,
    file: &str,
    line: usize,
    field: &str,
    value: &str,
    kind: ParseErrorKind,
) {
    errors.push(ParseError {
        file_name: file.to_owned(),
        line_number: line,
        field_name: field.to_owned(),
        value: value.to_owned(),
        kind,
    });
}

/// Reads a required string column, returning an empty `String` and emitting
/// `MissingRequired` when the column is absent.
pub fn required_str(
    row: &CsvRow,
    field: &str,
    file: &str,
    line: usize,
    errors: &mut Vec<ParseError>,
) -> String {
    if let Some(v) = get(row, field) {
        v.to_owned()
    } else {
        push_error(
            errors,
            file,
            line,
            field,
            "",
            ParseErrorKind::MissingRequired,
        );
        String::new()
    }
}

/// Reads an optional string column. Returns `None` when the column is absent.
#[must_use]
pub fn optional_str(row: &CsvRow, field: &str) -> Option<String> {
    get(row, field).map(str::to_owned)
}

/// Parses a required typed column with `T::from_str`. On missing or invalid
/// input, emits an error (`MissingRequired` or the supplied `err_kind`) and
/// returns `T::default()`.
pub fn required_parse<T: FromStr + Default>(
    row: &CsvRow,
    field: &str,
    file: &str,
    line: usize,
    err_kind: ParseErrorKind,
    errors: &mut Vec<ParseError>,
) -> T {
    if let Some(v) = get(row, field) {
        if let Ok(parsed) = v.parse() {
            parsed
        } else {
            push_error(errors, file, line, field, v, err_kind);
            T::default()
        }
    } else {
        push_error(
            errors,
            file,
            line,
            field,
            "",
            ParseErrorKind::MissingRequired,
        );
        T::default()
    }
}

/// Parses an optional typed column. Missing columns silently return `None`.
/// Invalid values emit `err_kind` and also return `None`.
pub fn optional_parse<T: FromStr>(
    row: &CsvRow,
    field: &str,
    file: &str,
    line: usize,
    err_kind: ParseErrorKind,
    errors: &mut Vec<ParseError>,
) -> Option<T> {
    let v = get(row, field)?;
    if let Ok(parsed) = v.parse() {
        Some(parsed)
    } else {
        push_error(errors, file, line, field, v, err_kind);
        None
    }
}

/// Reads a required ID column (anything constructible via `From<&str>`).
/// Missing values produce `MissingRequired` and return `T::from("")`.
pub fn required_id<T: for<'a> From<&'a str>>(
    row: &CsvRow,
    field: &str,
    file: &str,
    line: usize,
    errors: &mut Vec<ParseError>,
) -> T {
    if let Some(v) = get(row, field) {
        T::from(v)
    } else {
        push_error(
            errors,
            file,
            line,
            field,
            "",
            ParseErrorKind::MissingRequired,
        );
        T::from("")
    }
}

/// Reads an optional ID column. Returns `None` when the column is absent.
#[must_use]
pub fn optional_id<T: for<'a> From<&'a str>>(row: &CsvRow, field: &str) -> Option<T> {
    get(row, field).map(T::from)
}

/// Parses a required enum column encoded as an `i32` code. `from_i32` maps
/// valid codes to enum variants; unknown codes emit `InvalidEnum` and return
/// `default`. Missing columns emit `MissingRequired` and also return `default`.
pub fn required_enum<T>(
    row: &CsvRow,
    field: &str,
    file: &str,
    line: usize,
    from_i32: fn(i32) -> Option<T>,
    default: T,
    errors: &mut Vec<ParseError>,
) -> T {
    if let Some(v) = get(row, field) {
        if let Some(e) = v.parse::<i32>().ok().and_then(from_i32) {
            e
        } else {
            push_error(errors, file, line, field, v, ParseErrorKind::InvalidEnum);
            default
        }
    } else {
        push_error(
            errors,
            file,
            line,
            field,
            "",
            ParseErrorKind::MissingRequired,
        );
        default
    }
}

/// Parses an optional enum column encoded as an `i32` code. Missing columns
/// return `None` silently; unknown codes emit `InvalidEnum` and return `None`.
pub fn optional_enum<T>(
    row: &CsvRow,
    field: &str,
    file: &str,
    line: usize,
    from_i32: fn(i32) -> Option<T>,
    errors: &mut Vec<ParseError>,
) -> Option<T> {
    let v = get(row, field)?;
    if let Some(e) = v.parse::<i32>().ok().and_then(from_i32) {
        Some(e)
    } else {
        push_error(errors, file, line, field, v, ParseErrorKind::InvalidEnum);
        None
    }
}

/// Like [`required_enum`] but returns `Option<T>` instead of a default on
/// failure. Missing → `MissingRequired`, unknown code → `InvalidEnum`.
pub fn required_enum_opt<T>(
    row: &CsvRow,
    field: &str,
    file: &str,
    line: usize,
    from_i32: fn(i32) -> Option<T>,
    errors: &mut Vec<ParseError>,
) -> Option<T> {
    if let Some(v) = get(row, field) {
        if let Some(e) = v.parse::<i32>().ok().and_then(from_i32) {
            Some(e)
        } else {
            push_error(errors, file, line, field, v, ParseErrorKind::InvalidEnum);
            None
        }
    } else {
        push_error(
            errors,
            file,
            line,
            field,
            "",
            ParseErrorKind::MissingRequired,
        );
        None
    }
}

/// Reads a GTFS boolean column: `"1"` → `true`, anything else (including
/// absent) → `false`. No error is emitted for this column type.
#[must_use]
pub fn bool_field(row: &CsvRow, field: &str) -> bool {
    get(row, field).is_some_and(|v| v == "1")
}
