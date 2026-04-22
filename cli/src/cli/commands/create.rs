//! `gapline create` — insert a new record into a GTFS file.

use std::path::Path;
use std::process;
use std::sync::Arc;

use gapline_core::config::Config;

use super::super::exit;
use super::super::parser::CrudTarget;
use super::{load_dataset_or_exit, resolve_feed, resolve_output, warn_parse_errors};

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

    let (mut ds, parse_errors) = load_dataset_or_exit(&feed);
    warn_parse_errors(&parse_errors);

    let plan = match ds.plan_create(target.to_target(), set) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("{e}");
            process::exit(exit::COMMAND_FAILED);
        }
    };

    tracing::info!("Fields to create in {}:", plan.file_name);
    for a in &plan.assignments {
        tracing::info!("  {} = {}", a.field, a.value);
    }

    if !confirm {
        eprint!("Create 1 record in {}? [y/N] ", plan.file_name);
        let mut answer = String::new();
        if std::io::stdin().read_line(&mut answer).is_err()
            || !answer.trim().eq_ignore_ascii_case("y")
        {
            tracing::info!("Aborted.");
            process::exit(exit::SUCCESS);
        }
    }

    let target_gtfs = target.to_target();
    ds.apply_create(plan);

    let write_path = output.unwrap_or_else(|| feed.clone());
    if let Err(e) = ds.write_modified(&[target_gtfs], &write_path) {
        tracing::error!("{e}");
        process::exit(exit::INPUT_ERROR);
    }

    tracing::info!("Created 1 record in {}", target_gtfs.file_name());
}
