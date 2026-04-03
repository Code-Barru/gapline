//! Validation engine — orchestrates rule execution against a feed source.
//!
//! The engine collects both structural (pre-parsing) and semantic (post-parsing)
//! validation rules, groups them by GTFS specification section, and runs them
//! sequentially. Sections 1-2 run against raw `FeedSource`; sections 3+ run
//! against the parsed `GtfsFeed`.

use std::collections::HashMap;
use std::io::IsTerminal;
use std::sync::{Arc, LazyLock};

use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use rayon::prelude::*;

static BAR_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::with_template("{msg} [{bar:30.cyan/dim}] {pos}/{len}")
        .expect("hard-coded progress template is valid")
        .progress_chars("█░░")
});

use crate::config::Config;
use crate::models::GtfsFeed;
use crate::parser::FeedSource;
use crate::parser::error::ParseError;
use crate::validation::csv_formatting::scanner;
use crate::validation::csv_formatting::{CaseSensitiveRule, MissingHeaderRule};
use crate::validation::field_type::parse_error_converter;
use crate::validation::file_structure::{
    CsvParsingFailedRule, DuplicatedColumnRule, EmptyColumnNameRule, EmptyFileRule, EmptyRowRule,
    InvalidInputFilesInSubfolderRule, InvalidRowLengthRule, MissingCalendarFilesRule,
    MissingRecommendedFileRule, MissingRequiredColumnRule, MissingRequiredFileRule,
    TooManyRowsRule, UnknownColumnRule, UnknownFileRule,
};
use crate::validation::{
    StructuralValidationRule, ValidationError, ValidationReport, ValidationRule,
};

fn section_label(section: &str) -> &str {
    match section {
        "1" => "File Structure",
        "2" => "CSV Formatting",
        "3" => "Field Type Validation",
        "4" => "Field Definition Validation",
        "5" => "Foreign Key Validation",
        "6" => "Primary Key Uniqueness",
        _ => "Validation",
    }
}

/// Orchestrates validation of a GTFS feed.
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
/// let report = engine.validate_structural(&source);
/// ```
pub struct ValidationEngine {
    #[allow(dead_code)]
    config: Arc<Config>,
    /// Pre-parsing rules (sections 1-2) operating on raw `FeedSource`.
    pre_rules: Vec<Box<dyn StructuralValidationRule>>,
    /// Post-parsing rules (sections 3+) operating on the loaded `GtfsFeed`.
    rules: Vec<Box<dyn ValidationRule>>,
}

impl ValidationEngine {
    /// Creates a new engine pre-loaded with all registered rules.
    #[must_use]
    pub fn new(config: Arc<Config>) -> Self {
        let max_rows = config.max_rows;

        // Rules that remain as individual StructuralValidationRule instances.
        // The 6 content-scanning rules (encoding, delimiter, quoting, content,
        // whitespace, new_line_in_value) are handled by the single-pass scanner
        // in validate_structural().
        let pre_rules: Vec<Box<dyn StructuralValidationRule>> = vec![
            Box::new(MissingRequiredFileRule),
            Box::new(MissingRecommendedFileRule),
            Box::new(MissingCalendarFilesRule),
            Box::new(EmptyFileRule),
            Box::new(EmptyColumnNameRule),
            Box::new(DuplicatedColumnRule),
            Box::new(InvalidRowLengthRule),
            Box::new(InvalidInputFilesInSubfolderRule),
            Box::new(CsvParsingFailedRule),
            Box::new(TooManyRowsRule::new(max_rows)),
            Box::new(EmptyRowRule),
            Box::new(UnknownFileRule),
            Box::new(UnknownColumnRule),
            Box::new(MissingRequiredColumnRule),
            Box::new(MissingHeaderRule),
            Box::new(CaseSensitiveRule),
        ];

        let mut engine = Self {
            config,
            pre_rules,
            rules: Vec::new(),
        };
        crate::validation::field_type::register_rules(&mut engine);
        crate::validation::field_definition::register_rules(&mut engine);
        crate::validation::primary_key::register_rules(&mut engine);
        crate::validation::foreign_key::register_rules(&mut engine);
        engine
    }

    /// Adds a pre-parsing (structural) rule dynamically.
    pub fn register_pre_rule(&mut self, rule: Box<dyn StructuralValidationRule>) {
        self.pre_rules.push(rule);
    }

    /// Adds a post-parsing rule that operates on the loaded `GtfsFeed`.
    pub fn register_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    /// Groups the registered pre-parsing rules by their section identifier.
    #[must_use]
    pub fn group_rules_by_section(&self) -> HashMap<String, Vec<&dyn StructuralValidationRule>> {
        let mut map: HashMap<String, Vec<&dyn StructuralValidationRule>> = HashMap::new();
        for rule in &self.pre_rules {
            map.entry(rule.section().to_string())
                .or_default()
                .push(rule.as_ref());
        }
        map
    }

    /// Runs all pre-parsing rules against the given feed source.
    ///
    /// Rules within each section are executed **in parallel** via rayon.
    /// Sections themselves run sequentially to maintain the gate ordering.
    #[must_use]
    pub fn validate_structural(&self, source: &FeedSource) -> ValidationReport {
        let grouped = self.group_rules_by_section();

        let mut sections: Vec<&String> = grouped.keys().collect();
        sections.sort();

        let multi = MultiProgress::new();
        if self.config.quiet || !std::io::stderr().is_terminal() {
            multi.set_draw_target(ProgressDrawTarget::hidden());
        }

        let mut all_errors: Vec<ValidationError> = Vec::new();

        for section_key in sections {
            let rules = &grouped[section_key];
            let label = section_label(section_key);

            let pb = multi.add(ProgressBar::new(rules.len() as u64));
            pb.set_style(BAR_STYLE.clone());
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

        // --- Single-pass scanner for merged formatting rules ---
        // Runs encoding, delimiter, quoting, content, whitespace, and
        // new_line_in_value checks in one pass per file, in parallel.
        let file_names = source.file_names();
        let scanner_errors: Vec<ValidationError> = file_names
            .par_iter()
            .flat_map(|&file| {
                let Ok(bytes) = source.read_file_bytes(file) else {
                    return Vec::new();
                };
                scanner::scan(file, &bytes)
            })
            .collect();
        all_errors.extend(scanner_errors);

        ValidationReport::from(all_errors)
    }

    /// Runs post-parsing validation rules against a loaded `GtfsFeed`.
    ///
    /// Also converts any `ParseError`s into `ValidationError`s with
    /// appropriate rule IDs from section 3.
    #[must_use]
    pub fn validate_feed(&self, feed: &GtfsFeed, parse_errors: &[ParseError]) -> ValidationReport {
        let mut all_errors: Vec<ValidationError> = parse_error_converter::convert(parse_errors);

        let multi = MultiProgress::new();
        if self.config.quiet || !std::io::stderr().is_terminal() {
            multi.set_draw_target(ProgressDrawTarget::hidden());
        }

        let label = section_label("3");
        let pb = multi.add(ProgressBar::new(self.rules.len() as u64));
        pb.set_style(BAR_STYLE.clone());
        pb.set_message(label.to_string());

        let rule_errors: Vec<ValidationError> = self
            .rules
            .par_iter()
            .flat_map(|rule| {
                let errors = rule.validate(feed);
                pb.inc(1);
                errors
            })
            .collect();

        all_errors.extend(rule_errors);
        pb.finish();

        ValidationReport::from(all_errors)
    }
}
