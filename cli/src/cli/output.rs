use colored::{Color, Colorize};
use comfy_table::{ContentArrangement, Table};
use serde::Serialize;
use serde_json::{Value, json, to_string_pretty};
use std::io::{self, BufWriter, IsTerminal, Write};
use std::path::Path;

use crate::cli::OutputFormat;
use headway_core::config::Config;
use headway_core::crud::read::ReadResult;
use headway_core::validation::{Severity, ValidationError, ValidationReport};

/// Pipeline stage at which a validation rule runs. Used by
/// `headway rules list` so users can tell apart rules that gate parsing
/// (structural) from rules that operate on the parsed feed (semantic).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    /// Pre-parsing rules (sections 1–2). Run against the raw `FeedSource`
    /// before any CSV is loaded.
    Structural,
    /// Post-parsing rules (sections 3+). Run against the loaded `GtfsFeed`.
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

/// Findings to display after applying `[validation] min_severity`.
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

        // Recompute counts on the filtered view so the summary stays
        // consistent with what is actually displayed.
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

/// Maps a [`Severity`] to its terminal display color.
///
/// This keeps the `colored` dependency in the CLI crate where it belongs,
/// rather than leaking a presentation concern into `headway-core`.
fn severity_color(severity: Severity) -> Color {
    match severity {
        Severity::Error => Color::Red,
        Severity::Warning => Color::Yellow,
        Severity::Info => Color::Cyan,
    }
}

/// Helper function to create a file with better error messages
fn create_output_file(path: &Path) -> Result<std::fs::File, std::io::Error> {
    std::fs::File::create(path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("Cannot write to {}: {}", path.display(), e),
        )
    })
}

/// Renders a validation report in the specified output format.
///
/// Findings are filtered by `config.validation.min_severity` (CA9). The
/// summary counts shown at the bottom reflect the *filtered* view, not the
/// raw report — otherwise the user would see "1 error / 0 warnings" while
/// 50 hidden warnings still existed.
///
/// # Errors
///
/// Returns an error if the output file cannot be created, writing fails,
/// or an unsupported format (Csv, Xml) is requested.
pub fn render_report(
    report: &ValidationReport,
    format: OutputFormat,
    output_dest: Option<&Path>,
    config: &Config,
) -> Result<(), std::io::Error> {
    let view = FilteredView::new(report, config.validation.min_severity);
    // Resolve color policy: explicit config wins over TTY autodetection.
    // `force_color` keeps ANSI escapes alive even when stdout is piped;
    // `no_color` strips them even on a TTY. Otherwise fall back to the
    // standard "colors only when stdout is an interactive terminal and
    // we're not writing to a file" rule.
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
        OutputFormat::Csv => {
            eprintln!("Format 'csv' not yet available. Supported formats: text, json");
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported format",
            ))
        }
        OutputFormat::Xml => {
            eprintln!("Format 'xml' not yet available. Supported formats: text, json");
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported format",
            ))
        }
    }
}

fn render_text(
    view: &FilteredView<'_>,
    output_dest: Option<&Path>,
    use_color: bool,
) -> Result<(), std::io::Error> {
    // Local rebinding so the existing `is_tty` references below keep their
    // intent — "should this output be colorized?" — without renaming every
    // call site.
    let is_tty = use_color;

    // Errors - Warnings - Infos

    let mut writer: Box<dyn Write> = if let Some(path) = output_dest {
        Box::new(BufWriter::new(create_output_file(path)?))
    } else {
        Box::new(BufWriter::new(io::stdout()))
    };

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

    // Summary

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

    writer.flush()?;
    Ok(())
}

fn format_count(count: usize, singular: &str, plural: &str, color: Color, is_tty: bool) -> String {
    let label = if count == 1 { singular } else { plural };
    if is_tty {
        format!("{} {label}", count.to_string().color(color))
    } else {
        format!("{count} {label}")
    }
}

fn render_json(view: &FilteredView<'_>, output_dest: Option<&Path>) -> Result<(), std::io::Error> {
    let json = json!({
        "errors": view.errors,
        "summary" : {
            "error_count": view.error_count,
            "info_count": view.info_count,
            "passed": !view.has_errors(),
            "warning_count": view.warning_count
        }
    });

    let mut writer: Box<dyn Write> = if let Some(path) = output_dest {
        Box::new(BufWriter::new(create_output_file(path)?))
    } else {
        Box::new(BufWriter::new(io::stdout()))
    };

    writeln!(writer, "{}", to_string_pretty(&json)?)?;

    Ok(())
}

/// Renders read results in the specified output format.
///
/// # Errors
///
/// Returns an error if writing to the output destination fails.
pub fn render_read_results(
    result: &ReadResult,
    format: OutputFormat,
    output_dest: Option<&Path>,
) -> Result<(), std::io::Error> {
    match format {
        OutputFormat::Text => render_read_text(result, output_dest),
        OutputFormat::Json => render_read_json(result, output_dest),
        OutputFormat::Csv | OutputFormat::Xml => {
            let name = match format {
                OutputFormat::Csv => "csv",
                _ => "xml",
            };
            eprintln!("Format '{name}' not yet available. Supported formats: text, json");
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported format",
            ))
        }
    }
}

fn render_read_text(result: &ReadResult, output_dest: Option<&Path>) -> Result<(), std::io::Error> {
    let mut writer: Box<dyn Write> = if let Some(path) = output_dest {
        Box::new(BufWriter::new(create_output_file(path)?))
    } else {
        Box::new(BufWriter::new(io::stdout()))
    };

    let count = result.rows.len();

    if count == 0 {
        writeln!(writer, "0 records found")?;
        return writer.flush();
    }

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(&result.headers);

    for row in &result.rows {
        let cells: Vec<&str> = row
            .iter()
            .map(|cell| cell.as_deref().unwrap_or(""))
            .collect();
        table.add_row(cells);
    }

    writeln!(writer, "{table}")?;
    writeln!(writer, "Found {count} records in {}", result.file_name)?;

    writer.flush()
}

fn render_read_json(result: &ReadResult, output_dest: Option<&Path>) -> Result<(), std::io::Error> {
    let records: Vec<Value> = result
        .rows
        .iter()
        .map(|row| {
            let obj: serde_json::Map<String, Value> = result
                .headers
                .iter()
                .zip(row.iter())
                .map(|(header, cell)| {
                    let value = match cell {
                        Some(v) => Value::String(v.clone()),
                        None => Value::Null,
                    };
                    ((*header).to_owned(), value)
                })
                .collect();
            Value::Object(obj)
        })
        .collect();

    let mut writer: Box<dyn Write> = if let Some(path) = output_dest {
        Box::new(BufWriter::new(create_output_file(path)?))
    } else {
        Box::new(BufWriter::new(io::stdout()))
    };

    writeln!(writer, "{}", to_string_pretty(&records)?)?;

    writer.flush()
}

/// Renders a list of registered validation rules in the requested format.
///
/// Used by `headway rules list`. Mirrors [`render_read_results`] in
/// shape — text uses `comfy_table`, JSON wraps the entries plus a `count`
/// field. CSV and XML are not yet supported.
///
/// # Errors
///
/// Returns an error if the output file cannot be created, writing fails,
/// or an unsupported format (Csv, Xml) is requested.
pub fn render_rules_list(
    entries: &[RuleEntry],
    format: OutputFormat,
    output_dest: Option<&Path>,
) -> Result<(), std::io::Error> {
    match format {
        OutputFormat::Text => render_rules_list_text(entries, output_dest),
        OutputFormat::Json => render_rules_list_json(entries, output_dest),
        OutputFormat::Csv => {
            eprintln!("Format 'csv' not yet available. Supported formats: text, json");
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported format",
            ))
        }
        OutputFormat::Xml => {
            eprintln!("Format 'xml' not yet available. Supported formats: text, json");
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported format",
            ))
        }
    }
}

fn render_rules_list_text(
    entries: &[RuleEntry],
    output_dest: Option<&Path>,
) -> Result<(), std::io::Error> {
    let mut writer: Box<dyn Write> = if let Some(path) = output_dest {
        Box::new(BufWriter::new(create_output_file(path)?))
    } else {
        Box::new(BufWriter::new(io::stdout()))
    };

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

fn render_rules_list_json(
    entries: &[RuleEntry],
    output_dest: Option<&Path>,
) -> Result<(), std::io::Error> {
    let json = json!({
        "rules": entries,
        "count": entries.len(),
    });

    let mut writer: Box<dyn Write> = if let Some(path) = output_dest {
        Box::new(BufWriter::new(create_output_file(path)?))
    } else {
        Box::new(BufWriter::new(io::stdout()))
    };

    writeln!(writer, "{}", to_string_pretty(&json)?)?;
    writer.flush()
}
