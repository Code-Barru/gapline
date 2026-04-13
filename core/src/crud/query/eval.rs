use super::filterable::Filterable;
use super::types::{Filter, Query, QueryError};

impl Query {
    /// Evaluates the query against a record implementing [`Filterable`].
    ///
    /// - `Eq`: matches when the field value equals the filter value.
    /// - `Neq`: matches when the field value differs (a `None` field is considered different).
    /// - `Gt`/`Gte`/`Lt`/`Lte`: attempt numeric (`f64`) comparison first; fall back to
    ///   lexicographic ordering if either side is not a valid number. A `None` field never matches.
    /// - `And`: all sub-queries must match.
    /// - `Or`: at least one sub-query must match.
    pub fn matches(&self, record: &impl Filterable) -> bool {
        match self {
            Self::Filter(filter) => eval_filter(filter, record),
            Self::And(queries) => queries.iter().all(|q| q.matches(record)),
            Self::Or(queries) => queries.iter().any(|q| q.matches(record)),
        }
    }

    /// Validates that every field referenced in this query is recognized by `T`.
    ///
    /// # Errors
    ///
    /// Returns [`QueryError::UnknownField`] if any referenced field is not in
    /// `T::valid_fields()`.
    pub fn validate_fields<T: Filterable>(&self) -> Result<(), QueryError> {
        let valid = T::valid_fields();
        for field in self.fields() {
            if !valid.contains(&field.as_str()) {
                return Err(QueryError::UnknownField {
                    field: field.clone(),
                    valid: valid.to_vec(),
                });
            }
        }
        Ok(())
    }

    /// Collects all field names referenced in this query.
    fn fields(&self) -> Vec<String> {
        let mut out = Vec::new();
        self.collect_fields(&mut out);
        out
    }

    fn collect_fields(&self, out: &mut Vec<String>) {
        match self {
            Self::Filter(f) => out.push(f.field().to_owned()),
            Self::And(qs) | Self::Or(qs) => {
                for q in qs {
                    q.collect_fields(out);
                }
            }
        }
    }
}

/// Evaluates a single [`Filter`] against a record.
fn eval_filter(filter: &Filter, record: &impl Filterable) -> bool {
    match filter {
        Filter::Eq(field, value) => record.field_value(field).as_deref() == Some(value.as_str()),
        Filter::Neq(field, value) => record.field_value(field).as_deref() != Some(value.as_str()),
        Filter::Gt(field, value) => compare(record, field, value, std::cmp::Ordering::is_gt),
        Filter::Gte(field, value) => compare(record, field, value, std::cmp::Ordering::is_ge),
        Filter::Lt(field, value) => compare(record, field, value, std::cmp::Ordering::is_lt),
        Filter::Lte(field, value) => compare(record, field, value, std::cmp::Ordering::is_le),
        Filter::Like(field, pattern) => match record.field_value(field) {
            Some(v) => like_match(&v, pattern),
            None => false,
        },
    }
}

/// SQL `LIKE` pattern matching: `%` matches any sequence, `_` matches one char.
/// Case-sensitive, anchored on both ends.
fn like_match(value: &str, pattern: &str) -> bool {
    let v: Vec<char> = value.chars().collect();
    let p: Vec<char> = pattern.chars().collect();
    let (mut vi, mut pi) = (0, 0);
    let mut backtrack: Option<(usize, usize)> = None;

    while vi < v.len() {
        match p.get(pi) {
            Some('%') => {
                pi += 1;
                backtrack = Some((pi, vi));
            }
            Some('_') => {
                vi += 1;
                pi += 1;
            }
            Some(c) if *c == v[vi] => {
                vi += 1;
                pi += 1;
            }
            _ => match backtrack.as_mut() {
                Some((bp, bv)) => {
                    *bv += 1;
                    pi = *bp;
                    vi = *bv;
                }
                None => return false,
            },
        }
    }
    p[pi..].iter().all(|&c| c == '%')
}

/// Compares a record's field value against a filter value.
///
/// Tries numeric comparison first (`f64`). If either side fails to parse,
/// falls back to lexicographic string comparison. Returns `false` if the
/// field has no value.
fn compare(
    record: &impl Filterable,
    field: &str,
    filter_value: &str,
    predicate: fn(std::cmp::Ordering) -> bool,
) -> bool {
    let Some(record_value) = record.field_value(field) else {
        return false;
    };

    let ordering = match (record_value.parse::<f64>(), filter_value.parse::<f64>()) {
        (Ok(lhs), Ok(rhs)) => lhs.partial_cmp(&rhs).unwrap_or(std::cmp::Ordering::Equal),
        _ => record_value.as_str().cmp(filter_value),
    };

    predicate(ordering)
}
