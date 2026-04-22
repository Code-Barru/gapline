//! Batch execution of gapline commands on a loaded [`Dataset`].
//!
//! [`BatchExecutor`] is the core entry point for running sequences of commands
//! (validate, read, create, update, delete, save) on a single feed. It tracks
//! which targets have been modified so that [`BatchCommand::Save`] can write
//! only the changed files.
//!
//! CLI, server, and Python consumers should build a `Vec<BatchCommand>` from
//! whatever input format they parse (`.hw` files, HTTP request bodies, Python
//! calls), then hand off to `BatchExecutor::run`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::Dataset;
use crate::config::Config;
use crate::crud::create::CreateError;
use crate::crud::delete::{DeleteError, DeleteResult};
use crate::crud::query::Query;
use crate::crud::read::{GtfsTarget, ReadError, ReadResult};
use crate::crud::update::{UpdateError, UpdateResult};
use crate::parser::ParseError;
use crate::validation::ValidationReport;
use crate::writer::WriteError;

/// A single operation to execute inside a batch.
#[derive(Debug)]
pub enum BatchCommand {
    Validate {
        config: Arc<Config>,
    },
    Read {
        target: GtfsTarget,
        query: Option<Query>,
    },
    Create {
        target: GtfsTarget,
        assignments: Vec<String>,
    },
    Update {
        target: GtfsTarget,
        query: Query,
        assignments: Vec<String>,
        cascade: bool,
    },
    Delete {
        target: GtfsTarget,
        /// A filter is always required for delete. Use [`Dataset::delete`] directly
        /// for bulk operations that intentionally bypass this guard.
        query: Query,
    },
    Save {
        /// Explicit output path. `None` means overwrite the source feed.
        output: Option<PathBuf>,
    },
}

/// Result produced by a single successfully executed [`BatchCommand`].
#[derive(Debug)]
pub enum BatchCommandResult {
    Validated(ValidationReport),
    Read(ReadResult),
    Created,
    Updated(UpdateResult),
    Deleted(DeleteResult),
    /// Save wrote `count` modified targets.
    Saved {
        count: usize,
    },
    /// No matching records — command was a no-op.
    NoChanges,
}

/// Errors that can occur while executing a [`BatchCommand`].
#[derive(Debug, thiserror::Error)]
pub enum BatchError {
    #[error(transparent)]
    Read(#[from] ReadError),
    #[error(transparent)]
    Create(#[from] CreateError),
    #[error(transparent)]
    Update(#[from] UpdateError),
    #[error(transparent)]
    Delete(#[from] DeleteError),
    #[error(transparent)]
    Write(#[from] WriteError),
    #[error("save requires an explicit output path (no source path available)")]
    NoSavePath,
}

/// Executes a sequence of [`BatchCommand`]s on a loaded [`Dataset`].
///
/// Tracks which GTFS files have been mutated so that [`BatchCommand::Save`]
/// can copy only the changed entries from the source archive (efficient write).
pub struct BatchExecutor {
    dataset: Dataset,
    modified_targets: HashSet<GtfsTarget>,
    parse_errors: Vec<ParseError>,
}

impl BatchExecutor {
    /// `parse_errors` come from the initial feed load and are forwarded to validate commands.
    #[must_use]
    pub fn new(dataset: Dataset, parse_errors: Vec<ParseError>) -> Self {
        Self {
            dataset,
            modified_targets: HashSet::new(),
            parse_errors,
        }
    }

    /// Runs `commands` in order, stopping on the first error.
    ///
    /// On failure returns `(failing_index, error)`; commands after the failing
    /// one are never executed.
    ///
    /// # Errors
    ///
    /// Returns `(index, BatchError)` for the first failing command.
    pub fn run(
        &mut self,
        commands: &[BatchCommand],
    ) -> Result<Vec<(usize, BatchCommandResult)>, (usize, BatchError)> {
        let mut results = Vec::with_capacity(commands.len());
        for (i, cmd) in commands.iter().enumerate() {
            let result = self.execute_one(cmd).map_err(|e| (i, e))?;
            results.push((i, result));
        }
        Ok(results)
    }

    /// # Errors
    ///
    /// Returns [`BatchError`] if the command fails.
    pub fn execute_one(&mut self, cmd: &BatchCommand) -> Result<BatchCommandResult, BatchError> {
        match cmd {
            BatchCommand::Validate { config } => {
                let report = self
                    .dataset
                    .validate_with_parse_errors(config, &self.parse_errors);
                Ok(BatchCommandResult::Validated(report))
            }

            BatchCommand::Read { target, query } => {
                let result = self.dataset.read(*target, query.as_ref())?;
                Ok(BatchCommandResult::Read(result))
            }

            BatchCommand::Create {
                target,
                assignments,
            } => {
                let plan = self.dataset.plan_create(*target, assignments)?;
                self.modified_targets.insert(*target);
                self.dataset.apply_create(plan);
                Ok(BatchCommandResult::Created)
            }

            BatchCommand::Update {
                target,
                query,
                assignments,
                cascade,
            } => {
                let plan = self
                    .dataset
                    .plan_update(*target, query, assignments, *cascade)?;
                if plan.matched_count == 0 {
                    return Ok(BatchCommandResult::NoChanges);
                }
                let result = self.dataset.apply_update(&plan)?;
                self.modified_targets
                    .extend(result.modified_targets.iter().copied());
                Ok(BatchCommandResult::Updated(result))
            }

            BatchCommand::Delete { target, query } => {
                let plan = self.dataset.plan_delete(*target, query)?;
                if plan.matched_count == 0 {
                    return Ok(BatchCommandResult::NoChanges);
                }
                let result = self.dataset.apply_delete(&plan);
                self.modified_targets
                    .extend(result.modified_targets.iter().copied());
                Ok(BatchCommandResult::Deleted(result))
            }

            BatchCommand::Save { output } => {
                let targets: Vec<GtfsTarget> = self.modified_targets.iter().copied().collect();
                let out: &Path = output
                    .as_deref()
                    .or_else(|| self.dataset.source_path())
                    .ok_or(BatchError::NoSavePath)?;

                if targets.is_empty() {
                    self.dataset.write_zip_atomic(out)?;
                } else {
                    self.dataset.write_modified(&targets, out)?;
                }

                // Clear only after a successful write so callers can retry on failure.
                self.modified_targets.clear();
                Ok(BatchCommandResult::Saved {
                    count: targets.len(),
                })
            }
        }
    }

    #[must_use]
    pub fn modified_targets(&self) -> &HashSet<GtfsTarget> {
        &self.modified_targets
    }

    #[must_use]
    pub fn dataset(&self) -> &Dataset {
        &self.dataset
    }
}
