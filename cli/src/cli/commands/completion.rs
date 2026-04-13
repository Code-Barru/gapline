//! `headway completion <shell> [--install]` — shell completion script
//! generation and installation.

use std::process;

use clap_complete::Shell;

use super::super::completion_install::{InstallReport, install_completion};

/// Writes the clap-generated completion script for `shell` into `writer`.
/// Exposed for integration tests so they can capture output without a
/// subprocess.
pub fn generate_completion(shell: Shell, writer: &mut dyn std::io::Write) {
    use clap::CommandFactory;
    let mut cmd = super::super::parser::Cli::command();
    clap_complete::generate(shell, &mut cmd, "headway", writer);
}

/// Dispatch handler for `headway completion <shell> [--install]`.
///
/// Without `--install`, prints the script to stdout. With `--install`,
/// writes it to the shell's standard directory (oh-my-zsh aware for zsh)
/// and reports where it landed on stderr plus any manual follow-up step.
pub fn run_completion(shell: Shell, install: bool) {
    if install {
        match install_completion(shell) {
            Ok(InstallReport { path, hint }) => {
                eprintln!("Installed headway {shell} completion to {}", path.display());
                if let Some(h) = hint {
                    eprintln!("{h}");
                }
            }
            Err(e) => {
                eprintln!("Failed to install completion: {e}");
                process::exit(2);
            }
        }
    } else {
        generate_completion(shell, &mut std::io::stdout().lock());
    }
}
