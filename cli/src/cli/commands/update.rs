//! `headway update` — modify records matching a `--where` filter, with
//! optional cascade to PK-dependent files.

use std::path::Path;
use std::process;
use std::sync::Arc;

use headway_core::config::Config;
use headway_core::parser::FeedLoader;

use super::super::parser::CrudTarget;
use super::{resolve_feed, resolve_output};

#[allow(clippy::too_many_arguments)] // CRUD update needs all of these from the CLI
pub fn run_update(
    config: &Arc<Config>,
    feed: Option<&Path>,
    where_query: &str,
    set: &[String],
    target: CrudTarget,
    confirm: bool,
    cascade: bool,
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

    let query = match headway_core::crud::query::parse(where_query) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("Invalid query: {e}");
            process::exit(1);
        }
    };

    let needs_dependents =
        cascade || headway_core::crud::update::has_pk_assignments(target.to_target(), set);
    let files: std::collections::HashSet<_> =
        headway_core::crud::update::required_files(target.to_target(), needs_dependents)
            .into_iter()
            .collect();
    let (mut feed_data, _parse_errors) = FeedLoader::load_only(&source, &files);

    let plan = match headway_core::crud::update::validate_update(
        &feed_data,
        target.to_target(),
        &query,
        set,
        cascade,
    ) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };

    if plan.matched_count == 0 {
        eprintln!("0 records matched filter. Nothing to update.");
        process::exit(0);
    }

    if !confirm {
        eprint!(
            "Update {} record{} in {}",
            plan.matched_count,
            if plan.matched_count > 1 { "s" } else { "" },
            plan.file_name
        );
        if let Some(ref cascade_plan) = plan.cascade {
            eprintln!(" and cascade to:");
            for entry in &cascade_plan.entries {
                eprintln!(
                    "  - {} record{} in {} ({})",
                    entry.count,
                    if entry.count > 1 { "s" } else { "" },
                    entry.dependent.file_name(),
                    entry.fk_fields.join(", ")
                );
            }
            eprint!("Proceed? [y/N] ");
        } else {
            eprint!("? [y/N] ");
        }
        let mut answer = String::new();
        if std::io::stdin().read_line(&mut answer).is_err()
            || !answer.trim().eq_ignore_ascii_case("y")
        {
            eprintln!("Aborted.");
            process::exit(0);
        }
    }

    let result = headway_core::crud::update::apply_update(&mut feed_data, &plan);

    let write_path = output.unwrap_or_else(|| feed.clone());
    if let Err(e) = headway_core::writer::write_modified_targets(
        &feed_data,
        &source,
        &result.modified_targets,
        &write_path,
    ) {
        eprintln!("{e}");
        process::exit(1);
    }

    eprintln!(
        "Updated {} record{} in {}",
        result.count,
        if result.count > 1 { "s" } else { "" },
        target.to_target().file_name()
    );
    if let Some(ref cascade_plan) = plan.cascade {
        for entry in &cascade_plan.entries {
            eprintln!(
                "Cascaded to {} record{} in {}",
                entry.count,
                if entry.count > 1 { "s" } else { "" },
                entry.dependent.file_name()
            );
        }
    }
}
