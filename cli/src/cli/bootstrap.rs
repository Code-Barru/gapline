//! Process initialization — everything that needs to happen between
//! `Cli::parse()` and the subcommand dispatch.
//!
//! Keeps `main` free of overrides plumbing, config loading, thread-pool
//! configuration and color-output policy.

use std::process;
use std::sync::Arc;

use headway_core::config::{CliOverrides, Config, Verbosity};
use tracing_subscriber::{EnvFilter, fmt};

use super::exit;
use super::parser::{Cli, Commands, SeverityArg};

/// Build the runtime config from the parsed CLI args, configure the global
/// thread pool, color override, and logging, and return the loaded config.
///
/// Exits with code 2 on config-loading errors.
pub fn init(args: &mut Cli) -> Arc<Config> {
    let overrides = build_overrides(args);
    let config = load_config(overrides);
    init_logging(&config);
    apply_runtime(&config);
    config
}

/// Installs a `tracing` subscriber whose default level is derived from
/// `[output] verbosity` (`quiet`→warn, `normal`→info, `verbose`→debug).
/// `HEADWAY_LOG` overrides this if set — same syntax as `RUST_LOG`.
///
/// The formatter is minimal: no timestamp, level, target or span, so that
/// `tracing::info!("Updated 5 records")` prints exactly `Updated 5 records\n`
/// — identical to the previous `eprintln!` behaviour, but now gated by a
/// level filter the user can control.
fn init_logging(config: &Config) {
    let default = match config.output.verbosity {
        Verbosity::Quiet => "warn",
        Verbosity::Normal => "info",
        Verbosity::Verbose => "debug",
    };
    let filter = EnvFilter::try_from_env("HEADWAY_LOG").unwrap_or_else(|_| EnvFilter::new(default));
    // `try_init` is a no-op if a subscriber is already installed (e.g. during
    // tests that repeatedly call `bootstrap::init`).
    let _ = fmt()
        .with_writer(std::io::stderr)
        .with_level(false)
        .with_target(false)
        .without_time()
        .with_env_filter(filter)
        .try_init();
}

fn build_overrides(args: &mut Cli) -> CliOverrides {
    let mut overrides = CliOverrides {
        config_path: args.config.take(),
        no_color: args.no_color.then_some(true),
        force_color: args.force_color.then_some(true),
        threads: args.threads,
        ..CliOverrides::default()
    };
    if let Commands::Validate {
        min_severity,
        disable_rule,
        ..
    } = &args.command
    {
        overrides.min_severity = min_severity.map(SeverityArg::to_core);
        overrides.disabled_rules.clone_from(disable_rule);
    }
    overrides
}

fn load_config(overrides: CliOverrides) -> Arc<Config> {
    match Config::load(overrides) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            eprintln!("{e}");
            process::exit(exit::CONFIG_ERROR);
        }
    }
}

fn apply_runtime(config: &Config) {
    // `build_global` errors if called twice — we never do, so this is benign.
    if let Some(n) = config.performance.num_threads
        && let Err(e) = rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
    {
        tracing::warn!("failed to configure thread pool: {e}");
    }

    // Color override precedence (POSIX NO_COLOR wins over everything):
    // 1. `NO_COLOR` env var set (any value) → force off
    // 2. `[output] no_color = true` or `--no-color` → force off
    // 3. `[output] force_color = true` or `--force-color` → force on
    // 4. otherwise: fall back to `colored`'s auto-detection (TTY check)
    //
    // `colored::control::set_override` is process-global. clap's
    // `conflicts_with` prevents `--no-color` and `--force-color` being set at
    // the same time, but a config file with both true is resolved by
    // `no_color` winning.
    if std::env::var_os("NO_COLOR").is_some() || config.output.no_color {
        colored::control::set_override(false);
    } else if config.output.force_color {
        colored::control::set_override(true);
    }
}
