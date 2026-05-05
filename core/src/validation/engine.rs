//! Validation engine - orchestrates rule execution against a feed source.
//!
//! The engine collects both structural (pre-parsing) and semantic (post-parsing)
//! validation rules, groups them by GTFS specification section, and runs them
//! sequentially. Sections 1-2 run against raw `FeedSource`; sections 3+ run
//! against the parsed `GtfsFeed`.

use std::collections::{HashMap, HashSet};
use std::io::IsTerminal;
use std::sync::{Arc, LazyLock};

use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use rayon::prelude::*;

static BAR_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    // Pad the message to the width of the longest label
    // ("Field Definition Validation" = 27 chars) so every bar starts at the
    // same column.
    ProgressStyle::with_template("{msg:27} [{bar:30.cyan/dim}] {pos}/{len}")
        .expect("hard-coded progress template is valid")
        .progress_chars("█░░")
});

use crate::config::Config;
use crate::models::GtfsFeed;
use crate::parser::FeedSource;
use crate::parser::error::ParseError;
use crate::validation::csv_formatting::scanner;
use crate::validation::field_type::parse_error_converter;
use crate::validation::flex_semantic::register_rules as register_flex_semantic_rules;
use crate::validation::schedule_time_validation::{
    CalendarThresholds, DistanceThresholds, SpeedThresholds, TransferThresholds,
    service_dates::ServiceDateCache,
};
use crate::validation::{
    StructuralValidationRule, ValidationError, ValidationReport, ValidationRule,
};

fn progress_label(group: &str) -> &str {
    match group {
        "1" => "File Structure",
        "2" => "CSV Formatting",
        "3" => "Field Validation",
        "5" => "Key & Reference Validation",
        "7" => "Semantic & Logic",
        "8" => "Best Practices",
        "13" => "Third-Party Validators",
        _ => "Validation",
    }
}

/// Maps a rule's `progress_group` to a display group so that related
/// sections share one progress bar without touching each rule struct.
fn display_group(progress_group: &str) -> &str {
    match progress_group {
        "4" => "3",        // Field Definition → merged with Field Type
        "6" => "5",        // Primary Key → merged with Foreign Key
        "9" | "10" => "7", // Flex / Fares v2 Semantic → merged with Semantic & Logic
        g => g,
    }
}

/// Orchestrates validation of a GTFS feed.
///
/// # Example
///
/// ```no_run
/// use std::sync::Arc;
/// use gapline_core::config::Config;
/// use gapline_core::parser::FeedLoader;
/// use gapline_core::validation::engine::ValidationEngine;
///
/// let config = Arc::new(Config::default());
/// let engine = ValidationEngine::new(config);
/// let source = FeedLoader::open(std::path::Path::new("feed.zip")).unwrap();
/// let report = engine.validate_structural(&source);
/// ```
pub struct ValidationEngine {
    config: Arc<Config>,
    /// Pre-parsing rules (sections 1-2) operating on raw `FeedSource`.
    pre_rules: Vec<Box<dyn StructuralValidationRule>>,
    /// Post-parsing rules (sections 3+) operating on the loaded `GtfsFeed`.
    rules: Vec<Box<dyn ValidationRule>>,
}

/// Bundle of every threshold struct needed to register rules - built once in
/// [`ValidationEngine::new`] from the global [`Config`].
struct Thresholds {
    max_trip_duration_hours: Option<u32>,
    max_route_short_name_length: usize,
    distance: DistanceThresholds,
    calendar: CalendarThresholds,
    transfer: TransferThresholds,
    speed: SpeedThresholds,
    service_cache: Arc<ServiceDateCache>,
}

impl Thresholds {
    fn from_config(config: &Config) -> Self {
        let t = &config.validation.thresholds;
        Self {
            max_trip_duration_hours: t.time.max_trip_duration_hours,
            max_route_short_name_length: t.naming.max_route_short_name_length,
            distance: DistanceThresholds {
                max_stop_to_shape_distance_m: t.distances.max_stop_to_shape_distance_m,
                min_shape_point_distance_m: t.distances.min_shape_point_distance_m,
                shape_dist_incoherence_ratio: t.distances.shape_dist_incoherence_ratio,
                min_distance_from_origin_m: t.coordinates.min_distance_from_origin_m,
                min_distance_from_poles_m: t.coordinates.min_distance_from_poles_m,
            },
            calendar: CalendarThresholds {
                min_feed_coverage_days: t.calendar.min_feed_coverage_days,
                feed_expiration_warning_days: t.calendar.feed_expiration_warning_days,
                min_trip_activity_days: t.calendar.min_trip_activity_days,
                reference_date: t.calendar.reference_date,
            },
            transfer: TransferThresholds {
                max_transfer_distance_m: t.distances.max_transfer_distance_m,
                transfer_distance_warning_m: t.distances.transfer_distance_warning_m,
            },
            speed: SpeedThresholds {
                tram_kmh: t.speed_limits.tram_kmh,
                subway_kmh: t.speed_limits.subway_kmh,
                rail_kmh: t.speed_limits.rail_kmh,
                bus_kmh: t.speed_limits.bus_kmh,
                ferry_kmh: t.speed_limits.ferry_kmh,
                cable_tram_kmh: t.speed_limits.cable_tram_kmh,
                aerial_lift_kmh: t.speed_limits.aerial_lift_kmh,
                funicular_kmh: t.speed_limits.funicular_kmh,
                trolleybus_kmh: t.speed_limits.trolleybus_kmh,
                monorail_kmh: t.speed_limits.monorail_kmh,
                default_kmh: t.speed_limits.default_kmh,
            },
            service_cache: Arc::new(ServiceDateCache::new()),
        }
    }
}

impl ValidationEngine {
    /// Creates a new engine pre-loaded with all registered rules.
    #[must_use]
    pub fn new(config: Arc<Config>) -> Self {
        let max_rows = config.validation.max_rows;
        let t = Thresholds::from_config(&config);

        // Aggregate pre-parsing rules from each owning module. The 6
        // content-scanning rules (encoding, delimiter, quoting, content,
        // whitespace, new_line_in_value) are handled by the single-pass
        // scanner in validate_structural and are not listed here.
        let mut pre_rules = crate::validation::file_structure::pre_rules(max_rows);
        pre_rules.extend(crate::validation::csv_formatting::pre_rules());

        let mut engine = Self {
            config,
            pre_rules,
            rules: Vec::new(),
        };
        crate::validation::field_type::register_rules(&mut engine);
        crate::validation::field_definition::register_rules(&mut engine);
        crate::validation::primary_key::register_rules(&mut engine);
        crate::validation::foreign_key::register_rules(&mut engine);
        crate::validation::schedule_time_validation::register_rules(
            &mut engine,
            t.max_trip_duration_hours,
            t.distance,
            t.calendar,
            t.transfer,
            t.speed,
            t.service_cache.clone(),
        );
        register_flex_semantic_rules(&mut engine, t.service_cache);
        crate::validation::fares_v2_semantic::register_rules(&mut engine);
        let naming_thresholds = crate::validation::best_practices::NamingThresholds {
            max_route_short_name_length: t.max_route_short_name_length,
        };
        crate::validation::best_practices::register_rules(&mut engine, naming_thresholds);
        crate::validation::third_party::register_rules(&mut engine);

        // Apply [validation.disabled_rules] / [validation.enabled_rules].
        // Blacklist beats whitelist when both are set.
        let disabled: HashSet<&str> = engine
            .config
            .validation
            .disabled_rules
            .iter()
            .map(String::as_str)
            .collect();
        let enabled: HashSet<&str> = engine
            .config
            .validation
            .enabled_rules
            .iter()
            .map(String::as_str)
            .collect();
        if !disabled.is_empty() || !enabled.is_empty() {
            engine.pre_rules.retain(|r| {
                let id = r.rule_id();
                !disabled.contains(id) && (enabled.is_empty() || enabled.contains(id))
            });
            engine.rules.retain(|r| {
                let id = r.rule_id();
                !disabled.contains(id) && (enabled.is_empty() || enabled.contains(id))
            });
        }

        engine
    }

    /// All pre-parsing (sections 1–2) rules currently registered with
    /// the engine, in registration order. Used by `gapline rules list`
    /// to enumerate available rule IDs for `disabled_rules` /
    /// `enabled_rules` configuration.
    #[must_use]
    pub fn pre_rules(&self) -> &[Box<dyn StructuralValidationRule>] {
        &self.pre_rules
    }

    /// All post-parsing (sections 3+) rules currently registered with
    /// the engine, in registration order.
    #[must_use]
    pub fn post_rules(&self) -> &[Box<dyn ValidationRule>] {
        &self.rules
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
        if !self.config.output.show_progress || !std::io::stderr().is_terminal() {
            multi.set_draw_target(ProgressDrawTarget::hidden());
        }

        let mut all_errors: Vec<ValidationError> = Vec::new();

        for section_key in sections {
            let rules = &grouped[section_key];
            let label = progress_label(section_key);

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

    /// Groups the registered post-parsing rules by their progress-bar group.
    /// Rules can override `progress_group` to split themselves out of the
    /// default section-based bucket (e.g. geometric rules of section 7).
    fn group_post_rules_by_section(&self) -> HashMap<String, Vec<&dyn ValidationRule>> {
        let mut map: HashMap<String, Vec<&dyn ValidationRule>> = HashMap::new();
        for rule in &self.rules {
            map.entry(display_group(rule.progress_group()).to_string())
                .or_default()
                .push(rule.as_ref());
        }
        map
    }

    /// Runs post-parsing validation rules against a loaded `GtfsFeed`.
    ///
    /// Also converts any `ParseError`s into `ValidationError`s with
    /// appropriate rule IDs from section 3.
    ///
    /// All sections run in parallel since every rule operates on the
    /// immutable `&GtfsFeed`. Each section gets its own progress bar.
    #[must_use]
    pub fn validate_feed(&self, feed: &GtfsFeed, parse_errors: &[ParseError]) -> ValidationReport {
        let mut all_errors: Vec<ValidationError> = parse_error_converter::convert(parse_errors);

        let grouped = self.group_post_rules_by_section();
        let mut sections: Vec<&String> = grouped.keys().collect();
        sections.sort();

        let multi = MultiProgress::new();
        if !self.config.output.show_progress || !std::io::stderr().is_terminal() {
            multi.set_draw_target(ProgressDrawTarget::hidden());
        }

        // Create all progress bars up-front so they display together.
        let section_bars: Vec<(&String, &Vec<&dyn ValidationRule>, ProgressBar)> = sections
            .iter()
            .map(|&key| {
                let rules = &grouped[key];
                let pb = multi.add(ProgressBar::new(rules.len() as u64));
                pb.set_style(BAR_STYLE.clone());
                pb.set_message(progress_label(key).to_string());
                (key, rules, pb)
            })
            .collect();

        // Run all sections in parallel via rayon.
        let rule_errors: Vec<ValidationError> = section_bars
            .par_iter()
            .flat_map(|(_, rules, pb)| {
                let errors: Vec<ValidationError> = rules
                    .par_iter()
                    .flat_map(|rule| {
                        let errors = rule.validate(feed);
                        pb.inc(1);
                        errors
                    })
                    .collect();
                pb.finish();
                errors
            })
            .collect();

        all_errors.extend(rule_errors);

        ValidationReport::from(all_errors)
    }
}
