//! Application configuration.
//!
//! Provides a minimal configuration struct that will be extended as features
//! are added (TOML loading, per-rule overrides, etc.).

/// Global configuration for headway.
///
/// Currently holds only the optional row-count limit used by
/// [`TooManyRowsRule`](crate::validation::file_structure::TooManyRowsRule).
/// Future tickets will add TOML deserialization and a three-tier config system.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Maximum allowed data rows per file. `None` disables the check.
    pub max_rows: Option<usize>,
    /// Suppress all progress bars and terminal output. Defaults to `false`.
    ///
    /// Set to `true` for benchmarks, tests, or non-interactive contexts.
    pub quiet: bool,
}
