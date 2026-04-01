//! Validates numeric ranges: latitude/longitude bounds, non-negative constraints.

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

fn range_err(file: &str, field: &str, value: &str, line: usize) -> ValidationError {
    ValidationError::new("number_out_of_range", "3", Severity::Error)
        .file(file)
        .field(field)
        .value(value)
        .line(line)
        .message(format!("Value out of allowed range for {field}: '{value}'"))
}

pub struct NumericRangeValidator;

impl ValidationRule for NumericRangeValidator {
    fn rule_id(&self) -> &'static str {
        "number_out_of_range"
    }

    fn section(&self) -> &'static str {
        "3"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (i, s) in feed.stops.iter().enumerate() {
            let line = i + 2;
            if let Some(lat) = &s.stop_lat
                && (lat.0 < -90.0 || lat.0 > 90.0)
            {
                errors.push(range_err("stops.txt", "stop_lat", &lat.0.to_string(), line));
            }
            if let Some(lon) = &s.stop_lon
                && (lon.0 < -180.0 || lon.0 > 180.0)
            {
                errors.push(range_err("stops.txt", "stop_lon", &lon.0.to_string(), line));
            }
        }

        for (i, s) in feed.shapes.iter().enumerate() {
            let line = i + 2;
            if s.shape_pt_lat.0 < -90.0 || s.shape_pt_lat.0 > 90.0 {
                errors.push(range_err(
                    "shapes.txt",
                    "shape_pt_lat",
                    &s.shape_pt_lat.0.to_string(),
                    line,
                ));
            }
            if s.shape_pt_lon.0 < -180.0 || s.shape_pt_lon.0 > 180.0 {
                errors.push(range_err(
                    "shapes.txt",
                    "shape_pt_lon",
                    &s.shape_pt_lon.0.to_string(),
                    line,
                ));
            }
        }

        for (i, p) in feed.pathways.iter().enumerate() {
            let line = i + 2;
            if let Some(len) = p.length
                && len < 0.0
            {
                errors.push(range_err("pathways.txt", "length", &len.to_string(), line));
            }
        }

        for (i, fa) in feed.fare_attributes.iter().enumerate() {
            let line = i + 2;
            if fa.price < 0.0 {
                errors.push(range_err(
                    "fare_attributes.txt",
                    "price",
                    &fa.price.to_string(),
                    line,
                ));
            }
        }

        for (i, st) in feed.stop_times.iter().enumerate() {
            let line = i + 2;
            if let Some(dist) = st.shape_dist_traveled
                && dist < 0.0
            {
                errors.push(range_err(
                    "stop_times.txt",
                    "shape_dist_traveled",
                    &dist.to_string(),
                    line,
                ));
            }
        }

        for (i, s) in feed.shapes.iter().enumerate() {
            let line = i + 2;
            if let Some(dist) = s.shape_dist_traveled
                && dist < 0.0
            {
                errors.push(range_err(
                    "shapes.txt",
                    "shape_dist_traveled",
                    &dist.to_string(),
                    line,
                ));
            }
        }

        errors
    }
}
