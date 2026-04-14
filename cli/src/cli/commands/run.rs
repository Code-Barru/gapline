//! `headway run <file.hw>` — execute a batch of headway directives.

use std::path::Path;
use std::process;
use std::sync::Arc;

use headway_core::config::Config;

use super::super::exit;
use super::super::runner;

pub fn run_run(config: &Arc<Config>, file: &Path) {
    let directives = match runner::parse_hw_file(file) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            process::exit(exit::COMMAND_FAILED);
        }
    };

    if let Err(e) = runner::execute(&directives, config) {
        eprintln!("{e}");
        process::exit(exit::COMMAND_FAILED);
    }
}
