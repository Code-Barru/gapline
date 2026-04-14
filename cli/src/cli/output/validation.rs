use colored::{Color, Colorize};
use serde::Serialize;
use serde_json::{json, to_string_pretty};
use std::fmt::Write as _;
use std::io::{self, Write};
use std::path::Path;

use super::{announce_html_dest, csv_to_io, html_escape, open_html_sink, open_writer, xml_to_io};
use crate::cli::OutputFormat;
use headway_core::config::Config;
use headway_core::validation::{Severity, ValidationError, ValidationReport};

const VALIDATION_HTML_TEMPLATE: &str = include_str!("validation_template.html");

struct FilteredView<'a> {
    errors: Vec<&'a ValidationError>,
    error_count: usize,
    warning_count: usize,
    info_count: usize,
}

impl<'a> FilteredView<'a> {
    fn new(report: &'a ValidationReport, min: Option<Severity>) -> Self {
        let keep = |e: &&ValidationError| min.is_none_or(|m| e.severity >= m);
        let errors: Vec<&ValidationError> = report
            .errors_sorted_by_file()
            .into_iter()
            .filter(keep)
            .collect();

        let (mut error_count, mut warning_count, mut info_count) = (0, 0, 0);
        for e in &errors {
            match e.severity {
                Severity::Error => error_count += 1,
                Severity::Warning => warning_count += 1,
                Severity::Info => info_count += 1,
            }
        }

        Self {
            errors,
            error_count,
            warning_count,
            info_count,
        }
    }

    fn has_errors(&self) -> bool {
        self.error_count > 0
    }
}

fn severity_color(severity: Severity) -> Color {
    match severity {
        Severity::Error => Color::Red,
        Severity::Warning => Color::Yellow,
        Severity::Info => Color::Cyan,
    }
}

/// Renders a validation report in the specified output format.
///
/// Findings are filtered by `config.validation.min_severity`; summary counts
/// reflect the filtered view so the user never sees "1 error / 0 warnings"
/// while 50 hidden warnings still exist.
///
/// # Errors
///
/// Returns an error if writing to the output destination fails.
pub fn render_report(
    report: &ValidationReport,
    format: OutputFormat,
    feed_path: &Path,
    output_dest: Option<&Path>,
    config: &Config,
) -> io::Result<()> {
    let view = FilteredView::new(report, config.validation.min_severity);
    let use_color = super::should_use_color(&config.output, output_dest);
    match format {
        OutputFormat::Text => render_text(&view, output_dest, use_color),
        OutputFormat::Json => render_json(&view, output_dest),
        OutputFormat::Csv => render_csv(&view, output_dest),
        OutputFormat::Xml => render_xml(&view, output_dest),
        OutputFormat::Html => render_html(&view, feed_path, output_dest),
    }
}

fn render_text(
    view: &FilteredView<'_>,
    output_dest: Option<&Path>,
    use_color: bool,
) -> io::Result<()> {
    let is_tty = use_color;
    let mut writer = open_writer(output_dest)?;

    for error in &view.errors {
        let severity_str = error.severity.to_string();
        let label = if is_tty {
            format!("[{}]", severity_str.color(severity_color(error.severity)))
        } else {
            format!("[{severity_str}]")
        };

        write!(writer, "{label:10} {} — {}", error.rule_id, error.message)?;

        if let (Some(file_name), Some(line_number)) = (&error.file_name, error.line_number) {
            write!(writer, " — {file_name}:{line_number}")?;
        }

        if let (Some(field), Some(value)) = (&error.field_name, &error.value) {
            write!(writer, " — {field} = {value}")?;
        }

        writeln!(writer)?;
    }

    writeln!(writer)?;
    writeln!(writer)?;
    writeln!(writer, "{}", "=".repeat(35))?;

    let title = if is_tty {
        "Summary".bold().to_string()
    } else {
        "Summary".to_string()
    };
    writeln!(writer, "{title}")?;
    writeln!(writer, "{}", "=".repeat(35))?;

    let error_text = format_count(view.error_count, "Error", "Errors", Color::Red, is_tty);
    let warning_text = format_count(
        view.warning_count,
        "Warning",
        "Warnings",
        Color::Yellow,
        is_tty,
    );
    let info_text = format_count(view.info_count, "Info", "Infos", Color::Cyan, is_tty);
    writeln!(writer, "{error_text} — {warning_text} — {info_text}")?;

    let status = if view.has_errors() {
        if is_tty {
            format!("Status: {}", "FAIL".red().bold())
        } else {
            "Status: FAIL".to_string()
        }
    } else if is_tty {
        format!("Status: {}", "PASS".green().bold())
    } else {
        "Status: PASS".to_string()
    };
    writeln!(writer, "{status}")?;

    writer.flush()
}

fn format_count(count: usize, singular: &str, plural: &str, color: Color, is_tty: bool) -> String {
    let label = if count == 1 { singular } else { plural };
    if is_tty {
        format!("{} {label}", count.to_string().color(color))
    } else {
        format!("{count} {label}")
    }
}

fn render_json(view: &FilteredView<'_>, output_dest: Option<&Path>) -> io::Result<()> {
    let json = json!({
        "errors": view.errors,
        "summary": {
            "error_count": view.error_count,
            "info_count": view.info_count,
            "passed": !view.has_errors(),
            "warning_count": view.warning_count,
        }
    });

    let mut writer = open_writer(output_dest)?;
    writeln!(writer, "{}", to_string_pretty(&json)?)?;
    writer.flush()
}

const CSV_HEADERS: &[&str] = &[
    "rule_id",
    "section",
    "severity",
    "message",
    "file_name",
    "line_number",
    "field_name",
    "value",
];

fn render_csv(view: &FilteredView<'_>, output_dest: Option<&Path>) -> io::Result<()> {
    let writer = open_writer(output_dest)?;
    let mut csv_w = csv::Writer::from_writer(writer);
    if view.errors.is_empty() {
        csv_w.write_record(CSV_HEADERS).map_err(csv_to_io)?;
    } else {
        for err in &view.errors {
            csv_w.serialize(*err).map_err(csv_to_io)?;
        }
    }
    csv_w.flush()?;
    Ok(())
}

#[derive(Serialize)]
struct XmlView<'a> {
    summary: XmlSummary,
    #[serde(rename = "error")]
    errors: Vec<&'a ValidationError>,
}

#[derive(Serialize)]
struct XmlSummary {
    error_count: usize,
    warning_count: usize,
    info_count: usize,
    passed: bool,
}

fn render_xml(view: &FilteredView<'_>, output_dest: Option<&Path>) -> io::Result<()> {
    let xml_view = XmlView {
        summary: XmlSummary {
            error_count: view.error_count,
            warning_count: view.warning_count,
            info_count: view.info_count,
            passed: !view.has_errors(),
        },
        errors: view.errors.clone(),
    };

    let mut body = String::new();
    let mut ser = quick_xml::se::Serializer::with_root(&mut body, Some("validation_report"))
        .map_err(xml_to_io)?;
    ser.indent(' ', 2);
    xml_view.serialize(ser).map_err(xml_to_io)?;

    let mut writer = open_writer(output_dest)?;
    writeln!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(writer, "{body}")?;
    writer.flush()
}

fn severity_class(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "row-error",
        Severity::Warning => "row-warning",
        Severity::Info => "row-info",
    }
}

fn build_html_body(view: &FilteredView<'_>) -> String {
    if view.errors.is_empty() {
        return r#"<section class="empty-state"><div class="icon">&#10003;</div><p><strong>No issues found.</strong></p><p>The feed passed every enabled validation rule.</p></section>"#
            .to_string();
    }

    let mut out = String::new();
    let mut idx = 0;
    while idx < view.errors.len() {
        let group_file = view.errors[idx].file_name.as_deref();
        let mut end = idx + 1;
        while end < view.errors.len() && view.errors[end].file_name.as_deref() == group_file {
            end += 1;
        }
        let group = &view.errors[idx..end];
        let file_label = group_file.unwrap_or("(no file)");
        let count = group.len();
        let name = html_escape(file_label);
        let _ = write!(
            out,
            r#"<section class="file-group" data-file-count="{count}"><h2>{name} <span class="count">({count})</span></h2><ul>"#,
        );
        for err in group {
            let sev = err.severity.to_string();
            let class = severity_class(err.severity);
            let sev_esc = html_escape(&sev);
            let rule = html_escape(&err.rule_id);
            let msg = html_escape(&err.message);
            let _ = write!(
                out,
                r#"<li class="row {class}"><span class="sev">{sev_esc}</span><div class="body"><code>{rule}</code> {msg}"#,
            );
            if let (Some(file), Some(line)) = (&err.file_name, err.line_number) {
                let file_esc = html_escape(file);
                let _ = write!(out, r#"<span class="loc">— {file_esc}:{line}</span>"#,);
            }
            if let (Some(field), Some(value)) = (&err.field_name, &err.value) {
                let field_esc = html_escape(field);
                let value_esc = html_escape(value);
                let _ = write!(
                    out,
                    r#"<span class="field">— {field_esc} = <code>{value_esc}</code></span>"#,
                );
            }
            out.push_str("</div></li>");
        }
        out.push_str("</ul></section>");
        idx = end;
    }
    out
}

fn render_html_string(view: &FilteredView<'_>, feed_path: &Path) -> String {
    let feed_name = feed_path.file_name().map_or_else(
        || feed_path.to_string_lossy().into_owned(),
        |n| n.to_string_lossy().into_owned(),
    );
    let generated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let version = env!("CARGO_PKG_VERSION");
    let (verdict_class, verdict_label) = if view.has_errors() {
        (
            "fail",
            format!(
                "FAIL — {} error{}, {} warning{}",
                view.error_count,
                if view.error_count == 1 { "" } else { "s" },
                view.warning_count,
                if view.warning_count == 1 { "" } else { "s" },
            ),
        )
    } else {
        ("pass", "PASS — No issues found".to_string())
    };
    let body = build_html_body(view);

    VALIDATION_HTML_TEMPLATE
        .replace("{{FEED_NAME}}", &html_escape(&feed_name))
        .replace("{{GENERATED_AT}}", &html_escape(&generated_at))
        .replace("{{VERSION}}", &html_escape(version))
        .replace("{{VERDICT_CLASS}}", verdict_class)
        .replace("{{VERDICT_LABEL}}", &html_escape(&verdict_label))
        .replace("{{ERROR_COUNT}}", &view.error_count.to_string())
        .replace("{{WARNING_COUNT}}", &view.warning_count.to_string())
        .replace("{{INFO_COUNT}}", &view.info_count.to_string())
        .replace("{{BODY}}", &body)
}

fn render_html(
    view: &FilteredView<'_>,
    feed_path: &Path,
    output_dest: Option<&Path>,
) -> io::Result<()> {
    let rendered = render_html_string(view, feed_path);
    let (mut writer, dest) = open_html_sink(output_dest)?;
    writer.write_all(rendered.as_bytes())?;
    writer.flush()?;
    drop(writer);
    announce_html_dest(&dest);
    Ok(())
}
