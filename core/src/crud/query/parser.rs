use super::types::{Filter, Query, QueryError};

/// Parses a query expression string into a [`Query`] AST.
///
/// Supports:
/// - Equality: `field=value`
/// - Inequality: `field!=value`
/// - Comparison: `field>value`, `field>=value`, `field<value`, `field<=value`
/// - Pattern match: `field LIKE pattern` (`%` = any sequence, `_` = single char)
/// - Logical AND: `expr1 AND expr2` (higher precedence)
/// - Logical OR: `expr1 OR expr2` (lower precedence)
/// - Spaces around operators: `field = value`
/// - Backtick-quoted values: `` field=`value with AND inside` ``
///
/// # Errors
///
/// Returns [`QueryError`] if the expression is malformed (empty field/value,
/// unknown operator, dangling AND/OR).
pub fn parse(input: &str) -> Result<Query, QueryError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(QueryError::InvalidExpression(String::new()));
    }

    // Detect AND/OR at boundaries (leading, trailing, or standalone).
    check_boundary_keywords(input)?;

    let or_segments = split_keyword(input, " OR ");
    if or_segments.iter().any(|s| s.trim().is_empty()) {
        return Err(QueryError::UnexpectedOperator);
    }

    let mut or_branches = Vec::with_capacity(or_segments.len());
    for or_seg in &or_segments {
        let and_segments = split_keyword(or_seg.trim(), " AND ");
        if and_segments.iter().any(|s| s.trim().is_empty()) {
            return Err(QueryError::UnexpectedOperator);
        }

        let mut and_filters = Vec::with_capacity(and_segments.len());
        for and_seg in &and_segments {
            and_filters.push(Query::Filter(parse_filter(and_seg.trim())?));
        }
        or_branches.push(simplify_and(and_filters));
    }

    Ok(simplify_or(or_branches))
}

/// Rejects expressions that start or end with a bare `AND` / `OR` keyword.
fn check_boundary_keywords(input: &str) -> Result<(), QueryError> {
    for kw in ["AND", "OR"] {
        if input == kw || input.starts_with(&format!("{kw} ")) || input.ends_with(&format!(" {kw}"))
        {
            return Err(QueryError::UnexpectedOperator);
        }
    }
    Ok(())
}

/// Splits `input` on `keyword` while respecting backtick-quoted regions.
fn split_keyword<'a>(input: &'a str, keyword: &str) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut in_backtick = false;
    let bytes = input.as_bytes();
    let kw_bytes = keyword.as_bytes();
    let kw_len = kw_bytes.len();

    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            in_backtick = !in_backtick;
            i += 1;
        } else if !in_backtick && i + kw_len <= bytes.len() && &bytes[i..i + kw_len] == kw_bytes {
            parts.push(&input[start..i]);
            start = i + kw_len;
            i = start;
        } else {
            i += 1;
        }
    }
    parts.push(&input[start..]);
    parts
}

type FilterConstructor = fn(String, String) -> Filter;

const OPERATORS: &[(&str, FilterConstructor); 6] = &[
    ("!=", |f, v| Filter::Neq(f, v)),
    (">=", |f, v| Filter::Gte(f, v)),
    ("<=", |f, v| Filter::Lte(f, v)),
    (">", |f, v| Filter::Gt(f, v)),
    ("<", |f, v| Filter::Lt(f, v)),
    ("=", |f, v| Filter::Eq(f, v)),
];

/// Parses a single filter expression like `field=value` or `field >= value`.
fn parse_filter(segment: &str) -> Result<Filter, QueryError> {
    // `LIKE` is a word-keyword operator; pad so it's also caught at boundaries.
    let padded = format!(" {segment} ");
    let like_parts = split_keyword(&padded, " LIKE ");
    match like_parts.len() {
        1 => {}
        2 => return build_filter(segment, like_parts[0], like_parts[1], Filter::Like),
        _ => return Err(QueryError::InvalidExpression(segment.to_owned())),
    }

    for &(op, constructor) in OPERATORS {
        if let Some(pos) = find_operator(segment, op) {
            return build_filter(
                segment,
                &segment[..pos],
                &segment[pos + op.len()..],
                constructor,
            );
        }
    }

    Err(QueryError::UnknownOperator(segment.to_owned()))
}

/// Validates a `field`/`value` pair and constructs a [`Filter`] via `ctor`.
fn build_filter(
    segment: &str,
    field: &str,
    raw_value: &str,
    ctor: fn(String, String) -> Filter,
) -> Result<Filter, QueryError> {
    let field = field.trim();
    let value = strip_backticks(raw_value.trim());
    if field.is_empty() {
        return Err(QueryError::EmptyField);
    }
    if !is_valid_field_name(field) {
        return Err(QueryError::UnknownOperator(segment.to_owned()));
    }
    if value.is_empty() {
        return Err(QueryError::EmptyValue(field.to_owned()));
    }
    Ok(ctor(field.to_owned(), value.to_owned()))
}

/// Finds the position of `op` in `segment`, skipping backtick-quoted regions.
///
/// For multi-char operators, scans for the first char outside backticks, then
/// checks the full operator. For `=`, skips positions preceded by `!`, `>`, or `<`
/// to avoid matching inside `!=`, `>=`, or `<=`.
fn find_operator(segment: &str, op: &str) -> Option<usize> {
    let bytes = segment.as_bytes();
    let op_bytes = op.as_bytes();
    let op_len = op_bytes.len();
    let mut in_backtick = false;

    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            in_backtick = !in_backtick;
            i += 1;
            continue;
        }
        if in_backtick {
            i += 1;
            continue;
        }
        if i + op_len <= bytes.len() && &bytes[i..i + op_len] == op_bytes {
            // For single `=`, make sure it's not part of `!=`, `>=`, or `<=`.
            if op == "=" && i > 0 && matches!(bytes[i - 1], b'!' | b'>' | b'<') {
                i += 1;
                continue;
            }
            // For single `>` or `<`, reject doubled operators like `>>` or `<<`.
            if op_len == 1
                && matches!(op_bytes[0], b'>' | b'<')
                && i + 1 < bytes.len()
                && bytes[i + 1] == op_bytes[0]
            {
                i += 1;
                continue;
            }
            return Some(i);
        }
        i += 1;
    }
    None
}

/// A valid GTFS field name contains only ASCII letters, digits, and underscores.
fn is_valid_field_name(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// Strips surrounding backticks from a value, if present.
fn strip_backticks(value: &str) -> &str {
    if value.len() >= 2 && value.starts_with('`') && value.ends_with('`') {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

/// Simplifies `And([single])` to just the inner query.
fn simplify_and(mut items: Vec<Query>) -> Query {
    if items.len() == 1 {
        items.remove(0)
    } else {
        Query::And(items)
    }
}

/// Simplifies `Or([single])` to just the inner query.
fn simplify_or(mut items: Vec<Query>) -> Query {
    if items.len() == 1 {
        items.remove(0)
    } else {
        Query::Or(items)
    }
}
