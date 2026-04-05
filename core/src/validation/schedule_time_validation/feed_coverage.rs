//! Feed-wide temporal coverage validation (section 7.5).
//!
//! Emits:
//! - `short_feed_coverage` (WARNING): overall date span across `calendar.txt`
//!   and `calendar_dates.txt` is shorter than the configured threshold.
//! - `expired_feed` (WARNING): `feed_info.feed_end_date` is in the past.
//! - `feed_expiring_soon` (WARNING): `feed_info.feed_end_date` lies within
//!   the configured warning window from the reference date.

use chrono::Local;

use crate::models::{GtfsDate, GtfsFeed};
use crate::validation::{Severity, ValidationError, ValidationRule};

const SECTION: &str = "7";

/// Validates the overall feed coverage and expiration dates.
pub struct FeedCoverageRule {
    min_feed_coverage_days: u32,
    feed_expiration_warning_days: i64,
    reference_date: Option<GtfsDate>,
}

impl FeedCoverageRule {
    #[must_use]
    pub fn new(
        min_feed_coverage_days: u32,
        feed_expiration_warning_days: i64,
        reference_date: Option<GtfsDate>,
    ) -> Self {
        Self {
            min_feed_coverage_days,
            feed_expiration_warning_days,
            reference_date,
        }
    }

    fn today(&self) -> GtfsDate {
        self.reference_date
            .unwrap_or_else(|| GtfsDate(Local::now().date_naive()))
    }
}

impl ValidationRule for FeedCoverageRule {
    fn rule_id(&self) -> &'static str {
        "feed_coverage"
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn progress_group(&self) -> &'static str {
        "7-cal"
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        check_coverage(&mut errors, feed, self.min_feed_coverage_days);
        check_expiration(
            &mut errors,
            feed,
            self.today(),
            self.feed_expiration_warning_days,
        );

        errors
    }
}

/// Computes the feed's overall coverage as the union of `calendar.txt` ranges
/// and `calendar_dates.txt` dates, then warns when shorter than the threshold.
fn check_coverage(errors: &mut Vec<ValidationError>, feed: &GtfsFeed, min_days: u32) {
    let mut min_date: Option<GtfsDate> = None;
    let mut max_date: Option<GtfsDate> = None;

    for cal in &feed.calendars {
        // Ignore inverted ranges — CalendarRangesRule already flags those
        // and using them here would produce a nonsensical coverage span.
        if cal.start_date > cal.end_date {
            continue;
        }
        min_date = Some(min_date.map_or(cal.start_date, |d| d.min(cal.start_date)));
        max_date = Some(max_date.map_or(cal.end_date, |d| d.max(cal.end_date)));
    }

    for cd in &feed.calendar_dates {
        min_date = Some(min_date.map_or(cd.date, |d| d.min(cd.date)));
        max_date = Some(max_date.map_or(cd.date, |d| d.max(cd.date)));
    }

    let (Some(min), Some(max)) = (min_date, max_date) else {
        return;
    };

    let coverage_days = (max.0 - min.0).num_days() + 1;
    if coverage_days < i64::from(min_days) {
        let file = if feed.calendars.is_empty() {
            "calendar_dates.txt"
        } else {
            "calendar.txt"
        };
        errors.push(
            ValidationError::new("short_feed_coverage", SECTION, Severity::Warning)
                .message(format!(
                    "feed covers {coverage_days} day(s) ({min} → {max}) \
                     which is below the {min_days}-day threshold"
                ))
                .file(file)
                .line(1)
                .field("date")
                .value(format!("{min}..{max}")),
        );
    }
}

/// Checks `feed_info.feed_end_date` against the reference date for both
/// already-expired feeds and imminent expirations.
fn check_expiration(
    errors: &mut Vec<ValidationError>,
    feed: &GtfsFeed,
    today: GtfsDate,
    warning_days: i64,
) {
    let Some(feed_info) = &feed.feed_info else {
        return;
    };
    let Some(end_date) = feed_info.feed_end_date else {
        return;
    };

    let days_until_expiry = (end_date.0 - today.0).num_days();

    if days_until_expiry < 0 {
        errors.push(
            ValidationError::new("expired_feed", SECTION, Severity::Warning)
                .message(format!(
                    "feed_end_date {end_date} is in the past (reference date {today})"
                ))
                .file("feed_info.txt")
                .line(2)
                .field("feed_end_date")
                .value(end_date.to_string()),
        );
    } else if days_until_expiry < warning_days {
        errors.push(
            ValidationError::new("feed_expiring_soon", SECTION, Severity::Warning)
                .message(format!(
                    "feed_end_date {end_date} is in {days_until_expiry} day(s) \
                     (warning window: {warning_days} days)"
                ))
                .file("feed_info.txt")
                .line(2)
                .field("feed_end_date")
                .value(end_date.to_string()),
        );
    }
}
