//! `gapline delete` — remove records matching a `--where` filter, with
//! optional cascade to PK-dependent files.

use std::path::Path;
use std::process;
use std::sync::Arc;

use gapline_core::config::Config;
use gapline_core::crud::delete::DeletePlan;

use super::super::exit;
use super::super::parser::CrudTarget;
use super::{
    load_dataset_or_exit, parse_query_or_exit, resolve_feed, resolve_output, warn_parse_errors,
};

pub fn run_delete(
    config: &Arc<Config>,
    feed: Option<&Path>,
    where_query: &str,
    target: CrudTarget,
    confirm: bool,
    output: Option<&Path>,
) {
    let feed = resolve_feed(feed, config);
    let output = resolve_output(output, config);

    let (mut ds, parse_errors) = load_dataset_or_exit(&feed);
    warn_parse_errors(&parse_errors);

    let query = parse_query_or_exit(where_query);

    let plan = match ds.plan_delete(target.to_target(), &query) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("{e}");
            process::exit(exit::COMMAND_FAILED);
        }
    };

    if plan.matched_count == 0 {
        tracing::info!("0 records matched filter. Nothing to delete.");
        process::exit(exit::NO_CHANGES);
    }

    if !confirm && !confirm_delete(&plan) {
        tracing::info!("Aborted.");
        process::exit(exit::SUCCESS);
    }

    let result = ds.apply_delete(&plan);

    let write_path = output.unwrap_or_else(|| feed.clone());
    if let Err(e) = ds.write_modified(&result.modified_targets, &write_path) {
        tracing::error!("{e}");
        process::exit(exit::INPUT_ERROR);
    }

    let mut parts = vec![format!(
        "{} {}",
        result.primary_count,
        target.to_target().file_name()
    )];
    for (dep_target, count) in &result.cascade_counts {
        parts.push(format!("{count} {}", dep_target.file_name()));
    }
    let total: usize =
        result.primary_count + result.cascade_counts.iter().map(|(_, c)| c).sum::<usize>();
    tracing::info!(
        "Deleted {} ({total} record{} total)",
        parts.join(" + "),
        if total > 1 { "s" } else { "" }
    );
}

fn confirm_delete(plan: &DeletePlan) -> bool {
    tracing::info!("Records to delete from {}:", plan.file_name);
    let display_limit = 20;
    for pk in plan.matched_pks.iter().take(display_limit) {
        tracing::info!("  {pk}");
    }
    if plan.matched_count > display_limit {
        tracing::info!("  ... and {} more", plan.matched_count - display_limit);
    }

    if let Some(ref cascade) = plan.cascade {
        tracing::info!("Deleting would also delete:");
        for entry in &cascade.entries {
            tracing::info!(
                "  - {} record{} in {}",
                entry.count,
                if entry.count > 1 { "s" } else { "" },
                entry.dependent.file_name()
            );
        }
        eprint!("Proceed with cascade delete? [y/N] ");
    } else {
        eprint!(
            "Delete {} record{} from {}? [y/N] ",
            plan.matched_count,
            if plan.matched_count > 1 { "s" } else { "" },
            plan.file_name
        );
    }
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer).is_ok() && answer.trim().eq_ignore_ascii_case("y")
}
