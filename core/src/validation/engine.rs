//! Validation engine — orchestrates rule execution against a feed source.
//!
//! The engine collects [`StructuralValidationRule`] implementations, groups them
//! by GTFS specification section, and runs them sequentially. In Epic 2 only
//! sections 1 and 2 are registered; future Epics add sections 3-8+13 by calling
//! [`register_rule`](ValidationEngine::register_rule) without modifying the
//! engine itself.

use std::collections::HashMap;
use std::io::IsTerminal;
use std::sync::Arc;

use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use rayon::prelude::*;

use crate::config::Config;
use crate::parser::FeedSource;
use crate::validation::csv_formatting::{
    CaseSensitiveRule, InvalidContentRule, InvalidDelimiterRule, InvalidEncodingRule,
    InvalidQuotingRule, MissingHeaderRule, SuperfluousWhitespaceRule,
};
use crate::validation::file_structure::{
    CsvParsingFailedRule, DuplicatedColumnRule, EmptyColumnNameRule, EmptyFileRule, EmptyRowRule,
    InvalidInputFilesInSubfolderRule, InvalidRowLengthRule, MissingCalendarFilesRule,
    MissingRecommendedFileRule, MissingRequiredFileRule, NewLineInValueRule, TooManyRowsRule,
    UnknownColumnRule, UnknownFileRule,
};
use crate::validation::{StructuralValidationRule, ValidationError, ValidationReport};

/// Section display names used in progress bars.
fn section_label(section: &str) -> &str {
    match section {
        "1" => "File Structure",
        "2" => "CSV Formatting",
        _ => "Validation",
    }
}

/// Orchestrates structural validation of a GTFS feed.
///
/// Holds an `Arc<Config>` for thread-safe sharing (preparation for parallel
/// execution in later Epics) and a vector of boxed validation rules.
///
/// # Example
///
/// ```no_run
/// use std::sync::Arc;
/// use headway_core::config::Config;
/// use headway_core::parser::FeedLoader;
/// use headway_core::validation::engine::ValidationEngine;
///
/// let config = Arc::new(Config::default());
/// let engine = ValidationEngine::new(config);
/// let source = FeedLoader::open(std::path::Path::new("feed.zip")).unwrap();
/// let report = engine.validate(&source);
/// ```
pub struct ValidationEngine {
    /// Shared configuration (thread-safe for future rayon usage).
    #[allow(dead_code)]
    config: Arc<Config>,
    /// Registered validation rules.
    rules: Vec<Box<dyn StructuralValidationRule>>,
}

impl ValidationEngine {
    /// Creates a new engine pre-loaded with all section 1 and section 2 rules.
    #[must_use]
    pub fn new(config: Arc<Config>) -> Self {
        let max_rows = config.max_rows;

        let rules: Vec<Box<dyn StructuralValidationRule>> = vec![
            Box::new(MissingRequiredFileRule),
            Box::new(MissingRecommendedFileRule),
            Box::new(MissingCalendarFilesRule),
            Box::new(EmptyFileRule),
            Box::new(EmptyColumnNameRule),
            Box::new(DuplicatedColumnRule),
            Box::new(InvalidRowLengthRule),
            Box::new(NewLineInValueRule),
            Box::new(InvalidInputFilesInSubfolderRule),
            Box::new(CsvParsingFailedRule),
            Box::new(TooManyRowsRule::new(max_rows)),
            Box::new(EmptyRowRule),
            Box::new(UnknownFileRule),
            Box::new(UnknownColumnRule),
            Box::new(InvalidEncodingRule),
            Box::new(InvalidDelimiterRule),
            Box::new(InvalidQuotingRule),
            Box::new(InvalidContentRule),
            Box::new(MissingHeaderRule),
            Box::new(SuperfluousWhitespaceRule),
            Box::new(CaseSensitiveRule),
        ];

        Self { config, rules }
    }

    /// Adds a rule dynamically, enabling extensibility without modifying the engine.
    pub fn register_rule(&mut self, rule: Box<dyn StructuralValidationRule>) {
        self.rules.push(rule);
    }

    /// Groups the registered rules by their section identifier.
    #[must_use]
    pub fn group_rules_by_section(&self) -> HashMap<String, Vec<&dyn StructuralValidationRule>> {
        let mut map: HashMap<String, Vec<&dyn StructuralValidationRule>> = HashMap::new();
        for rule in &self.rules {
            map.entry(rule.section().to_string())
                .or_default()
                .push(rule.as_ref());
        }
        map
    }

    /// Runs all registered rules against the given feed source.
    ///
    /// Rules within each section are executed **in parallel** via rayon.
    /// Sections themselves run sequentially to maintain the gate ordering.
    /// A progress bar is displayed per section when stderr is a TTY.
    ///
    /// Returns a [`ValidationReport`] aggregating all findings.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn validate(&self, source: &FeedSource) -> ValidationReport {
        let grouped = self.group_rules_by_section();

        let mut sections: Vec<&String> = grouped.keys().collect();
        sections.sort();

        let multi = MultiProgress::new();
        if !std::io::stderr().is_terminal() {
            multi.set_draw_target(ProgressDrawTarget::hidden());
        }

        let style = ProgressStyle::with_template("{msg} [{bar:30.cyan/dim}] {pos}/{len}")
            .expect("valid progress template")
            .progress_chars("█░░");

        let mut all_errors: Vec<ValidationError> = Vec::new();

        for section_key in sections {
            let rules = &grouped[section_key];
            let label = section_label(section_key);

            let pb = multi.add(ProgressBar::new(rules.len() as u64));
            pb.set_style(style.clone());
            pb.set_message(label.to_string());

            let section_errors: Vec<ValidationError> = rules
                .par_iter()
                .flat_map(|rule| {
                    let errors = rule.validate(source);
                    pb.inc(1);
                    errors
                })
                .collect();

            all_errors.extend(section_errors);
            pb.finish();
        }

        ValidationReport::from(all_errors)
    }
}
