//! Unified entry point for the gapline core API.
//!
//! [`Dataset`] is the **single entry point** for all gapline-core operations.
//! CLI, server, and Python bindings must go through `Dataset` — never
//! re-implement orchestration by calling internal modules directly.
//!
//! [`Dataset`] owns a [`crate::models::GtfsFeed`] together with its
//! [`crate::integrity::IntegrityIndex`] and exposes all core operations
//! (validation, CRUD, writing) as methods, eliminating the boilerplate that
//! every consumer previously had to repeat.
//!
//! # Workflow manuel
//!
//! ```no_run
//! use std::path::Path;
//! use gapline_core::{Dataset, config::Config, crud::{GtfsTarget, parse}};
//!
//! let (mut dataset, _parse_errors) = Dataset::from_path(Path::new("feed.zip")).unwrap();
//! let config = Config::default();
//! let report = dataset.validate(&config);
//! if !report.has_errors() {
//!     let query = parse("stop_id=S1").unwrap();
//!     dataset.delete(GtfsTarget::Stops, &query).unwrap();
//!     dataset.write_zip(Path::new("out.zip")).unwrap();
//! }
//! ```
//!
//! # One-shot
//!
//! ```no_run
//! use std::path::Path;
//! use gapline_core::{Dataset, config::Config};
//!
//! let (dataset, _) = Dataset::from_path(Path::new("feed.zip")).unwrap();
//! let report = dataset.validate(&Config::default());
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::Config;
use crate::crud::create::{CreateError, CreatePlan, apply_create, validate_create};
use crate::crud::delete::{DeleteError, DeletePlan, DeleteResult, apply_delete, validate_delete};
use crate::crud::query::Query;
use crate::crud::read::{GtfsTarget, ReadError, ReadResult, read_records};
use crate::crud::update::{UpdateError, UpdatePlan, UpdateResult, apply_update, validate_update};
use crate::integrity::IntegrityIndex;
use crate::models::GtfsFeed;
use crate::parser::{FeedLoader, FeedSource, ParseError, ParserError};
use crate::validation::{ValidationEngine, ValidationReport};
use crate::writer::{WriteError, write_modified_targets};

/// Unified view of a loaded GTFS feed: data + integrity index.
///
/// All core operations delegate to the existing modules — no business logic
/// lives here.
pub struct Dataset {
    feed: GtfsFeed,
    integrity: IntegrityIndex,
    /// Parse errors captured at load time, used by [`Dataset::validate`].
    parse_errors: Vec<ParseError>,
    /// Original path used to open this dataset, stored for [`Dataset::write_modified`].
    /// `None` when created from an already-parsed feed or in-memory source.
    source_path: Option<PathBuf>,
}

// CA2 — static Send + Sync assertion
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Dataset>();
};

impl Dataset {
    // ── Constructors ────────────────────────────────────────────────────────

    /// Creates a dataset from an already-parsed feed.
    /// No parse errors are recorded; [`validate`](Dataset::validate) runs semantic-only.
    #[must_use]
    pub fn from_feed(feed: GtfsFeed) -> Self {
        let integrity = IntegrityIndex::build_from_feed(&feed);
        Self {
            feed,
            integrity,
            parse_errors: vec![],
            source_path: None,
        }
    }

    /// Loads a feed from an already-opened [`FeedSource`].
    ///
    /// Returns the dataset and the parse errors encountered during loading.
    /// The same errors are stored inside for use by [`validate`](Dataset::validate).
    #[must_use]
    pub fn from_source(source: &FeedSource) -> (Self, Vec<ParseError>) {
        let (feed, parse_errors) = FeedLoader::load(source);
        let integrity = IntegrityIndex::build_from_feed(&feed);
        let dataset = Self {
            feed,
            integrity,
            parse_errors: parse_errors.clone(),
            source_path: None,
        };
        (dataset, parse_errors)
    }

    /// Opens, preloads, and loads a feed from `path`.
    ///
    /// The path is stored internally so that [`write_modified`](Dataset::write_modified)
    /// can copy unchanged files from the original source.
    ///
    /// # Errors
    ///
    /// Returns [`ParserError`] if the feed cannot be opened or preloaded.
    pub fn from_path(path: &Path) -> Result<(Self, Vec<ParseError>), ParserError> {
        let mut source = FeedLoader::open(path)?;
        source.preload()?;
        let (feed, parse_errors) = FeedLoader::load(&source);
        let integrity = IntegrityIndex::build_from_feed(&feed);
        let dataset = Self {
            feed,
            integrity,
            parse_errors: parse_errors.clone(),
            source_path: Some(path.to_path_buf()),
        };
        Ok((dataset, parse_errors))
    }

    /// Creates an empty dataset with no records and an empty integrity index.
    #[must_use]
    pub fn empty() -> Self {
        Self::from_feed(GtfsFeed::default())
    }

    // ── Accessors ───────────────────────────────────────────────────────────

    /// Returns a read-only reference to the underlying feed.
    #[must_use]
    pub fn feed(&self) -> &GtfsFeed {
        &self.feed
    }

    /// Returns a read-only reference to the integrity index.
    #[must_use]
    pub fn integrity(&self) -> &IntegrityIndex {
        &self.integrity
    }

    /// Returns the path this dataset was loaded from, if available.
    ///
    /// Present when created via [`from_path`](Dataset::from_path); `None` for
    /// in-memory or feed-constructed datasets.
    #[must_use]
    pub fn source_path(&self) -> Option<&Path> {
        self.source_path.as_deref()
    }

    // ── Validation ──────────────────────────────────────────────────────────

    /// Runs semantic validation on the dataset (parse errors from load included).
    ///
    /// For structural validation (requires the source), use the associated
    /// function [`Dataset::validate_structural`].
    #[must_use]
    pub fn validate(&self, config: &Config) -> ValidationReport {
        let engine = ValidationEngine::new(Arc::new(config.clone()));
        engine.validate_feed(&self.feed, &self.parse_errors)
    }

    /// Runs structural validation on a raw [`FeedSource`] (pre-load checks).
    ///
    /// This is a free associated function because structural rules operate on
    /// the source, not on a loaded `Dataset`.
    #[must_use]
    pub fn validate_structural(source: &FeedSource, config: &Config) -> ValidationReport {
        let engine = ValidationEngine::new(Arc::new(config.clone()));
        engine.validate_structural(source)
    }

    /// Runs semantic validation with explicitly supplied parse errors.
    ///
    /// Useful when you want to override or extend the errors recorded at load time.
    #[must_use]
    pub fn validate_semantic(
        &self,
        config: &Config,
        parse_errors: &[ParseError],
    ) -> ValidationReport {
        let engine = ValidationEngine::new(Arc::new(config.clone()));
        engine.validate_feed(&self.feed, parse_errors)
    }

    /// Alias for [`validate_semantic`](Dataset::validate_semantic).
    #[must_use]
    pub fn validate_with_parse_errors(
        &self,
        config: &Config,
        parse_errors: &[ParseError],
    ) -> ValidationReport {
        self.validate_semantic(config, parse_errors)
    }

    // ── Read ────────────────────────────────────────────────────────────────

    /// Reads records from `target`, optionally filtered by `query`.
    ///
    /// # Errors
    ///
    /// Returns [`ReadError`] if the query is invalid.
    pub fn read(&self, target: GtfsTarget, query: Option<&Query>) -> Result<ReadResult, ReadError> {
        read_records(&self.feed, target, query)
    }

    // ── Update ──────────────────────────────────────────────────────────────

    /// Builds an update plan without applying it (dry-run / preview).
    ///
    /// # Errors
    ///
    /// Returns [`UpdateError`] if assignments or the query are invalid.
    pub fn plan_update(
        &self,
        target: GtfsTarget,
        query: &Query,
        assignments: &[String],
        cascade: bool,
    ) -> Result<UpdatePlan, UpdateError> {
        validate_update(&self.feed, target, query, assignments, cascade)
    }

    /// Applies a previously built [`UpdatePlan`] and rebuilds the integrity index.
    ///
    /// # Errors
    ///
    /// Returns [`UpdateError`] if application fails.
    pub fn apply_update(&mut self, plan: &UpdatePlan) -> Result<UpdateResult, UpdateError> {
        let result = apply_update(&mut self.feed, plan)?;
        self.rebuild_integrity();
        Ok(result)
    }

    /// Plans and applies an update in one step.
    ///
    /// # Errors
    ///
    /// Returns [`UpdateError`] on validation or application failure.
    pub fn update(
        &mut self,
        target: GtfsTarget,
        query: &Query,
        assignments: &[String],
        cascade: bool,
    ) -> Result<UpdateResult, UpdateError> {
        let plan = self.plan_update(target, query, assignments, cascade)?;
        self.apply_update(&plan)
    }

    // ── Delete ──────────────────────────────────────────────────────────────

    /// Builds a delete plan without applying it.
    ///
    /// # Errors
    ///
    /// Returns [`DeleteError`] if the query is invalid.
    pub fn plan_delete(
        &self,
        target: GtfsTarget,
        query: &Query,
    ) -> Result<DeletePlan, DeleteError> {
        validate_delete(&self.feed, target, query)
    }

    /// Applies a previously built [`DeletePlan`] and rebuilds the integrity index.
    pub fn apply_delete(&mut self, plan: &DeletePlan) -> DeleteResult {
        let result = apply_delete(&mut self.feed, plan);
        self.rebuild_integrity();
        result
    }

    /// Plans and applies a delete in one step.
    ///
    /// # Errors
    ///
    /// Returns [`DeleteError`] if the query is invalid.
    pub fn delete(
        &mut self,
        target: GtfsTarget,
        query: &Query,
    ) -> Result<DeleteResult, DeleteError> {
        let plan = self.plan_delete(target, query)?;
        Ok(self.apply_delete(&plan))
    }

    // ── Create ──────────────────────────────────────────────────────────────

    /// Builds a create plan without applying it.
    ///
    /// # Errors
    ///
    /// Returns [`CreateError`] if assignments are invalid.
    pub fn plan_create(
        &self,
        target: GtfsTarget,
        assignments: &[String],
    ) -> Result<CreatePlan, CreateError> {
        validate_create(&self.feed, target, assignments)
    }

    /// Applies a previously built [`CreatePlan`] and rebuilds the integrity index.
    pub fn apply_create(&mut self, plan: CreatePlan) {
        apply_create(&mut self.feed, plan);
        self.rebuild_integrity();
    }

    /// Plans and applies a create in one step.
    ///
    /// # Errors
    ///
    /// Returns [`CreateError`] if assignments are invalid.
    pub fn create(
        &mut self,
        target: GtfsTarget,
        assignments: &[String],
    ) -> Result<(), CreateError> {
        let plan = self.plan_create(target, assignments)?;
        self.apply_create(plan);
        Ok(())
    }

    // ── Write ────────────────────────────────────────────────────────────────

    /// Writes the full feed as a ZIP archive at `path`.
    ///
    /// # Errors
    ///
    /// Returns [`WriteError`] on I/O or serialization failure.
    pub fn write_zip(&self, path: &Path) -> Result<(), WriteError> {
        crate::writer::write_feed(&self.feed, path)
    }

    /// Like [`write_zip`](Dataset::write_zip) but writes to a `.zip.tmp` then renames atomically.
    ///
    /// # Errors
    ///
    /// Returns [`WriteError`] on I/O or serialization failure.
    pub fn write_zip_atomic(&self, path: &Path) -> Result<(), WriteError> {
        crate::writer::write_feed_atomic(&self.feed, path)
    }

    /// Rewrites only `targets` from the in-memory feed; all other entries are
    /// copied from the original source path stored at load time.
    ///
    /// Requires the dataset to have been created via [`from_path`](Dataset::from_path).
    /// Falls back to [`write_zip_atomic`](Dataset::write_zip_atomic) when no
    /// source path is available (e.g. in-memory or test datasets).
    ///
    /// # Errors
    ///
    /// Returns [`WriteError`] on I/O, CSV, or ZIP failure, or if the original
    /// source file can no longer be opened.
    pub fn write_modified(&self, targets: &[GtfsTarget], output: &Path) -> Result<(), WriteError> {
        match self.source_path.as_ref() {
            Some(path) => {
                let source =
                    FeedLoader::open(path).map_err(|e| WriteError::Source(e.to_string()))?;
                write_modified_targets(&self.feed, &source, targets, output)
            }
            None => crate::writer::write_feed_atomic(&self.feed, output),
        }
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    fn rebuild_integrity(&mut self) {
        self.integrity = IntegrityIndex::build_from_feed(&self.feed);
    }
}
