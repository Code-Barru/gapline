//! Output formatting for validation reports, CRUD results, and rule listings.

mod read;
mod rules;
mod validation;

use std::io::{self, BufWriter, IsTerminal, Write};
use std::path::{Path, PathBuf};

use headway_core::config::OutputSection;

pub use read::render_read_results;
pub use rules::{RuleEntry, Stage, render_rules_list};
pub use validation::render_report;

/// Single source of truth for "should this invocation emit ANSI color?".
///
/// Precedence: explicit `--force-color` / `[output] force_color = true` wins,
/// then `--no-color` / `[output] no_color = true`, otherwise auto-detect based
/// on whether stdout is a TTY *and* the output isn't being redirected to a
/// file (`-o PATH`).
///
/// The POSIX `NO_COLOR` env var is honored earlier, in `bootstrap::init`, via
/// `colored::control::set_override(false)` — that override wins over every
/// caller path, so this function doesn't need to re-check it.
pub(super) fn should_use_color(cfg: &OutputSection, output_dest: Option<&Path>) -> bool {
    if cfg.force_color {
        return true;
    }
    if cfg.no_color {
        return false;
    }
    output_dest.is_none() && io::stdout().is_terminal()
}

fn create_output_file(path: &Path) -> io::Result<std::fs::File> {
    std::fs::File::create(path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("Cannot write to {}: {}", path.display(), e),
        )
    })
}

fn open_writer(output_dest: Option<&Path>) -> io::Result<Box<dyn Write>> {
    match output_dest {
        Some(path) => Ok(Box::new(BufWriter::new(create_output_file(path)?))),
        None => Ok(Box::new(BufWriter::new(io::stdout()))),
    }
}

/// Where an HTML render is being written.
///
/// Returned alongside the writer by [`open_html_sink`] so the caller can
/// decide whether to print a "written to…" notice on stderr.
pub(super) enum HtmlDest {
    /// User passed `-o PATH`. No notice needed — they asked for this file.
    ExplicitFile,
    /// Stdout is a TTY and no `-o` was given. A tempfile was created and its
    /// path **should** be printed on stderr so the user knows where it went.
    TempFile(PathBuf),
    /// Stdout is piped/redirected. HTML goes straight to stdout.
    Stdout,
}

/// Opens the right sink for an HTML render.
///
/// - `output_dest = Some(path)` → write to `path` ([`HtmlDest::ExplicitFile`]).
/// - else if stdout is a TTY → create a persistent tempfile with `.html`
///   suffix ([`HtmlDest::TempFile`]) — dumping raw HTML on a terminal is
///   useless, so we redirect to a file the user can open.
/// - else (pipe/redirect) → stdout ([`HtmlDest::Stdout`]).
pub(super) fn open_html_sink(output_dest: Option<&Path>) -> io::Result<(Box<dyn Write>, HtmlDest)> {
    if let Some(path) = output_dest {
        let writer = Box::new(BufWriter::new(create_output_file(path)?));
        return Ok((writer, HtmlDest::ExplicitFile));
    }
    if io::stdout().is_terminal() {
        let temp = tempfile::Builder::new()
            .prefix("headway-report-")
            .suffix(".html")
            .tempfile()?;
        let (file, path) = temp
            .keep()
            .map_err(|e| io::Error::other(format!("failed to persist HTML tempfile: {e}")))?;
        let writer = Box::new(BufWriter::new(file));
        return Ok((writer, HtmlDest::TempFile(path)));
    }
    Ok((Box::new(BufWriter::new(io::stdout())), HtmlDest::Stdout))
}

/// Prints a "written to" notice on stderr if the HTML sink created a
/// tempfile. No-op for explicit files (user already knows the path) and
/// stdout (nothing to announce).
pub(super) fn announce_html_dest(dest: &HtmlDest) {
    if let HtmlDest::TempFile(path) = dest {
        eprintln!("HTML report written to: {}", path.display());
    }
}

/// HTML-escapes a string for safe interpolation into element text or
/// double-quoted attribute values. Replaces `& < > " '` with entities.
pub(super) fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

fn csv_to_io(err: csv::Error) -> io::Error {
    match err.into_kind() {
        csv::ErrorKind::Io(e) => e,
        other => io::Error::new(io::ErrorKind::InvalidData, format!("{other:?}")),
    }
}

fn xml_to_io<E: std::fmt::Display>(err: E) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("xml serialization failed: {err}"),
    )
}

fn xml_writer_to_io(err: quick_xml::Error) -> io::Error {
    match err {
        quick_xml::Error::Io(arc) => io::Error::new(arc.kind(), arc.to_string()),
        other => io::Error::new(io::ErrorKind::InvalidData, other.to_string()),
    }
}
