//! File-based installation of shell completion scripts.
//!
//! Resolves the standard per-shell directory (with oh-my-zsh detection for
//! zsh) and writes the generated script there. Path resolution is driven
//! entirely by environment variables so it can be unit-tested by pointing
//! `$HOME` and `$ZSH_CUSTOM` at a [`tempfile::TempDir`].

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clap_complete::Shell;

use super::commands::generate_completion;

/// Result of a successful [`install_completion`] call.
pub struct InstallReport {
    /// Absolute path of the script that was written.
    pub path: PathBuf,
    /// Optional post-install instruction for the user.
    pub hint: Option<&'static str>,
}

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("$HOME is not set")]
    NoHome,
    #[error("shell {0} is not supported for --install (supported: bash, zsh, fish)")]
    Unsupported(Shell),
    #[error("failed to write {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

const PLUGIN_STUB: &str = "# gapline completion plugin - auto-generated\n";
const BASH_HINT: Option<&str> = Some("Restart your shell or source the file to activate.");
const OMZ_HINT: Option<&str> =
    Some("Add `gapline` to your plugins=(...) array in ~/.zshrc and restart your shell.");
const ZSH_NAKED_HINT: Option<&str> =
    Some("Ensure this directory is in your $fpath and run `compinit` (restart your shell).");

/// Generates the completion script for `shell` and writes it to the
/// shell-specific standard location. For zsh, detects oh-my-zsh via
/// `$ZSH_CUSTOM` / `$ZSH` and installs as a plugin.
///
/// # Errors
///
/// Returns [`InstallError::NoHome`] if `$HOME` is unset,
/// [`InstallError::Unsupported`] for shells other than bash/zsh/fish, and
/// [`InstallError::Io`] on directory creation or file write failures.
pub fn install_completion(shell: Shell) -> Result<InstallReport, InstallError> {
    let home = home_dir()?;
    match shell {
        Shell::Bash => install_at(shell, bash_path(&home), BASH_HINT),
        Shell::Fish => install_at(shell, fish_path(&home), None),
        Shell::Zsh => install_zsh(&home),
        other => Err(InstallError::Unsupported(other)),
    }
}

fn install_zsh(home: &Path) -> Result<InstallReport, InstallError> {
    if let Some(plugin_dir) = oh_my_zsh_plugin_dir() {
        write_text(&plugin_dir.join("gapline.plugin.zsh"), PLUGIN_STUB)?;
        return install_at(Shell::Zsh, plugin_dir.join("_gapline"), OMZ_HINT);
    }
    install_at(
        Shell::Zsh,
        xdg_data_home(home).join("zsh/site-functions/_gapline"),
        ZSH_NAKED_HINT,
    )
}

fn install_at(
    shell: Shell,
    path: PathBuf,
    hint: Option<&'static str>,
) -> Result<InstallReport, InstallError> {
    let mut file = create_with_parents(&path)?;
    generate_completion(shell, &mut file);
    Ok(InstallReport { path, hint })
}

fn oh_my_zsh_plugin_dir() -> Option<PathBuf> {
    if let Some(custom) = std::env::var_os("ZSH_CUSTOM") {
        return Some(PathBuf::from(custom).join("plugins/gapline"));
    }
    std::env::var_os("ZSH").map(|zsh| PathBuf::from(zsh).join("custom/plugins/gapline"))
}

fn home_dir() -> Result<PathBuf, InstallError> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or(InstallError::NoHome)
}

fn xdg_data_home(home: &Path) -> PathBuf {
    match std::env::var_os("XDG_DATA_HOME") {
        Some(v) => PathBuf::from(v),
        None => home.join(".local/share"),
    }
}

fn xdg_config_home(home: &Path) -> PathBuf {
    match std::env::var_os("XDG_CONFIG_HOME") {
        Some(v) => PathBuf::from(v),
        None => home.join(".config"),
    }
}

fn bash_path(home: &Path) -> PathBuf {
    xdg_data_home(home).join("bash-completion/completions/gapline")
}

fn fish_path(home: &Path) -> PathBuf {
    xdg_config_home(home).join("fish/completions/gapline.fish")
}

fn create_with_parents(path: &Path) -> Result<fs::File, InstallError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| io_err(parent, source))?;
    }
    fs::File::create(path).map_err(|source| io_err(path, source))
}

fn write_text(path: &Path, contents: &str) -> Result<(), InstallError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| io_err(parent, source))?;
    }
    fs::write(path, contents).map_err(|source| io_err(path, source))
}

fn io_err(path: &Path, source: io::Error) -> InstallError {
    InstallError::Io {
        path: path.to_path_buf(),
        source,
    }
}
