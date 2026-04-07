use std::fmt;

use thiserror::Error;

/// Errors that can occur when parsing or validating a query expression.
#[derive(Debug, Error)]
pub enum QueryError {
    #[error("empty field name")]
    EmptyField,

    #[error("empty value for field '{0}'")]
    EmptyValue(String),

    #[error("unexpected logical operator without expressions")]
    UnexpectedOperator,

    #[error("unknown operator in '{0}'")]
    UnknownOperator(String),

    #[error("invalid expression: '{0}'")]
    InvalidExpression(String),

    #[error("unknown field '{field}', valid fields: {valid:?}")]
    UnknownField {
        field: String,
        valid: Vec<&'static str>,
    },
}

/// A single comparison filter.
#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    /// `field = value`
    Eq(String, String),
    /// `field != value`
    Neq(String, String),
    /// `field > value`
    Gt(String, String),
    /// `field >= value`
    Gte(String, String),
    /// `field < value`
    Lt(String, String),
    /// `field <= value`
    Lte(String, String),
}

impl Filter {
    /// Returns the field name referenced by this filter.
    #[must_use]
    pub fn field(&self) -> &str {
        match self {
            Self::Eq(f, _)
            | Self::Neq(f, _)
            | Self::Gt(f, _)
            | Self::Gte(f, _)
            | Self::Lt(f, _)
            | Self::Lte(f, _) => f,
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eq(field, val) => write!(f, "{field}={val}"),
            Self::Neq(field, val) => write!(f, "{field}!={val}"),
            Self::Gt(field, val) => write!(f, "{field}>{val}"),
            Self::Gte(field, val) => write!(f, "{field}>={val}"),
            Self::Lt(field, val) => write!(f, "{field}<{val}"),
            Self::Lte(field, val) => write!(f, "{field}<={val}"),
        }
    }
}

/// A parsed query expression tree.
///
/// Supports single filters, AND (all must match), and OR (any must match).
/// AND has higher precedence than OR.
#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    /// A single filter comparison.
    Filter(Filter),
    /// All sub-queries must match (logical AND).
    And(Vec<Query>),
    /// At least one sub-query must match (logical OR).
    Or(Vec<Query>),
}
