//! `headway rules list` — discoverability listing of every registered
//! validation rule.

use std::path::Path;
use std::process;
use std::sync::Arc;

use headway_core::config::Config;
use headway_core::validation::engine::ValidationEngine;

use super::super::output::{RuleEntry, Stage, render_rules_list};
use super::super::parser::{OutputFormat, SeverityArg};
use super::{resolve_format, resolve_output};

/// `headway rules list` — prints every registered validation rule.
///
/// The listing always uses a fresh `Config::default()` engine so that the
/// user's `[validation] disabled_rules` / `enabled_rules` do **not** hide
/// entries — discoverability is the whole point of the command. The user
/// `config` is still consulted for `[default] format` and `[default]
/// output`, mirroring every other subcommand.
pub fn run_rules_list(
    config: &Arc<Config>,
    severity_filter: Option<SeverityArg>,
    format_cli: Option<OutputFormat>,
    output_cli: Option<&Path>,
) {
    let listing_engine = ValidationEngine::new(Arc::new(Config::default()));

    let mut entries: Vec<RuleEntry> = listing_engine
        .pre_rules()
        .iter()
        .map(|r| RuleEntry::new(r.rule_id(), r.severity(), Stage::Structural))
        .chain(
            listing_engine
                .post_rules()
                .iter()
                .map(|r| RuleEntry::new(r.rule_id(), r.severity(), Stage::Semantic)),
        )
        .collect();

    if let Some(filter) = severity_filter {
        let target = filter.to_core();
        entries.retain(|e| e.severity == target);
    }

    // Stage first (structural before semantic), then alphabetical rule_id.
    entries.sort_by(|a, b| a.stage.cmp(&b.stage).then(a.rule_id.cmp(b.rule_id)));

    let fmt = resolve_format(format_cli, config);
    let output = resolve_output(output_cli, config);

    if let Err(e) = render_rules_list(&entries, fmt, output.as_deref()) {
        eprintln!("Error rendering rules list: {e}");
        process::exit(1);
    }
}
