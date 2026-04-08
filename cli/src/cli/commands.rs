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

pub fn run_update(
    feed: &Path,
    where_query: &str,
    set: &[String],
    target: CrudTarget,
    confirm: bool,
    cascade: bool,
    output: Option<&Path>,
) {
    let source = match FeedLoader::open(feed) {
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

    let write_path = output.map_or_else(|| feed.to_path_buf(), Path::to_path_buf);
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
        process::exit(0);
    }
}

pub fn run_delete(
    feed: &Path,
    where_query: Option<&String>,
    target: CrudTarget,
    confirm: bool,
    output: Option<&Path>,
) {
    let Some(where_query) = where_query else {
        eprintln!("Missing --where filter. Refusing to delete without filter.");
        process::exit(1);
    };

    let source = match FeedLoader::open(feed) {
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
                process::exit(1);
            }
        };

    if plan.matched_count == 0 {
        eprintln!("0 records matched filter. Nothing to delete.");
        process::exit(0);
    }

    if !confirm {
        confirm_delete(&plan);
    }

    let result = headway_core::crud::delete::apply_delete(&mut feed_data, &plan);

    let write_path = output.map_or_else(|| feed.to_path_buf(), Path::to_path_buf);
    if let Err(e) = headway_core::writer::write_modified_targets(
        &feed_data,
        &source,
        &result.modified_targets,
        &write_path,
    ) {
        eprintln!("{e}");
        process::exit(1);
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
    eprintln!(
        "Deleted {} ({total} record{} total)",
        parts.join(" + "),
        if total > 1 { "s" } else { "" }
    );
}
