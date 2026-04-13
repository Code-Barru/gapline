//! `headway create` — insert a new record into a GTFS file.

use std::path::Path;
use std::process;
use std::sync::Arc;

use headway_core::config::Config;
use headway_core::parser::FeedLoader;

use super::super::parser::CrudTarget;
use super::{resolve_feed, resolve_output};

pub fn run_create(
    config: &Arc<Config>,
    feed: Option<&Path>,
    set: &[String],
    target: CrudTarget,
    confirm: bool,
    output: Option<&Path>,
) {
    let feed = resolve_feed(feed, config);
    let output = resolve_output(output, config);

    let source = match FeedLoader::open(&feed) {
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

    let write_path = output.unwrap_or_else(|| feed.clone());
    if let Err(e) =
        headway_core::writer::write_modified(&feed_data, &source, target.to_target(), &write_path)
    {
        eprintln!("{e}");
        process::exit(1);
    }

    eprintln!("Created 1 record in {}", target.to_target().file_name());
}
