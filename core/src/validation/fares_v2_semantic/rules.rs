use std::collections::{HashMap, HashSet};

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const SECTION: &str = "10";
const FARE_PRODUCTS: &str = "fare_products.txt";
const FARE_TRANSFER_RULES: &str = "fare_transfer_rules.txt";
const TIMEFRAMES: &str = "timeframes.txt";
const AREAS: &str = "areas.txt";

pub struct NegativeAmountRule;

impl ValidationRule for NegativeAmountRule {
    fn rule_id(&self) -> &'static str {
        "fares_negative_amount"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_fares_v2() {
            return Vec::new();
        }
        let mut errors = Vec::new();
        for (i, fp) in feed.fare_products.iter().enumerate() {
            if fp.amount < 0.0 {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Error)
                        .message("amount must be >= 0")
                        .file(FARE_PRODUCTS)
                        .line(i + 2)
                        .field("amount")
                        .value(fp.amount.to_string()),
                );
            }
        }
        errors
    }
}

pub struct ZeroAmountRule;

impl ValidationRule for ZeroAmountRule {
    fn rule_id(&self) -> &'static str {
        "fares_zero_amount"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_fares_v2() {
            return Vec::new();
        }
        let mut errors = Vec::new();
        for (i, fp) in feed.fare_products.iter().enumerate() {
            if fp.amount == 0.0 {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Warning)
                        .message("amount is zero")
                        .file(FARE_PRODUCTS)
                        .line(i + 2)
                        .field("amount")
                        .value("0"),
                );
            }
        }
        errors
    }
}

pub struct TimeframeOverlapRule;

impl ValidationRule for TimeframeOverlapRule {
    fn rule_id(&self) -> &'static str {
        "fares_timeframe_overlap"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_fares_v2() {
            return Vec::new();
        }
        let mut groups: HashMap<&str, Vec<usize>> = HashMap::new();
        for (i, tf) in feed.timeframes.iter().enumerate() {
            groups
                .entry(tf.timeframe_group_id.as_ref())
                .or_default()
                .push(i);
        }
        let mut errors = Vec::new();
        for indices in groups.values_mut() {
            indices.sort_by_key(|&i| feed.timeframes[i].start_time.total_seconds);
            for (j, &a_idx) in indices.iter().enumerate() {
                let a = &feed.timeframes[a_idx];
                for &b_idx in indices.iter().skip(j + 1) {
                    let b = &feed.timeframes[b_idx];
                    if a.end_time.total_seconds <= b.start_time.total_seconds {
                        break;
                    }
                    errors.push(
                        ValidationError::new(self.rule_id(), SECTION, Severity::Warning)
                            .message("timeframe overlaps another in the same group")
                            .file(TIMEFRAMES)
                            .line(b_idx + 2)
                            .field("start_time")
                            .value(b.start_time.to_string()),
                    );
                }
            }
        }
        errors
    }
}

pub struct InvalidTransferCountRule;

impl ValidationRule for InvalidTransferCountRule {
    fn rule_id(&self) -> &'static str {
        "fares_invalid_transfer_count"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_fares_v2() {
            return Vec::new();
        }
        let mut errors = Vec::new();
        for (i, ftr) in feed.fare_transfer_rules.iter().enumerate() {
            if let Some(n) = ftr.transfer_count
                && n <= 0
            {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Error)
                        .message("transfer_count must be > 0")
                        .file(FARE_TRANSFER_RULES)
                        .line(i + 2)
                        .field("transfer_count")
                        .value(n.to_string()),
                );
            }
        }
        errors
    }
}

pub struct ZeroDurationLimitRule;

impl ValidationRule for ZeroDurationLimitRule {
    fn rule_id(&self) -> &'static str {
        "fares_zero_duration_limit"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_fares_v2() {
            return Vec::new();
        }
        let mut errors = Vec::new();
        for (i, ftr) in feed.fare_transfer_rules.iter().enumerate() {
            if ftr.duration_limit == Some(0) {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Error)
                        .message("duration_limit must be > 0 when set")
                        .file(FARE_TRANSFER_RULES)
                        .line(i + 2)
                        .field("duration_limit")
                        .value("0"),
                );
            }
        }
        errors
    }
}

pub struct CircularTransferRule;

impl ValidationRule for CircularTransferRule {
    fn rule_id(&self) -> &'static str {
        "fares_circular_transfer"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_fares_v2() {
            return Vec::new();
        }
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
        for ftr in &feed.fare_transfer_rules {
            let (Some(from), Some(to)) = (&ftr.from_leg_group_id, &ftr.to_leg_group_id) else {
                continue;
            };
            graph.entry(from.as_ref()).or_default().push(to.as_ref());
        }
        let mut errors = Vec::new();
        for (i, ftr) in feed.fare_transfer_rules.iter().enumerate() {
            let (Some(from), Some(to)) = (&ftr.from_leg_group_id, &ftr.to_leg_group_id) else {
                continue;
            };
            // Self-loops price staying within a leg group; not a cycle.
            if from == to {
                continue;
            }
            if has_path(to.as_ref(), from.as_ref(), &graph) {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Warning)
                        .message("transfer rule participates in a cycle")
                        .file(FARE_TRANSFER_RULES)
                        .line(i + 2)
                        .field("to_leg_group_id")
                        .value(to.as_ref()),
                );
            }
        }
        errors
    }
}

fn has_path(start: &str, target: &str, graph: &HashMap<&str, Vec<&str>>) -> bool {
    let mut stack = vec![start];
    let mut visited: HashSet<&str> = HashSet::new();
    while let Some(node) = stack.pop() {
        if !visited.insert(node) {
            continue;
        }
        if let Some(neighbors) = graph.get(node) {
            for &n in neighbors {
                if n == target {
                    return true;
                }
                stack.push(n);
            }
        }
    }
    false
}

pub struct UnusedFareProductRule;

impl ValidationRule for UnusedFareProductRule {
    fn rule_id(&self) -> &'static str {
        "fares_unused_product"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_fares_v2() {
            return Vec::new();
        }
        let mut used: HashSet<&str> = HashSet::new();
        for r in &feed.fare_leg_rules {
            used.insert(r.fare_product_id.as_ref());
        }
        for r in &feed.fare_transfer_rules {
            if let Some(fp) = &r.fare_product_id {
                used.insert(fp.as_ref());
            }
        }
        let mut errors = Vec::new();
        for (i, fp) in feed.fare_products.iter().enumerate() {
            if !used.contains(fp.fare_product_id.as_ref()) {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Warning)
                        .message("fare_product is not referenced by any leg or transfer rule")
                        .file(FARE_PRODUCTS)
                        .line(i + 2)
                        .field("fare_product_id")
                        .value(fp.fare_product_id.to_string()),
                );
            }
        }
        errors
    }
}

pub struct EmptyAreaRule;

impl ValidationRule for EmptyAreaRule {
    fn rule_id(&self) -> &'static str {
        "fares_empty_area"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        if !feed.has_fares_v2() {
            return Vec::new();
        }
        let populated: HashSet<&str> = feed
            .stop_areas
            .iter()
            .map(|sa| sa.area_id.as_ref())
            .collect();
        let mut errors = Vec::new();
        for (i, area) in feed.areas.iter().enumerate() {
            if !populated.contains(area.area_id.as_ref()) {
                errors.push(
                    ValidationError::new(self.rule_id(), SECTION, Severity::Warning)
                        .message("area has no stops in stop_areas.txt")
                        .file(AREAS)
                        .line(i + 2)
                        .field("area_id")
                        .value(area.area_id.to_string()),
                );
            }
        }
        errors
    }
}
