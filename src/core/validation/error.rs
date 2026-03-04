use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub(crate) rule_id: String,
    pub(crate) section: String,
    pub(crate) severity: Severity,
    pub(crate) message: String,
    pub(crate) file_name: Option<String>,
    pub(crate) line_number: Option<usize>,
    pub(crate) field_name: Option<String>,
    pub(crate) value: Option<String>,
}

impl ValidationError {
    pub fn new(rule_id: impl Into<String>, section: impl Into<String>, severity: Severity) -> Self {
        Self {
            rule_id: rule_id.into(),
            section: section.into(),
            severity,
            message: String::new(),
            file_name: None,
            line_number: None,
            field_name: None,
            value: None,
        }
    }

    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    #[must_use]
    pub fn file(mut self, file: impl Into<String>) -> Self {
        self.file_name = Some(file.into());
        self
    }

    #[must_use]
    pub fn line(mut self, line: usize) -> Self {
        self.line_number = Some(line);
        self
    }

    #[must_use]
    pub fn field(mut self, field: impl Into<String>) -> Self {
        self.field_name = Some(field.into());
        self
    }

    #[must_use]
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl Display for Severity {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Error => write!(f, "ERROR"),
        }
    }
}
