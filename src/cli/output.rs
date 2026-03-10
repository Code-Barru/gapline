use colored::{Color, Colorize};
use serde_json::{json, to_string_pretty};
use std::io::{self, BufWriter, IsTerminal, Write};
use std::path::Path;

use crate::{cli::OutputFormat, validation::ValidationReport};

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
/// This function formats a [`ValidationReport`] for display or export. It supports
/// multiple output formats (text, JSON) and can write to stdout or a file.
///
/// # Text Format
///
/// - Groups errors alphabetically by file name for easier navigation
/// - Displays colored output when writing to a TTY (terminal)
/// - Shows severity labels (`[ERROR]`, `[WARNING]`, `[INFO]`) with appropriate colors
/// - Includes file location (`file.txt:42`) and field context when available
/// - Prints a summary with error/warning/info counts and PASS/FAIL status
///
/// # JSON Format
///
/// - Produces valid, pretty-printed JSON
/// - Contains an `errors` array with all validation findings
/// - Includes a `summary` object with counts and `passed` boolean
/// - Field values serialize as `null` when not present
///
/// # Arguments
///
/// * `report` - The validation report to render
/// * `format` - Output format (Text, Json, Csv, or Xml)
/// * `output_dest` - Optional file path. If `None`, writes to stdout
///
/// # Examples
///
/// ```no_run
/// use headway::cli::{render_report, OutputFormat};
/// use headway::validation::{ValidationReport, ValidationError, Severity};
///
/// let errors = vec![
///     ValidationError::new("e1", "1", Severity::Error)
///         .message("Invalid latitude")
///         .file("stops.txt")
///         .line(42),
/// ];
/// let report = ValidationReport::from(errors);
///
/// // Print to stdout with colors
/// render_report(&report, OutputFormat::Text, None).unwrap();
///
/// // Write JSON to file
/// let path = std::path::Path::new("/tmp/report.json");
/// render_report(&report, OutputFormat::Json, Some(path)).unwrap();
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The output file cannot be created (e.g., nonexistent directory, permission denied)
/// - Writing to the output destination fails
/// - An unsupported format (Csv or Xml) is requested
///
/// Error messages for file creation failures include the full path and underlying
/// system error for easier debugging.
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
    let mut sorted_errors = report.error_list.clone();
    sorted_errors.sort_by(|a, b| match (&a.file_name, &b.file_name) {
        (Some(file_a), Some(file_b)) => file_a.cmp(file_b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });

    for error in &sorted_errors {
        let severity_str = error.severity.to_string();
        let label = if is_tty {
            format!("[{}]", severity_str.color(error.severity.color()))
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
    } else {
        if is_tty {
            format!("Status: {}", "PASS".green().bold())
        } else {
            "Status: PASS".to_string()
        }
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
        "errors": report.error_list,
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
