use comfy_table::{ContentArrangement, Table};
use serde::Serialize;
use serde_json::{json, to_string_pretty};
use std::fmt::Write as _;
use std::io::{self, Write};
use std::path::Path;

use super::{
    HtmlDest, announce_html_dest, csv_to_io, html_escape, open_html_sink, open_writer, xml_to_io,
};
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
        OutputFormat::Html => render_html(entries, output_dest),
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

const RULES_HTML_STYLE: &str = r#"<style>
*, *::before, *::after { box-sizing: border-box; }
body { margin: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #f5f5f7; color: #222; }
header { background: #1a1a2e; color: #fff; padding: 1.25rem 2rem; }
header h1 { margin: 0; font-size: 1.25rem; }
header .meta { opacity: .75; font-size: .85rem; margin-top: .25rem; }
main { max-width: 900px; margin: 0 auto; padding: 1.5rem 2rem; }
section.stage { background: #fff; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,.06); margin-bottom: 1rem; overflow: hidden; }
section.stage > h2 { margin: 0; padding: .85rem 1.25rem; font-size: 1rem; background: #eceff1; border-bottom: 1px solid #dfe3e6; }
ul { list-style: none; margin: 0; padding: 0; }
li.rule { padding: .65rem 1.25rem; border-bottom: 1px solid #f0f0f0; display: grid; grid-template-columns: auto 1fr; gap: .75rem; align-items: center; }
li.rule:last-child { border-bottom: none; }
li.rule code { background: #f5f5f5; padding: .1rem .4rem; border-radius: 3px; font-size: .9rem; }
.sev { display: inline-block; padding: .15rem .55rem; border-radius: 3px; font-size: .7rem; font-weight: 700; letter-spacing: .5px; color: #fff; min-width: 62px; text-align: center; }
.sev.error { background: #d32f2f; }
.sev.warning { background: #f57c00; }
.sev.info { background: #1976d2; }
</style>"#;

fn render_html(entries: &[RuleEntry], output_dest: Option<&Path>) -> io::Result<()> {
    let mut html = String::new();
    let total = entries.len();
    let version = env!("CARGO_PKG_VERSION");
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    html.push_str("<title>Headway Validation Rules</title>\n");
    html.push_str(RULES_HTML_STYLE);
    html.push_str("\n</head>\n<body>\n");
    let _ = writeln!(
        html,
        "<header><h1>Validation Rules</h1><div class=\"meta\">{total} rules · headway v{version}</div></header>",
    );
    html.push_str("<main>\n");

    if entries.is_empty() {
        html.push_str("<p>0 rules</p>\n");
    } else {
        // entries arrive sorted by stage then rule_id
        let mut idx = 0;
        while idx < entries.len() {
            let stage = entries[idx].stage;
            let mut end = idx + 1;
            while end < entries.len() && entries[end].stage == stage {
                end += 1;
            }
            let stage_label = html_escape(stage.as_str());
            let group_len = end - idx;
            let _ = writeln!(
                html,
                "<section class=\"stage\"><h2>{stage_label} ({group_len})</h2>\n<ul>",
            );
            for entry in &entries[idx..end] {
                let sev_lower = html_escape(&entry.severity.to_string().to_lowercase());
                let sev_upper = html_escape(&entry.severity.to_string());
                let rid = html_escape(entry.rule_id);
                let _ = writeln!(
                    html,
                    "<li class=\"rule\"><span class=\"sev {sev_lower}\">{sev_upper}</span><code>{rid}</code></li>",
                );
            }
            html.push_str("</ul></section>\n");
            idx = end;
        }
    }

    html.push_str("</main>\n</body>\n</html>\n");

    let (mut writer, dest): (_, HtmlDest) = open_html_sink(output_dest)?;
    writer.write_all(html.as_bytes())?;
    writer.flush()?;
    drop(writer);
    announce_html_dest(&dest);
    Ok(())
}
