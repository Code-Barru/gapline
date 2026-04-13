//! Process initialization — everything that needs to happen between
//! `Cli::parse()` and the subcommand dispatch.
//!
//! Keeps `main` free of overrides plumbing, config loading, thread-pool
//! configuration and color-output policy.

use std::process;
use std::sync::Arc;

use headway_core::config::{CliOverrides, Config};

use super::parser::{Cli, Commands, SeverityArg};

/// Build the runtime config from the parsed CLI args, configure the global
/// thread pool and color override, and return the loaded config.
///
/// Exits with code 2 on config-loading errors.
pub fn init(args: &mut Cli) -> Arc<Config> {
    let overrides = build_overrides(args);
    let config = load_config(overrides);
    apply_runtime(&config);
    config
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
            process::exit(2);
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
        eprintln!("Warning: failed to configure thread pool: {e}");
    }

    // `colored::control::set_override` is process-global. `no_color` wins if
    // both flags somehow ended up set (clap's `conflicts_with` already
    // prevents that on the CLI side, but a config file with both true would
    // otherwise be ambiguous).
    if config.output.no_color {
        colored::control::set_override(false);
    } else if config.output.force_color {
        colored::control::set_override(true);
    }
}
