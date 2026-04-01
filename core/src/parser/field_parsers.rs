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

#[must_use]
pub fn optional_str(row: &CsvRow, field: &str) -> Option<String> {
    get(row, field).map(str::to_owned)
}

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

pub fn required_id<T: From<String>>(
    row: &CsvRow,
    field: &str,
    file: &str,
    line: usize,
    errors: &mut Vec<ParseError>,
) -> T {
    if let Some(v) = get(row, field) {
        T::from(v.to_owned())
    } else {
        push_error(
            errors,
            file,
            line,
            field,
            "",
            ParseErrorKind::MissingRequired,
        );
        T::from(String::new())
    }
}

#[must_use]
pub fn optional_id<T: From<String>>(row: &CsvRow, field: &str) -> Option<T> {
    get(row, field).map(|v| T::from(v.to_owned()))
}

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

#[must_use]
pub fn bool_field(row: &CsvRow, field: &str) -> bool {
    get(row, field).is_some_and(|v| v == "1")
}
