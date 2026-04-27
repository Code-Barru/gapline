//! Binary entry point for gapline.
//!
//! Parses CLI arguments, hands the runtime setup off to
//! [`gapline::cli::bootstrap`], then dispatches to the appropriate handler
//! in [`gapline::cli::commands`].

use clap::Parser;

use gapline::cli::{Cli, Commands, RulesCommand, bootstrap, commands};
use gapline::http::{DownloadOptions, resolve_feed};

#[allow(clippy::too_many_lines)]
fn main() {
    let mut args = Cli::parse();
    let config = bootstrap::init(&mut args);

    match &args.command {
        Commands::Validate {
            feed,
            no_cache,
            max_size,
            format,
            output,
            ..
        } => {
            let opts = DownloadOptions {
                no_cache: *no_cache,
                max_size_bytes: *max_size,
                ..Default::default()
            };
            let (path, _temp) = resolve_feed(feed.as_ref(), &opts).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1)
            });
            commands::run_validate(&config, path.as_deref(), *format, output.as_deref());
        }
        Commands::Read {
            feed,
            no_cache,
            max_size,
            where_query,
            target,
            format,
            output,
        } => {
            let opts = DownloadOptions {
                no_cache: *no_cache,
                max_size_bytes: *max_size,
                ..Default::default()
            };
            let (path, _temp) = resolve_feed(feed.as_ref(), &opts).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1)
            });
            commands::run_read(
                &config,
                path.as_deref(),
                where_query.as_ref(),
                *target,
                *format,
                output.as_deref(),
            );
        }
        Commands::Create {
            feed,
            no_cache,
            max_size,
            set,
            target,
            confirm,
            output,
        } => {
            let opts = DownloadOptions {
                no_cache: *no_cache,
                max_size_bytes: *max_size,
                ..Default::default()
            };
            let (path, _temp) = resolve_feed(feed.as_ref(), &opts).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1)
            });
            commands::run_create(
                &config,
                path.as_deref(),
                set,
                *target,
                *confirm,
                output.as_deref(),
            );
        }
        Commands::Update {
            feed,
            no_cache,
            max_size,
            where_query,
            set,
            target,
            confirm,
            cascade,
            output,
        } => {
            let opts = DownloadOptions {
                no_cache: *no_cache,
                max_size_bytes: *max_size,
                ..Default::default()
            };
            let (path, _temp) = resolve_feed(feed.as_ref(), &opts).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1)
            });
            commands::run_update(
                &config,
                &commands::UpdateArgs {
                    feed: path.as_deref(),
                    where_query,
                    set,
                    target: *target,
                    confirm: *confirm,
                    cascade: *cascade,
                    output: output.as_deref(),
                },
            );
        }
        Commands::Delete {
            feed,
            no_cache,
            max_size,
            where_query,
            target,
            confirm,
            output,
        } => {
            let opts = DownloadOptions {
                no_cache: *no_cache,
                max_size_bytes: *max_size,
                ..Default::default()
            };
            let (path, _temp) = resolve_feed(feed.as_ref(), &opts).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1)
            });
            commands::run_delete(
                &config,
                path.as_deref(),
                where_query,
                *target,
                *confirm,
                output.as_deref(),
            );
        }
        Commands::Run { file } => commands::run_run(&config, file),
        Commands::Rules { command } => match command {
            RulesCommand::List {
                severity,
                format,
                output,
            } => commands::run_rules_list(&config, *severity, *format, output.as_deref()),
        },
        Commands::Completion { shell, install } => commands::run_completion(*shell, *install),
    }
}
