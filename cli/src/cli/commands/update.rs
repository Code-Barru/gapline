//! `headway update` — modify records matching a `--where` filter, with
//! optional cascade to PK-dependent files.

use std::path::Path;
use std::process;
use std::sync::Arc;

use headway_core::config::Config;
use headway_core::crud::update::UpdatePlan;
use headway_core::parser::FeedLoader;

use super::super::exit;
use super::super::parser::CrudTarget;
use super::{load_feed_or_exit, parse_query_or_exit, resolve_feed, resolve_output};

/// Parameters for [`run_update`]. Bundled into a struct to keep the call site
/// readable and to avoid `#[allow(clippy::too_many_arguments)]`.
pub struct UpdateArgs<'a> {
    pub feed: Option<&'a Path>,
    pub where_query: &'a str,
    pub set: &'a [String],
    pub target: CrudTarget,
    pub confirm: bool,
    pub cascade: bool,
    pub output: Option<&'a Path>,
}

fn confirm_update_plan(plan: &UpdatePlan) -> bool {
    tracing::info!(
        "Update {} record{} in {}",
        plan.matched_count,
        if plan.matched_count > 1 { "s" } else { "" },
        plan.file_name
    );
    if let Some(ref cascade_plan) = plan.cascade {
        tracing::info!("Will cascade to:");
        for entry in &cascade_plan.entries {
            tracing::info!(
                "  - {} record{} in {} ({})",
                entry.count,
                if entry.count > 1 { "s" } else { "" },
                entry.dependent.file_name(),
                entry.fk_fields.join(", ")
            );
        }
        eprint!("Proceed with cascade update? [y/N] ");
    } else {
        eprint!("Proceed? [y/N] ");
    }
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer).is_ok() && answer.trim().eq_ignore_ascii_case("y")
}

pub fn run_update(config: &Arc<Config>, args: &UpdateArgs<'_>) {
    let feed = resolve_feed(args.feed, config);
    let output = resolve_output(args.output, config);

    let source = load_feed_or_exit(&feed);
    let query = parse_query_or_exit(args.where_query);

    let target = args.target.to_target();
    let needs_dependents =
        args.cascade || headway_core::crud::update::has_pk_assignments(target, args.set);
    let files: std::collections::HashSet<_> =
        headway_core::crud::update::required_files(target, needs_dependents)
            .into_iter()
            .collect();
    let (mut feed_data, _parse_errors) = FeedLoader::load_only(&source, &files);

    let plan = match headway_core::crud::update::validate_update(
        &feed_data,
        target,
        &query,
        args.set,
        args.cascade,
    ) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("{e}");
            process::exit(exit::COMMAND_FAILED);
        }
    };

    if plan.matched_count == 0 {
        tracing::info!("0 records matched filter. Nothing to update.");
        process::exit(exit::NO_CHANGES);
    }

    if !args.confirm && !confirm_update_plan(&plan) {
        tracing::info!("Aborted.");
        process::exit(exit::SUCCESS);
    }

    let result = match headway_core::crud::update::apply_update(&mut feed_data, &plan) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("{e}");
            process::exit(exit::COMMAND_FAILED);
        }
    };

    let write_path = output.unwrap_or_else(|| feed.clone());
    if let Err(e) = headway_core::writer::write_modified_targets(
        &feed_data,
        &source,
        &result.modified_targets,
        &write_path,
    ) {
        tracing::error!("{e}");
        process::exit(exit::INPUT_ERROR);
    }

    tracing::info!(
        "Updated {} record{} in {}",
        result.count,
        if result.count > 1 { "s" } else { "" },
        target.file_name()
    );
    if let Some(ref cascade_plan) = plan.cascade {
        for entry in &cascade_plan.entries {
            tracing::info!(
                "Cascaded to {} record{} in {}",
                entry.count,
                if entry.count > 1 { "s" } else { "" },
                entry.dependent.file_name()
            );
        }
    }
}
