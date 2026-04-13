//! Output formatting for validation reports, CRUD results, and rule listings.

mod read;
mod rules;
mod validation;

use std::io::{self, BufWriter, Write};
use std::path::Path;

pub use read::render_read_results;
pub use rules::{RuleEntry, Stage, render_rules_list};
pub use validation::render_report;

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
