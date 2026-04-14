//! Command handlers — one module per subcommand.
//!
//! Every public `run_*` function below is the dispatch target for a
//! `Commands` variant. They do not call each other; the only shared code is
//! the small set of `resolve_*` helpers in this file (visible to the
//! sibling modules via `pub(super)`).

use std::path::{Path, PathBuf};
use std::process;

use headway_core::config::Config;
use headway_core::crud::query::Query;
use headway_core::parser::{FeedLoader, FeedSource};

use super::exit;
use super::parser::OutputFormat;

mod completion;
mod create;
mod delete;
mod read;
mod rules;
mod run;
mod update;
mod validate;

pub use completion::{generate_completion, run_completion};
pub use create::run_create;
pub use delete::run_delete;
pub use read::run_read;
pub use rules::run_rules_list;
pub use run::run_run;
pub use update::{UpdateArgs, run_update};
pub use validate::run_validate;

/// Resolves the GTFS feed path from CLI flags then `[default] feed`.
/// Exits with an error if neither is set.
pub(super) fn resolve_feed(cli_feed: Option<&Path>, config: &Config) -> PathBuf {
    if let Some(p) = cli_feed {
        return p.to_path_buf();
    }
    if let Some(p) = config.default.feed.as_ref() {
        return p.clone();
    }
    tracing::error!(
        "Missing feed path. Pass --feed PATH or set [default] feed = \"...\" in your config."
    );
    process::exit(exit::COMMAND_FAILED);
}

/// Opens a GTFS feed for reading, exiting with `INPUT_ERROR` on failure.
pub(super) fn load_feed_or_exit(feed: &Path) -> FeedSource {
    match FeedLoader::open(feed) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("{e}");
            process::exit(exit::INPUT_ERROR);
        }
    }
}

/// Parses a `--where` filter string, exiting with `COMMAND_FAILED` on failure.
pub(super) fn parse_query_or_exit(where_query: &str) -> Query {
    match headway_core::crud::query::parse(where_query) {
        Ok(q) => q,
        Err(e) => {
            tracing::error!("Invalid query: {e}");
            process::exit(exit::COMMAND_FAILED);
        }
    }
}

/// Resolves the output format from CLI flag then `[default] format`,
/// falling back to `OutputFormat::Text`.
pub(super) fn resolve_format(cli_format: Option<OutputFormat>, config: &Config) -> OutputFormat {
    cli_format
        .or_else(|| {
            config
                .default
                .format
                .as_deref()
                .and_then(OutputFormat::from_config_str)
        })
        .unwrap_or(OutputFormat::Text)
}

/// Resolves the output destination from CLI flag then `[default] output`.
pub(super) fn resolve_output(cli_output: Option<&Path>, config: &Config) -> Option<PathBuf> {
    cli_output
        .map(Path::to_path_buf)
        .or_else(|| config.default.output.clone())
}
