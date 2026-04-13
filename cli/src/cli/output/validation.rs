use colored::{Color, Colorize};
use serde::Serialize;
use serde_json::{json, to_string_pretty};
use std::io::{self, IsTerminal, Write};
use std::path::Path;

use super::{csv_to_io, open_writer, xml_to_io};
use crate::cli::OutputFormat;
use headway_core::config::Config;
use headway_core::validation::{Severity, ValidationError, ValidationReport};

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
    output_dest: Option<&Path>,
    config: &Config,
) -> io::Result<()> {
    let view = FilteredView::new(report, config.validation.min_severity);
    let use_color = if config.output.force_color {
        true
    } else if config.output.no_color {
        false
    } else {
        io::stdout().is_terminal() && output_dest.is_none()
    };
    match format {
        OutputFormat::Text => render_text(&view, output_dest, use_color),
        OutputFormat::Json => render_json(&view, output_dest),
        OutputFormat::Csv => render_csv(&view, output_dest),
        OutputFormat::Xml => render_xml(&view, output_dest),
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
