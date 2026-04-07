//! Command handlers — one function per subcommand.

use std::path::Path;
use std::process;
use std::sync::Arc;

use headway_core::config::Config;
use headway_core::parser::FeedLoader;

use super::output::{render_read_results, render_report};
use super::parser::{CrudTarget, OutputFormat};

pub fn run_validate(feed: &Path, format: Option<OutputFormat>, output: Option<&Path>) {
    let config = Arc::new(Config::default());
    let report = match headway_core::validation::validate(feed, config) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };

    let fmt = format.unwrap_or(OutputFormat::Text);
    if let Err(e) = render_report(&report, fmt, output) {
        eprintln!("Error while rendering report: {e}");
        process::exit(1);
    }

    if report.has_errors() {
        process::exit(1);
    }
}

pub fn run_read(
    feed: &Path,
    where_query: Option<&String>,
    target: CrudTarget,
    format: Option<OutputFormat>,
    output: Option<&Path>,
) {
    let mut source = match FeedLoader::open(feed) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };
    if let Err(e) = source.preload() {
        eprintln!("{e}");
        process::exit(1);
    }
    let (feed_data, _parse_errors) = FeedLoader::load(&source);

    let query = match where_query {
        Some(q) => match headway_core::crud::query::parse(q) {
            Ok(parsed) => Some(parsed),
            Err(e) => {
                eprintln!("Invalid query: {e}");
                process::exit(1);
            }
        },
        None => None,
    };

    let result = match headway_core::crud::read::read_records(
        &feed_data,
        target.to_target(),
        query.as_ref(),
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };

    let fmt = format.unwrap_or(OutputFormat::Text);
    if let Err(e) = render_read_results(&result, fmt, output) {
        eprintln!("Error while rendering results: {e}");
        process::exit(1);
    }
}

pub fn run_create(
    feed: &Path,
    set: &[String],
    target: CrudTarget,
    confirm: bool,
    output: Option<&Path>,
) {
    let source = match FeedLoader::open(feed) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };
    let files: std::collections::HashSet<_> =
        headway_core::crud::create::required_files(target.to_target())
            .into_iter()
            .collect();
    let (mut feed_data, _parse_errors) = FeedLoader::load_only(&source, &files);

    let plan =
        match headway_core::crud::create::validate_create(&feed_data, target.to_target(), set) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{e}");
                process::exit(1);
            }
        };

    eprintln!("Fields to create in {}:", plan.file_name);
    for (field, value) in &plan.display_fields {
        eprintln!("  {field} = {value}");
    }

    if !confirm {
        eprint!("Create 1 record in {}? [y/N] ", plan.file_name);
        let mut answer = String::new();
        if std::io::stdin().read_line(&mut answer).is_err()
            || !answer.trim().eq_ignore_ascii_case("y")
        {
            eprintln!("Aborted.");
            process::exit(0);
        }
    }

    headway_core::crud::create::apply_create(&mut feed_data, plan);

    let write_path = output.map_or_else(|| feed.to_path_buf(), Path::to_path_buf);
    if let Err(e) =
        headway_core::writer::write_modified(&feed_data, &source, target.to_target(), &write_path)
    {
        eprintln!("{e}");
        process::exit(1);
    }

    eprintln!("Created 1 record in {}", target.to_target().file_name());
}
