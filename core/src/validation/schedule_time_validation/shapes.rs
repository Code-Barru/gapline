//! Shape geometry validation for `shapes.txt` (section 7.3).
//!
//! Emits at most four distinct aggregated findings per shape:
//! - `degenerate_shape` - a shape with fewer than 2 points.
//! - `duplicate_shape_point` - N consecutive point pairs with identical
//!   coordinates (Haversine distance == 0).
//! - `shape_points_too_close` - N consecutive point pairs closer than the
//!   configured minimum distance (0 < d < threshold).
//! - `shape_dist_traveled_incoherent` - N segments whose declared
//!   `shape_dist_traveled` increment diverges from the shape's median
//!   declared/Haversine ratio by more than the configured tolerance. This
//!   approach is unit-agnostic: whether the feed uses meters, kilometers,
//!   miles, or feet, the median ratio captures the unit implicitly.

use std::collections::HashMap;

use crate::geo::haversine_meters;
use crate::models::{GtfsFeed, Shape};
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "shapes.txt";
const SECTION: &str = "7";

/// Validates geometric consistency of shape points within each shape.
pub struct ShapesGeometryRule {
    min_point_distance_m: f64,
    incoherence_ratio: f64,
}

impl ShapesGeometryRule {
    #[must_use]
    pub fn new(min_point_distance_m: f64, incoherence_ratio: f64) -> Self {
        Self {
            min_point_distance_m,
            incoherence_ratio,
        }
    }
}

impl ValidationRule for ShapesGeometryRule {
    fn rule_id(&self) -> &'static str {
        "shape_geometry"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Group shape points by shape_id, preserving original CSV indices so
        // we can surface accurate line numbers in findings.
        let mut groups: HashMap<&str, Vec<(usize, &Shape)>> = HashMap::new();
        for (i, shape) in feed.shapes.iter().enumerate() {
            groups
                .entry(shape.shape_id.as_ref())
                .or_default()
                .push((i, shape));
        }

        for (shape_id, points) in &groups {
            let mut sorted = points.clone();
            sorted.sort_by_key(|&(_, s)| s.shape_pt_sequence);

            if sorted.len() < 2 {
                let (idx, _) = sorted[0];
                errors.push(
                    ValidationError::new("degenerate_shape", SECTION, Severity::Warning)
                        .message(format!(
                            "shape '{shape_id}' has {} point(s); a shape needs at least 2",
                            sorted.len()
                        ))
                        .file(FILE)
                        .line(idx + 2)
                        .field("shape_id")
                        .value(*shape_id),
                );
                continue;
            }

            self.scan_shape(&mut errors, shape_id, &sorted);
        }

        errors
    }
}

/// Per-shape aggregated findings. One instance per shape; emits 0-3 warnings
/// at the end of the scan.
#[derive(Default)]
struct ShapeFindings {
    total_pairs: usize,
    duplicate: PairCounter,
    too_close: PairCounter,
    // For the incoherence check, we track worst divergence for diagnostics.
    incoherent_count: usize,
    incoherent_first_line: usize,
    incoherent_first_seq_prev: u32,
    incoherent_first_seq_curr: u32,
    worst_divergence: f64,
    segments_with_dist: usize,
}

#[derive(Default)]
struct PairCounter {
    count: usize,
    first_line: usize,
    first_seq_prev: u32,
    first_seq_curr: u32,
    first_distance_m: f64,
}

impl PairCounter {
    fn record(&mut self, line: usize, seq_prev: u32, seq_curr: u32, distance_m: f64) {
        if self.count == 0 {
            self.first_line = line;
            self.first_seq_prev = seq_prev;
            self.first_seq_curr = seq_curr;
            self.first_distance_m = distance_m;
        }
        self.count += 1;
    }
}

impl ShapesGeometryRule {
    fn scan_shape(
        &self,
        errors: &mut Vec<ValidationError>,
        shape_id: &str,
        sorted: &[(usize, &Shape)],
    ) {
        let mut findings = ShapeFindings::default();
        // Segment ratios (declared / haversine) for the median computation.
        let mut segment_ratios: Vec<f64> = Vec::new();
        // Per-segment raw data kept for the second pass (after median known).
        let mut segments: Vec<SegmentData> = Vec::new();

        for pair in sorted.windows(2) {
            let (_, prev) = pair[0];
            let (idx, curr) = pair[1];
            let line = idx + 2;
            findings.total_pairs += 1;

            let d = haversine_meters(
                prev.shape_pt_lat.0,
                prev.shape_pt_lon.0,
                curr.shape_pt_lat.0,
                curr.shape_pt_lon.0,
            );

            if d == 0.0 {
                findings.duplicate.record(
                    line,
                    prev.shape_pt_sequence,
                    curr.shape_pt_sequence,
                    0.0,
                );
            } else if d < self.min_point_distance_m {
                findings
                    .too_close
                    .record(line, prev.shape_pt_sequence, curr.shape_pt_sequence, d);
            }

            if let (Some(prev_dist), Some(curr_dist)) =
                (prev.shape_dist_traveled, curr.shape_dist_traveled)
                && d > 0.0
            {
                let declared = curr_dist - prev_dist;
                let ratio = declared / d;
                segment_ratios.push(ratio);
                segments.push(SegmentData {
                    line,
                    seq_prev: prev.shape_pt_sequence,
                    seq_curr: curr.shape_pt_sequence,
                    ratio,
                });
            }
        }

        findings.segments_with_dist = segments.len();
        if let Some(median) = median(&mut segment_ratios.clone()) {
            for seg in &segments {
                let divergence = ((seg.ratio - median) / median).abs();
                if divergence > self.incoherence_ratio {
                    if findings.incoherent_count == 0 {
                        findings.incoherent_first_line = seg.line;
                        findings.incoherent_first_seq_prev = seg.seq_prev;
                        findings.incoherent_first_seq_curr = seg.seq_curr;
                    }
                    if divergence > findings.worst_divergence {
                        findings.worst_divergence = divergence;
                    }
                    findings.incoherent_count += 1;
                }
            }
            self.emit_incoherent(errors, shape_id, &findings, median);
        }

        emit_duplicate(errors, shape_id, &findings);
        self.emit_too_close(errors, shape_id, &findings);
    }
}

fn emit_duplicate(errors: &mut Vec<ValidationError>, shape_id: &str, findings: &ShapeFindings) {
    let c = &findings.duplicate;
    if c.count == 0 {
        return;
    }
    errors.push(
        ValidationError::new("duplicate_shape_point", SECTION, Severity::Warning)
            .message(format!(
                "shape '{shape_id}' has {} consecutive duplicate point pair(s) \
                     (first: sequences {}={} at line {})",
                c.count, c.first_seq_prev, c.first_seq_curr, c.first_line
            ))
            .file(FILE)
            .line(c.first_line)
            .field("shape_pt_sequence")
            .value(c.first_seq_curr.to_string()),
    );
}

impl ShapesGeometryRule {
    fn emit_too_close(
        &self,
        errors: &mut Vec<ValidationError>,
        shape_id: &str,
        findings: &ShapeFindings,
    ) {
        let c = &findings.too_close;
        if c.count == 0 {
            return;
        }
        errors.push(
            ValidationError::new("shape_points_too_close", SECTION, Severity::Warning)
                .message(format!(
                    "shape '{shape_id}' has {} consecutive point pair(s) closer than {:.2}m \
                     (first: sequences {}→{} are {:.2}m apart at line {})",
                    c.count,
                    self.min_point_distance_m,
                    c.first_seq_prev,
                    c.first_seq_curr,
                    c.first_distance_m,
                    c.first_line
                ))
                .file(FILE)
                .line(c.first_line)
                .field("shape_pt_sequence")
                .value(c.first_seq_curr.to_string()),
        );
    }

    fn emit_incoherent(
        &self,
        errors: &mut Vec<ValidationError>,
        shape_id: &str,
        findings: &ShapeFindings,
        median: f64,
    ) {
        if findings.incoherent_count == 0 {
            return;
        }
        let unit = detect_unit_label(median);
        errors.push(
            ValidationError::new("shape_dist_traveled_incoherent", SECTION, Severity::Warning)
                .message(format!(
                    "shape '{shape_id}' has {}/{} segment(s) with shape_dist_traveled inconsistent \
                     with detected unit '{unit}' (median ratio={median:.6}, \
                     worst divergence {:.1}%, threshold {:.0}%, first offender: sequences {}→{} at line {})",
                    findings.incoherent_count,
                    findings.segments_with_dist,
                    findings.worst_divergence * 100.0,
                    self.incoherence_ratio * 100.0,
                    findings.incoherent_first_seq_prev,
                    findings.incoherent_first_seq_curr,
                    findings.incoherent_first_line
                ))
                .file(FILE)
                .line(findings.incoherent_first_line)
                .field("shape_dist_traveled")
                .value(findings.incoherent_first_seq_curr.to_string()),
        );
    }
}

struct SegmentData {
    line: usize,
    seq_prev: u32,
    seq_curr: u32,
    ratio: f64,
}

/// Median of a slice of f64. Returns `None` if empty. Mutates the slice
/// (partial sort). NaN values are treated as `Ordering::Equal` - shouldn't
/// occur in practice since we filter d > 0 before division.
fn median(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        Some(f64::midpoint(values[mid - 1], values[mid]))
    } else {
        Some(values[mid])
    }
}

/// Identifies a plausible unit name from the median declared/Haversine ratio.
/// The ratio is declared-units per meter. Known units and their ±20% bands:
/// - `1.0` → meters
/// - `0.001` → kilometers
/// - `0.000_621_371` → miles
/// - `3.280_84` → feet
fn detect_unit_label(ratio: f64) -> &'static str {
    if (0.8..=1.2).contains(&ratio) {
        "meters"
    } else if (0.000_8..=0.001_2).contains(&ratio) {
        "kilometers"
    } else if (0.000_497..=0.000_746).contains(&ratio) {
        "miles"
    } else if (2.624..=3.937).contains(&ratio) {
        "feet"
    } else {
        "unknown"
    }
}
