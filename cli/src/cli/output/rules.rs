use comfy_table::{ContentArrangement, Table};
use serde::Serialize;
use serde_json::{json, to_string_pretty};
use std::io::{self, Write};
use std::path::Path;

use super::{csv_to_io, open_writer, xml_to_io};
use crate::cli::OutputFormat;
use headway_core::validation::Severity;

/// Pipeline stage at which a validation rule runs.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Structural,
    Semantic,
}

impl Stage {
    fn as_str(self) -> &'static str {
        match self {
            Self::Structural => "structural",
            Self::Semantic => "semantic",
        }
    }
}

/// One entry in the `headway rules list` output.
#[derive(Debug, Serialize)]
pub struct RuleEntry {
    pub rule_id: &'static str,
    pub severity: Severity,
    pub stage: Stage,
}

impl RuleEntry {
    #[must_use]
    pub fn new(rule_id: &'static str, severity: Severity, stage: Stage) -> Self {
        Self {
            rule_id,
            severity,
            stage,
        }
    }
}

/// Renders a list of registered validation rules in the requested format.
///
/// # Errors
///
/// Returns an error if writing to the output destination fails.
pub fn render_rules_list(
    entries: &[RuleEntry],
    format: OutputFormat,
    output_dest: Option<&Path>,
) -> io::Result<()> {
    match format {
        OutputFormat::Text => render_text(entries, output_dest),
        OutputFormat::Json => render_json(entries, output_dest),
        OutputFormat::Csv => render_csv(entries, output_dest),
        OutputFormat::Xml => render_xml(entries, output_dest),
    }
}

fn render_text(entries: &[RuleEntry], output_dest: Option<&Path>) -> io::Result<()> {
    let mut writer = open_writer(output_dest)?;

    if entries.is_empty() {
        writeln!(writer, "0 rules")?;
        return writer.flush();
    }

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Rule ID", "Severity", "Stage"]);
    for entry in entries {
        table.add_row(vec![
            entry.rule_id.to_string(),
            entry.severity.to_string().to_lowercase(),
            entry.stage.as_str().to_string(),
        ]);
    }

    writeln!(writer, "{table}")?;
    writeln!(writer, "{} rules", entries.len())?;
    writer.flush()
}

fn render_json(entries: &[RuleEntry], output_dest: Option<&Path>) -> io::Result<()> {
    let json = json!({
        "rules": entries,
        "count": entries.len(),
    });

    let mut writer = open_writer(output_dest)?;
    writeln!(writer, "{}", to_string_pretty(&json)?)?;
    writer.flush()
}

fn render_csv(entries: &[RuleEntry], output_dest: Option<&Path>) -> io::Result<()> {
    let writer = open_writer(output_dest)?;
    let mut csv_w = csv::Writer::from_writer(writer);
    csv_w
        .write_record(["rule_id", "severity", "stage"])
        .map_err(csv_to_io)?;
    for entry in entries {
        csv_w
            .write_record([
                entry.rule_id,
                &entry.severity.to_string().to_lowercase(),
                entry.stage.as_str(),
            ])
            .map_err(csv_to_io)?;
    }
    csv_w.flush()?;
    Ok(())
}

#[derive(Serialize)]
struct XmlView<'a> {
    count: usize,
    #[serde(rename = "rule")]
    rules: &'a [RuleEntry],
}

fn render_xml(entries: &[RuleEntry], output_dest: Option<&Path>) -> io::Result<()> {
    let view = XmlView {
        count: entries.len(),
        rules: entries,
    };

    let mut body = String::new();
    let mut ser =
        quick_xml::se::Serializer::with_root(&mut body, Some("rules")).map_err(xml_to_io)?;
    ser.indent(' ', 2);
    view.serialize(ser).map_err(xml_to_io)?;

    let mut writer = open_writer(output_dest)?;
    writeln!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(writer, "{body}")?;
    writer.flush()
}
