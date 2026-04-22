//! Sequential executor for parsed `.hw` directives.

use std::path::PathBuf;
use std::sync::Arc;

use gapline_core::Dataset;
use gapline_core::batch::{BatchCommand, BatchCommandResult, BatchExecutor};
use gapline_core::config::Config;

use super::error::RunError;
use super::parser::{DirectiveKind, HwDirective};
use crate::cli::output::{render_read_results, render_report};
use crate::cli::parser::OutputFormat;

fn cmd_err(line: usize, message: String) -> RunError {
    RunError::Command { line, message }
}

/// Executes `.hw` directives sequentially, stopping on first error.
///
/// # Errors
///
/// Returns the first [`RunError`] encountered.
pub fn execute(directives: &[HwDirective], parent_config: &Arc<Config>) -> Result<(), RunError> {
    let runner_config = {
        let mut c = (**parent_config).clone();
        c.output.show_progress = false;
        Arc::new(c)
    };

    let mut executor: Option<BatchExecutor> = None;
    let mut feed_path: Option<PathBuf> = None;

    for (i, directive) in directives.iter().enumerate() {
        tracing::info!("[{}] {}", i + 1, directive.raw_line);
        exec_directive(directive, &mut executor, &mut feed_path, &runner_config)?;
    }

    Ok(())
}

fn exec_directive(
    directive: &HwDirective,
    executor: &mut Option<BatchExecutor>,
    feed_path: &mut Option<PathBuf>,
    config: &Arc<Config>,
) -> Result<(), RunError> {
    let line = directive.line_number;

    match &directive.kind {
        DirectiveKind::Feed { path } => {
            let (ds, parse_errors) = Dataset::from_path(path).map_err(|e| RunError::FeedLoad {
                line,
                message: e.to_string(),
            })?;
            *feed_path = Some(path.clone());
            *executor = Some(BatchExecutor::new(ds, parse_errors));
            Ok(())
        }

        DirectiveKind::Save { path } => {
            let exec = executor.as_mut().ok_or(RunError::NoFeedLoaded { line })?;
            let output = path.clone().or_else(|| feed_path.clone()).ok_or_else(|| {
                cmd_err(line, "save requires a path (no feed path available)".into())
            })?;
            exec.execute_one(&BatchCommand::Save {
                output: Some(output),
            })
            .map_err(|e| cmd_err(line, e.to_string()))?;
            Ok(())
        }

        DirectiveKind::Validate { format, output } => {
            let exec = executor.as_mut().ok_or(RunError::NoFeedLoaded { line })?;
            let cmd = BatchCommand::Validate {
                config: Arc::clone(config),
            };
            let result = exec
                .execute_one(&cmd)
                .map_err(|e| cmd_err(line, e.to_string()))?;
            if let BatchCommandResult::Validated(report) = result {
                let fmt = format.unwrap_or(OutputFormat::Text);
                let path = feed_path.clone().unwrap_or_else(|| PathBuf::from("feed"));
                render_report(&report, fmt, &path, output.as_deref(), config)
                    .map_err(|e| cmd_err(line, format!("render error: {e}")))?;
                if report.has_errors() {
                    return Err(RunError::ValidationFailed { line });
                }
            }
            Ok(())
        }

        DirectiveKind::Read {
            target,
            where_query,
            format,
            output,
        } => exec_read(
            executor,
            line,
            *target,
            where_query.as_deref(),
            *format,
            output.as_deref(),
        ),

        DirectiveKind::Create {
            target,
            set,
            confirm,
        } => {
            let exec = executor.as_mut().ok_or(RunError::NoFeedLoaded { line })?;
            if !confirm {
                return Err(RunError::MissingConfirm { line });
            }
            let t = target.to_target();
            exec.execute_one(&BatchCommand::Create {
                target: t,
                assignments: set.clone(),
            })
            .map_err(|e| cmd_err(line, e.to_string()))?;
            tracing::info!("  Created 1 record in {}", t.file_name());
            Ok(())
        }

        DirectiveKind::Update {
            target,
            where_query,
            set,
            confirm,
            cascade,
        } => exec_update(
            executor,
            line,
            *target,
            where_query,
            set,
            *confirm,
            *cascade,
        ),

        DirectiveKind::Delete {
            target,
            where_query,
            confirm,
        } => exec_delete(executor, line, *target, where_query.as_deref(), *confirm),
    }
}

fn exec_read(
    executor: &mut Option<BatchExecutor>,
    line: usize,
    target: crate::cli::parser::CrudTarget,
    where_query: Option<&str>,
    format: Option<OutputFormat>,
    output: Option<&std::path::Path>,
) -> Result<(), RunError> {
    let exec = executor.as_mut().ok_or(RunError::NoFeedLoaded { line })?;
    let query = where_query
        .map(gapline_core::crud::query::parse)
        .transpose()
        .map_err(|e| cmd_err(line, format!("invalid query: {e}")))?;
    let result = exec
        .execute_one(&BatchCommand::Read {
            target: target.to_target(),
            query,
        })
        .map_err(|e| cmd_err(line, e.to_string()))?;
    if let BatchCommandResult::Read(read_result) = result {
        let fmt = format.unwrap_or(OutputFormat::Text);
        render_read_results(&read_result, fmt, output)
            .map_err(|e| cmd_err(line, format!("render error: {e}")))?;
    }
    Ok(())
}

fn exec_update(
    executor: &mut Option<BatchExecutor>,
    line: usize,
    target: crate::cli::parser::CrudTarget,
    where_query: &str,
    set: &[String],
    confirm: bool,
    cascade: bool,
) -> Result<(), RunError> {
    let exec = executor.as_mut().ok_or(RunError::NoFeedLoaded { line })?;
    if !confirm {
        return Err(RunError::MissingConfirm { line });
    }
    let query = gapline_core::crud::query::parse(where_query)
        .map_err(|e| cmd_err(line, format!("invalid query: {e}")))?;
    let t = target.to_target();
    let result = exec
        .execute_one(&BatchCommand::Update {
            target: t,
            query,
            assignments: set.to_vec(),
            cascade,
        })
        .map_err(|e| cmd_err(line, e.to_string()))?;
    match result {
        BatchCommandResult::Updated(r) => {
            tracing::info!(
                "  Updated {} record{} in {}",
                r.count,
                if r.count > 1 { "s" } else { "" },
                t.file_name()
            );
        }
        BatchCommandResult::NoChanges => tracing::info!("  0 records matched. Nothing to update."),
        _ => {}
    }
    Ok(())
}

fn exec_delete(
    executor: &mut Option<BatchExecutor>,
    line: usize,
    target: crate::cli::parser::CrudTarget,
    where_query: Option<&str>,
    confirm: bool,
) -> Result<(), RunError> {
    let exec = executor.as_mut().ok_or(RunError::NoFeedLoaded { line })?;
    if !confirm {
        return Err(RunError::MissingConfirm { line });
    }
    let where_query =
        where_query.ok_or_else(|| cmd_err(line, "delete requires --where filter".into()))?;
    let query = gapline_core::crud::query::parse(where_query)
        .map_err(|e| cmd_err(line, format!("invalid query: {e}")))?;
    let t = target.to_target();
    let result = exec
        .execute_one(&BatchCommand::Delete { target: t, query })
        .map_err(|e| cmd_err(line, e.to_string()))?;
    match result {
        BatchCommandResult::Deleted(r) => {
            tracing::info!(
                "  Deleted {} record{} from {}",
                r.primary_count,
                if r.primary_count > 1 { "s" } else { "" },
                t.file_name()
            );
        }
        BatchCommandResult::NoChanges => tracing::info!("  0 records matched. Nothing to delete."),
        _ => {}
    }
    Ok(())
}
