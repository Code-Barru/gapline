//! Binary entry point for headway.
//!
//! Parses CLI arguments, hands the runtime setup off to
//! [`headway::cli::bootstrap`], then dispatches to the appropriate handler
//! in [`headway::cli::commands`].

use clap::Parser;

use headway::cli::{Cli, Commands, RulesCommand, bootstrap, commands};

fn main() {
    let mut args = Cli::parse();
    let config = bootstrap::init(&mut args);

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
            where_query,
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
        Commands::Completion { shell, install } => commands::run_completion(*shell, *install),
    }
}
