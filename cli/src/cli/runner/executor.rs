//! Sequential executor for parsed `.hw` directives.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use headway_core::config::Config;
use headway_core::crud::read::GtfsTarget;
use headway_core::models::GtfsFeed;
use headway_core::parser::{FeedLoader, ParseError};
use headway_core::validation::ValidationReport;
use headway_core::validation::engine::ValidationEngine;

use super::error::RunError;
use super::parser::{DirectiveKind, HwDirective};
use crate::cli::output::{render_read_results, render_report};
use crate::cli::parser::OutputFormat;

struct RunnerState {
    feed: Option<GtfsFeed>,
    feed_path: Option<PathBuf>,
    modified_targets: HashSet<GtfsTarget>,
    parse_errors: Vec<ParseError>,
}

impl RunnerState {
    fn new() -> Self {
        Self {
            feed: None,
            feed_path: None,
            modified_targets: HashSet::new(),
            parse_errors: Vec::new(),
        }
    }

    fn require_feed(&self, line: usize) -> Result<&GtfsFeed, RunError> {
        self.feed.as_ref().ok_or(RunError::NoFeedLoaded { line })
    }

    fn require_feed_mut(&mut self, line: usize) -> Result<&mut GtfsFeed, RunError> {
        self.feed.as_mut().ok_or(RunError::NoFeedLoaded { line })
    }
}

/// Executes `.hw` directives sequentially, stopping on first error.
///
/// # Errors
///
/// Returns the first [`RunError`] encountered.
pub fn execute(directives: &[HwDirective]) -> Result<(), RunError> {
    let mut state = RunnerState::new();

    for (i, directive) in directives.iter().enumerate() {
        eprintln!("[{}] {}", i + 1, directive.raw_line);

        match &directive.kind {
            DirectiveKind::Feed { path } => exec_feed(&mut state, path, directive.line_number)?,
            DirectiveKind::Save { path } => {
                exec_save(&state, path.as_ref(), directive.line_number)?;
            }
            DirectiveKind::Validate { format, output } => {
                exec_validate(&state, *format, output.as_deref(), directive.line_number)?;
            }
            DirectiveKind::Read {
                target,
                where_query,
                format,
                output,
            } => {
                exec_read(
                    &state,
                    *target,
                    where_query.as_deref(),
                    *format,
                    output.as_deref(),
                    directive.line_number,
                )?;
            }
            DirectiveKind::Create {
                target,
                set,
                confirm,
            } => {
                exec_create(&mut state, *target, set, *confirm, directive.line_number)?;
            }
            DirectiveKind::Update {
                target,
                where_query,
                set,
                confirm,
                cascade,
            } => {
                exec_update(
                    &mut state,
                    *target,
                    where_query,
                    set,
                    *confirm,
                    *cascade,
                    directive.line_number,
                )?;
            }
            DirectiveKind::Delete {
                target,
                where_query,
                confirm,
            } => {
                exec_delete(
                    &mut state,
                    *target,
                    where_query.as_deref(),
                    *confirm,
                    directive.line_number,
                )?;
            }
        }
    }

    Ok(())
}

fn exec_feed(state: &mut RunnerState, path: &std::path::Path, line: usize) -> Result<(), RunError> {
    let mut source = FeedLoader::open(path).map_err(|e| RunError::FeedLoad {
        line,
        message: e.to_string(),
    })?;
    source.preload().map_err(|e| RunError::FeedLoad {
        line,
        message: e.to_string(),
    })?;
    let (feed, parse_errors) = FeedLoader::load(&source);

    state.feed = Some(feed);
    state.feed_path = Some(path.to_path_buf());
    state.modified_targets.clear();
    state.parse_errors = parse_errors;

    Ok(())
}

fn exec_save(state: &RunnerState, path: Option<&PathBuf>, line: usize) -> Result<(), RunError> {
    let feed = state.require_feed(line)?;
    let output = path.or(state.feed_path.as_ref()).ok_or(RunError::Command {
        line,
        message: "save requires a path (no feed path available as default)".to_string(),
    })?;

    let targets: Vec<GtfsTarget> = state.modified_targets.iter().copied().collect();

    let Some(feed_path) = state.feed_path.as_ref().filter(|_| !targets.is_empty()) else {
        return headway_core::writer::write_feed_atomic(feed, output)
            .map_err(|e| RunError::Write { line, source: e });
    };

    let source = FeedLoader::open(feed_path).map_err(|e| RunError::Command {
        line,
        message: e.to_string(),
    })?;

    headway_core::writer::write_modified_targets(feed, &source, &targets, output)
        .map_err(|e| RunError::Write { line, source: e })
}

fn exec_validate(
    state: &RunnerState,
    format: Option<OutputFormat>,
    output: Option<&std::path::Path>,
    line: usize,
) -> Result<(), RunError> {
    let feed = state.require_feed(line)?;

    let config = Config {
        quiet: true,
        ..Config::default()
    };
    let engine = ValidationEngine::new(Arc::new(config));

    let report: ValidationReport = engine.validate_feed(feed, &state.parse_errors);

    let fmt = format.unwrap_or(OutputFormat::Text);
    render_report(&report, fmt, output).map_err(|e| RunError::Command {
        line,
        message: format!("render error: {e}"),
    })?;

    if report.has_errors() {
        return Err(RunError::ValidationFailed { line });
    }

    Ok(())
}

fn exec_read(
    state: &RunnerState,
    target: crate::cli::parser::CrudTarget,
    where_query: Option<&str>,
    format: Option<OutputFormat>,
    output: Option<&std::path::Path>,
    line: usize,
) -> Result<(), RunError> {
    let feed = state.require_feed(line)?;

    let query = match where_query {
        Some(q) => Some(
            headway_core::crud::query::parse(q).map_err(|e| RunError::Command {
                line,
                message: format!("invalid query: {e}"),
            })?,
        ),
        None => None,
    };

    let result = headway_core::crud::read::read_records(feed, target.to_target(), query.as_ref())
        .map_err(|e| RunError::Command {
        line,
        message: e.to_string(),
    })?;

    let fmt = format.unwrap_or(OutputFormat::Text);
    render_read_results(&result, fmt, output).map_err(|e| RunError::Command {
        line,
        message: format!("render error: {e}"),
    })?;

    Ok(())
}

fn exec_create(
    state: &mut RunnerState,
    target: crate::cli::parser::CrudTarget,
    set: &[String],
    confirm: bool,
    line: usize,
) -> Result<(), RunError> {
    if !confirm {
        return Err(RunError::MissingConfirm { line });
    }

    let feed = state.require_feed(line)?;

    let plan = headway_core::crud::create::validate_create(feed, target.to_target(), set).map_err(
        |e| RunError::Command {
            line,
            message: e.to_string(),
        },
    )?;

    let feed = state.require_feed_mut(line)?;
    headway_core::crud::create::apply_create(feed, plan);
    state.modified_targets.insert(target.to_target());

    eprintln!("  Created 1 record in {}", target.to_target().file_name());
    Ok(())
}

fn exec_update(
    state: &mut RunnerState,
    target: crate::cli::parser::CrudTarget,
    where_query: &str,
    set: &[String],
    confirm: bool,
    cascade: bool,
    line: usize,
) -> Result<(), RunError> {
    if !confirm {
        return Err(RunError::MissingConfirm { line });
    }

    let feed = state.require_feed(line)?;

    let query = headway_core::crud::query::parse(where_query).map_err(|e| RunError::Command {
        line,
        message: format!("invalid query: {e}"),
    })?;

    let plan =
        headway_core::crud::update::validate_update(feed, target.to_target(), &query, set, cascade)
            .map_err(|e| RunError::Command {
                line,
                message: e.to_string(),
            })?;

    if plan.matched_count == 0 {
        eprintln!("  0 records matched. Nothing to update.");
        return Ok(());
    }

    let feed = state.require_feed_mut(line)?;
    let result = headway_core::crud::update::apply_update(feed, &plan);
    state
        .modified_targets
        .extend(result.modified_targets.iter().copied());

    eprintln!(
        "  Updated {} record{} in {}",
        result.count,
        if result.count > 1 { "s" } else { "" },
        target.to_target().file_name()
    );
    Ok(())
}

fn exec_delete(
    state: &mut RunnerState,
    target: crate::cli::parser::CrudTarget,
    where_query: Option<&str>,
    confirm: bool,
    line: usize,
) -> Result<(), RunError> {
    if !confirm {
        return Err(RunError::MissingConfirm { line });
    }

    let where_query = where_query.ok_or_else(|| RunError::Command {
        line,
        message: "delete requires --where filter".to_string(),
    })?;

    let feed = state.require_feed(line)?;

    let query = headway_core::crud::query::parse(where_query).map_err(|e| RunError::Command {
        line,
        message: format!("invalid query: {e}"),
    })?;

    let plan = headway_core::crud::delete::validate_delete(feed, target.to_target(), &query)
        .map_err(|e| RunError::Command {
            line,
            message: e.to_string(),
        })?;

    if plan.matched_count == 0 {
        eprintln!("  0 records matched. Nothing to delete.");
        return Ok(());
    }

    let feed = state.require_feed_mut(line)?;
    let result = headway_core::crud::delete::apply_delete(feed, &plan);
    state
        .modified_targets
        .extend(result.modified_targets.iter().copied());

    eprintln!(
        "  Deleted {} record{} from {}",
        result.primary_count,
        if result.primary_count > 1 { "s" } else { "" },
        target.to_target().file_name()
    );
    Ok(())
}
