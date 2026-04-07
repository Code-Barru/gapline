use colored::{Color, Colorize};
use comfy_table::{ContentArrangement, Table};
use serde_json::{Value, json, to_string_pretty};
use std::io::{self, BufWriter, IsTerminal, Write};
use std::path::Path;

use crate::cli::OutputFormat;
use headway_core::crud::read::ReadResult;
use headway_core::validation::{Severity, ValidationReport};

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
/// # Errors
///
/// Returns an error if the output file cannot be created, writing fails,
/// or an unsupported format (Csv, Xml) is requested.
pub fn render_report(
    report: &ValidationReport,
    format: OutputFormat,
    output_dest: Option<&Path>,
) -> Result<(), std::io::Error> {
    match format {
        OutputFormat::Text => render_text(report, output_dest),
        OutputFormat::Json => render_json(report, output_dest),
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
    report: &ValidationReport,
    output_dest: Option<&Path>,
) -> Result<(), std::io::Error> {
    let is_tty = io::stdout().is_terminal() && output_dest.is_none();

    // Errors - Warnings - Infos

    let mut writer: Box<dyn Write> = if let Some(path) = output_dest {
        Box::new(BufWriter::new(create_output_file(path)?))
    } else {
        Box::new(BufWriter::new(io::stdout()))
    };

    // Sort errors by file name for grouped display
    let sorted_errors = report.errors_sorted_by_file();

    for error in &sorted_errors {
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

    let error_text = format_count(report.error_count(), "Error", "Errors", Color::Red, is_tty);
    let warning_text = format_count(
        report.warning_count(),
        "Warning",
        "Warnings",
        Color::Yellow,
        is_tty,
    );
    let info_text = format_count(report.info_count(), "Info", "Infos", Color::Cyan, is_tty);

    writeln!(writer, "{error_text} — {warning_text} — {info_text}")?;

    let status = if report.has_errors() {
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

fn render_json(
    report: &ValidationReport,
    output_dest: Option<&Path>,
) -> Result<(), std::io::Error> {
    let json = json!({
        "errors": report.errors(),
        "summary" : {
            "error_count": report.error_count(),
            "info_count": report.info_count(),
            "passed": !report.has_errors(),
            "warning_count": report.warning_count()
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
