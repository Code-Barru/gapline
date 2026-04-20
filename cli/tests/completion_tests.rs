//! Integration tests for `gapline completion <shell>` — covers script
//! generation markers (CA1/CA2/CA3/CA7/CA12) and `--install` path
//! resolution including oh-my-zsh detection.

use clap_complete::Shell;
use gapline::cli::commands::generate_completion;
use gapline::cli::install_completion;
use tempfile::TempDir;

fn generate(shell: Shell) -> String {
    let mut out = Vec::new();
    generate_completion(shell, &mut out);
    String::from_utf8(out).expect("completion script is utf-8")
}

#[test]
fn bash_script_contains_expected_markers() {
    let s = generate(Shell::Bash);
    assert!(!s.is_empty());
    assert!(s.contains("_gapline"), "bash script missing _gapline fn");
    assert!(s.contains("validate"));
    assert!(s.contains("completion"));
}

#[test]
fn zsh_script_starts_with_compdef() {
    let s = generate(Shell::Zsh);
    assert!(s.contains("#compdef gapline"));
    assert!(s.contains("validate"));
}

#[test]
fn fish_script_uses_complete_c() {
    let s = generate(Shell::Fish);
    assert!(s.contains("complete -c gapline"));
    assert!(s.contains("validate"));
}

#[test]
fn script_covers_every_subcommand() {
    let s = generate(Shell::Bash);
    for cmd in [
        "validate",
        "read",
        "create",
        "update",
        "delete",
        "run",
        "rules",
        "completion",
    ] {
        assert!(s.contains(cmd), "missing subcommand {cmd}");
    }
}

#[test]
fn script_covers_crud_targets() {
    // clap renders ValueEnum variants in kebab-case — `stop_times` becomes
    // `stop-times` in the generated completion, even though the snake_case
    // form is accepted as an alias at parse time.
    let s = generate(Shell::Bash);
    for target in [
        "trips",
        "stops",
        "stop-times",
        "calendar",
        "calendar-dates",
        "routes",
        "agency",
    ] {
        assert!(s.contains(target), "missing target {target}");
    }
}

#[test]
fn unknown_shell_fails_to_parse() {
    use clap::Parser;
    let err = gapline::cli::Cli::try_parse_from(["gapline", "completion", "bogus"]);
    assert!(err.is_err());
}

/// Single serialized test that exercises every `--install` path in
/// sequence. Mutating process-wide env vars forbids running the install
/// checks in parallel, and bundling them here is cheaper than pulling in
/// `serial_test` as a dev-dependency.
#[test]
fn install_writes_scripts_and_detects_oh_my_zsh() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    std::fs::create_dir_all(&home).unwrap();

    // SAFETY: the other tests in this binary never touch $HOME / XDG / ZSH*,
    // so overriding them here cannot race with them. Tests run on a single
    // thread only when `--test-threads=1` is passed, but the env-var churn
    // is scoped to this #[test] and restored at the end.
    let prior = EnvSnapshot::capture();
    unsafe {
        std::env::set_var("HOME", &home);
        std::env::remove_var("XDG_DATA_HOME");
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("ZSH");
        std::env::remove_var("ZSH_CUSTOM");
    }

    // bash -> XDG data home default
    let r = install_completion(Shell::Bash).unwrap();
    assert_eq!(
        r.path,
        home.join(".local/share/bash-completion/completions/gapline")
    );
    let body = std::fs::read_to_string(&r.path).unwrap();
    assert!(body.contains("_gapline"));

    // fish -> XDG config home default
    let r = install_completion(Shell::Fish).unwrap();
    assert_eq!(r.path, home.join(".config/fish/completions/gapline.fish"));
    assert!(
        std::fs::read_to_string(&r.path)
            .unwrap()
            .contains("complete -c gapline")
    );

    // zsh without oh-my-zsh -> site-functions under XDG data
    let r = install_completion(Shell::Zsh).unwrap();
    assert_eq!(
        r.path,
        home.join(".local/share/zsh/site-functions/_gapline")
    );
    assert!(
        std::fs::read_to_string(&r.path)
            .unwrap()
            .contains("#compdef gapline")
    );
    assert!(r.hint.unwrap().contains("fpath"));

    // zsh + ZSH_CUSTOM -> plugin layout with stub plugin.zsh file
    let custom = home.join(".oh-my-zsh/custom");
    unsafe {
        std::env::set_var("ZSH_CUSTOM", &custom);
    }
    let r = install_completion(Shell::Zsh).unwrap();
    assert_eq!(r.path, custom.join("plugins/gapline/_gapline"));
    assert!(custom.join("plugins/gapline/gapline.plugin.zsh").exists());
    assert!(r.hint.unwrap().contains("plugins"));

    // zsh with only ZSH set (stock oh-my-zsh install) -> $ZSH/custom/plugins
    unsafe {
        std::env::remove_var("ZSH_CUSTOM");
        std::env::set_var("ZSH", home.join(".oh-my-zsh"));
    }
    let r = install_completion(Shell::Zsh).unwrap();
    assert_eq!(
        r.path,
        home.join(".oh-my-zsh/custom/plugins/gapline/_gapline")
    );

    // elvish / powershell must be rejected
    unsafe {
        std::env::remove_var("ZSH");
    }
    assert!(install_completion(Shell::Elvish).is_err());
    assert!(install_completion(Shell::PowerShell).is_err());

    prior.restore();
}

struct EnvSnapshot {
    home: Option<std::ffi::OsString>,
    xdg_data: Option<std::ffi::OsString>,
    xdg_config: Option<std::ffi::OsString>,
    zsh: Option<std::ffi::OsString>,
    zsh_custom: Option<std::ffi::OsString>,
}

impl EnvSnapshot {
    fn capture() -> Self {
        Self {
            home: std::env::var_os("HOME"),
            xdg_data: std::env::var_os("XDG_DATA_HOME"),
            xdg_config: std::env::var_os("XDG_CONFIG_HOME"),
            zsh: std::env::var_os("ZSH"),
            zsh_custom: std::env::var_os("ZSH_CUSTOM"),
        }
    }

    fn restore(self) {
        unsafe {
            restore("HOME", self.home);
            restore("XDG_DATA_HOME", self.xdg_data);
            restore("XDG_CONFIG_HOME", self.xdg_config);
            restore("ZSH", self.zsh);
            restore("ZSH_CUSTOM", self.zsh_custom);
        }
    }
}

unsafe fn restore(key: &str, value: Option<std::ffi::OsString>) {
    unsafe {
        match value {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
}
