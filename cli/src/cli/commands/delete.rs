//! `headway delete` — remove records matching a `--where` filter, with
//! optional cascade to PK-dependent files.

use std::path::Path;
use std::process;
use std::sync::Arc;

use headway_core::config::Config;
use headway_core::parser::FeedLoader;

use super::super::exit;
use super::super::parser::CrudTarget;
use super::{resolve_feed, resolve_output};

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

    let source = match FeedLoader::open(&feed) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            process::exit(exit::INPUT_ERROR);
        }
    };

    let query = match headway_core::crud::query::parse(where_query) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("Invalid query: {e}");
            process::exit(exit::COMMAND_FAILED);
        }
    };

    let files: std::collections::HashSet<_> =
        headway_core::crud::delete::required_files(target.to_target())
            .into_iter()
            .collect();
    let (mut feed_data, _parse_errors) = FeedLoader::load_only(&source, &files);

    let plan =
        match headway_core::crud::delete::validate_delete(&feed_data, target.to_target(), &query) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{e}");
                process::exit(exit::COMMAND_FAILED);
            }
        };

    if plan.matched_count == 0 {
        tracing::info!("0 records matched filter. Nothing to delete.");
        process::exit(exit::NO_CHANGES);
    }

    if !confirm {
        confirm_delete(&plan);
    }

    let result = headway_core::crud::delete::apply_delete(&mut feed_data, &plan);

    let write_path = output.unwrap_or_else(|| feed.clone());
    if let Err(e) = headway_core::writer::write_modified_targets(
        &feed_data,
        &source,
        &result.modified_targets,
        &write_path,
    ) {
        eprintln!("{e}");
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

fn confirm_delete(plan: &headway_core::crud::delete::DeletePlan) {
    eprintln!("Records to delete from {}:", plan.file_name);
    let display_limit = 20;
    for pk in plan.matched_pks.iter().take(display_limit) {
        eprintln!("  {pk}");
    }
    if plan.matched_count > display_limit {
        eprintln!("  ... and {} more", plan.matched_count - display_limit);
    }

    if let Some(ref cascade) = plan.cascade {
        eprintln!("Deleting would also delete:");
        for entry in &cascade.entries {
            eprintln!(
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
    if std::io::stdin().read_line(&mut answer).is_err() || !answer.trim().eq_ignore_ascii_case("y")
    {
        eprintln!("Aborted.");
        process::exit(exit::SUCCESS);
    }
}
