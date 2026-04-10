//! Binary entry point for headway.
//!
//! Parses CLI arguments via [`clap`], loads the TOML configuration through
//! the standard hierarchy (defaults → global → local → CLI overrides),
//! configures the global rayon thread pool and the colored-output policy,
//! then dispatches to the appropriate handler in [`headway::cli::commands`].

use std::process;
use std::sync::Arc;

use clap::Parser;

use headway::cli::{Cli, Commands, RulesCommand, SeverityArg, commands};
use headway_core::config::{CliOverrides, Config};

#[allow(clippy::too_many_lines)] // straight-line dispatch — splitting it would only add indirection
fn main() {
    let mut args = Cli::parse();

    // 1. Build CLI overrides from the parsed args (global flags + Validate-
    //    specific severity / disabled-rule flags). Subcommand-local
    //    `feed` / `format` / `output` are passed directly to handlers
    //    instead of going through `CliOverrides` because they would prevent
    //    handlers from honoring `[default]` cleanly.
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

    // 2. Load the config (defaults → global → local → overrides). Errors
    //    here exit with code 2 to distinguish "bad config" from "validation
    //    failed" (code 1).
    let config = match Config::load(overrides) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            eprintln!("{e}");
            process::exit(2);
        }
    };

    // 3. Configure the rayon global thread pool if the user asked for a
    //    specific count. `build_global` errors if called twice — we never
    //    do, so the failure path here is benign.
    if let Some(n) = config.performance.num_threads
        && let Err(e) = rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
    {
        eprintln!("Warning: failed to configure thread pool: {e}");
    }

    // 4. Apply the colored-output override. `colored::control::set_override`
    //    is process-global; `no_color` wins if both flags somehow ended up
    //    set (clap's `conflicts_with` already prevents that on the CLI side,
    //    but a config file with both true would otherwise be ambiguous).
    if config.output.no_color {
        colored::control::set_override(false);
    } else if config.output.force_color {
        colored::control::set_override(true);
    }

    // 5. Dispatch to the subcommand handler.
    match &args.command {
        Commands::Validate {
            feed,
            format,
            output,
            ..
        } => commands::run_validate(&config, feed.as_deref(), *format, output.as_deref()),
        Commands::Read {
            feed,
            where_query,
            target,
            format,
            output,
        } => commands::run_read(
            &config,
            feed.as_deref(),
            where_query.as_ref(),
            *target,
            *format,
            output.as_deref(),
        ),
        Commands::Create {
            feed,
            set,
            target,
            confirm,
            output,
        } => commands::run_create(
            &config,
            feed.as_deref(),
            set,
            *target,
            *confirm,
            output.as_deref(),
        ),
        Commands::Update {
            feed,
            where_query,
            set,
            target,
            confirm,
            cascade,
            output,
        } => commands::run_update(
            &config,
            feed.as_deref(),
            where_query,
            set,
            *target,
            *confirm,
            *cascade,
            output.as_deref(),
        ),
        Commands::Delete {
            feed,
            where_query,
            target,
            confirm,
            output,
        } => commands::run_delete(
            &config,
            feed.as_deref(),
            where_query.as_ref(),
            *target,
            *confirm,
            output.as_deref(),
        ),
        Commands::Run { file } => commands::run_run(&config, file),
        Commands::Rules { command } => match command {
            RulesCommand::List {
                severity,
                format,
                output,
            } => commands::run_rules_list(&config, *severity, *format, output.as_deref()),
        },
    }
}
