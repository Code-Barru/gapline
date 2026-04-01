#![allow(clippy::implicit_hasher)]

use std::collections::HashMap;
use std::str::FromStr;

use crate::parser::error::{ParseError, ParseErrorKind};

fn get(row: &HashMap<String, String>, field: &str) -> Option<String> {
    row.get(field).filter(|v| !v.is_empty()).cloned()
}

fn make_error(
    file: &str,
    line: usize,
    field: &str,
    value: &str,
    kind: ParseErrorKind,
) -> ParseError {
    ParseError {
        file_name: file.to_owned(),
        line_number: line,
        field_name: field.to_owned(),
        value: value.to_owned(),
        kind,
    }
}

#[must_use]
pub fn required_str(
    row: &HashMap<String, String>,
    field: &str,
    file: &str,
    line: usize,
) -> (String, Vec<ParseError>) {
    match get(row, field) {
        Some(v) => (v, vec![]),
        None => (
            String::new(),
            vec![make_error(
                file,
                line,
                field,
                "",
                ParseErrorKind::MissingRequired,
            )],
        ),
    }
}

#[must_use]
pub fn optional_str(row: &HashMap<String, String>, field: &str) -> Option<String> {
    get(row, field)
}

#[must_use]
pub fn required_parse<T: FromStr + Default>(
    row: &HashMap<String, String>,
    field: &str,
    file: &str,
    line: usize,
    err_kind: ParseErrorKind,
) -> (T, Vec<ParseError>) {
    match get(row, field) {
        Some(v) => match v.parse() {
            Ok(parsed) => (parsed, vec![]),
            Err(_) => (
                T::default(),
                vec![make_error(file, line, field, &v, err_kind)],
            ),
        },
        None => (
            T::default(),
            vec![make_error(
                file,
                line,
                field,
                "",
                ParseErrorKind::MissingRequired,
            )],
        ),
    }
}

#[must_use]
pub fn optional_parse<T: FromStr>(
    row: &HashMap<String, String>,
    field: &str,
    file: &str,
    line: usize,
    err_kind: ParseErrorKind,
) -> (Option<T>, Vec<ParseError>) {
    match get(row, field) {
        Some(v) => match v.parse() {
            Ok(parsed) => (Some(parsed), vec![]),
            Err(_) => (None, vec![make_error(file, line, field, &v, err_kind)]),
        },
        None => (None, vec![]),
    }
}

#[must_use]
pub fn required_id<T: From<String>>(
    row: &HashMap<String, String>,
    field: &str,
    file: &str,
    line: usize,
) -> (T, Vec<ParseError>) {
    match get(row, field) {
        Some(v) => (T::from(v), vec![]),
        None => (
            T::from(String::new()),
            vec![make_error(
                file,
                line,
                field,
                "",
                ParseErrorKind::MissingRequired,
            )],
        ),
    }
}

pub fn optional_id<T: From<String>>(row: &HashMap<String, String>, field: &str) -> Option<T> {
    get(row, field).map(T::from)
}

pub fn required_enum<T>(
    row: &HashMap<String, String>,
    field: &str,
    file: &str,
    line: usize,
    from_i32: fn(i32) -> Option<T>,
    default: T,
) -> (T, Vec<ParseError>) {
    match get(row, field) {
        Some(v) => {
            let parsed = v.parse::<i32>().ok().and_then(from_i32);
            match parsed {
                Some(e) => (e, vec![]),
                None => (
                    default,
                    vec![make_error(
                        file,
                        line,
                        field,
                        &v,
                        ParseErrorKind::InvalidEnum,
                    )],
                ),
            }
        }
        None => (
            default,
            vec![make_error(
                file,
                line,
                field,
                "",
                ParseErrorKind::MissingRequired,
            )],
        ),
    }
}

pub fn optional_enum<T>(
    row: &HashMap<String, String>,
    field: &str,
    file: &str,
    line: usize,
    from_i32: fn(i32) -> Option<T>,
) -> (Option<T>, Vec<ParseError>) {
    match get(row, field) {
        Some(v) => {
            let parsed = v.parse::<i32>().ok().and_then(from_i32);
            match parsed {
                Some(e) => (Some(e), vec![]),
                None => (
                    None,
                    vec![make_error(
                        file,
                        line,
                        field,
                        &v,
                        ParseErrorKind::InvalidEnum,
                    )],
                ),
            }
        }
        None => (None, vec![]),
    }
}

#[must_use]
pub fn bool_field(row: &HashMap<String, String>, field: &str) -> bool {
    get(row, field).is_some_and(|v| v == "1")
}
