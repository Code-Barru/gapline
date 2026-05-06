//! `gapline validate` — runs the full validation pipeline.
//!
//! Accepts one or two feed paths and auto-detects whether each is a GTFS
//! Schedule (zip / directory) or a GTFS-Realtime protobuf. With one feed
//! the matching pipeline runs; with one of each, the RT feed is
//! cross-validated against the Schedule.

use std::fs::File;
use std::io::{IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

use gapline_core::config::Config;
use gapline_core::models::rt::GtfsRtFeed;
use gapline_core::validation::{ValidationEngine, ValidationReport};

use super::super::exit;
use super::super::output::render_report;
use super::super::parser::OutputFormat;
use super::{load_dataset_or_exit, resolve_feeds, resolve_format, resolve_output};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum FeedKind {
    Schedule,
    RealTime,
}

pub fn run_validate(
    config: &Arc<Config>,
    feeds: &[PathBuf],
    format: Option<OutputFormat>,
    output: Option<&Path>,
) {
    let feeds = resolve_feeds(feeds, config);

    let mut schedule_path: Option<&Path> = None;
    let mut rt_path: Option<&Path> = None;
    for p in &feeds {
        match detect_feed_kind(p) {
            FeedKind::Schedule => {
                if schedule_path.is_some() {
                    tracing::error!(
                        "two Schedule feeds provided; expected at most one Schedule + one GTFS-RT"
                    );
                    process::exit(exit::COMMAND_FAILED);
                }
                schedule_path = Some(p.as_path());
            }
            FeedKind::RealTime => {
                if rt_path.is_some() {
                    tracing::error!(
                        "two GTFS-RT feeds provided; expected at most one Schedule + one GTFS-RT"
                    );
                    process::exit(exit::COMMAND_FAILED);
                }
                rt_path = Some(p.as_path());
            }
        }
    }

    let (report, report_path) = match (schedule_path, rt_path) {
        (Some(sched), None) => (validate_schedule(sched, config), sched),
        (sched, Some(rt)) => (validate_rt(rt, sched, config), rt),
        (None, None) => {
            tracing::error!("no feed resolved; this should never happen after resolve_feeds");
            process::exit(exit::COMMAND_FAILED);
        }
    };

    let fmt = resolve_format(format, config);
    let out = resolve_output(output, config);
    if let Err(e) = render_report(&report, fmt, report_path, out.as_deref(), config) {
        eprintln!("Error while rendering report: {e}");
        process::exit(exit::COMMAND_FAILED);
    }

    if report.has_errors() {
        process::exit(exit::COMMAND_FAILED);
    }
}

fn validate_schedule(path: &Path, config: &Arc<Config>) -> ValidationReport {
    match gapline_core::validation::validate(path, Arc::clone(config)) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            process::exit(exit::INPUT_ERROR);
        }
    }
}

fn validate_rt(rt_path: &Path, schedule: Option<&Path>, config: &Arc<Config>) -> ValidationReport {
    let rt_spinner = make_spinner(config, "Loading RT feed...");
    let rt = GtfsRtFeed::from_file(rt_path).unwrap_or_else(|e| {
        rt_spinner.finish_and_clear();
        tracing::error!("invalid protobuf: {e}");
        process::exit(exit::INPUT_ERROR);
    });
    rt_spinner.finish_and_clear();

    let dataset_holder = schedule.map(load_dataset_or_exit);
    let schedule_feed = dataset_holder.as_ref().map(|(d, _)| d.feed());
    if schedule_feed.is_none() {
        tracing::info!("no Schedule feed provided: cross-validation rules skipped");
    }

    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());

    let engine = ValidationEngine::new(Arc::clone(config));
    engine.validate_rt(&rt, schedule_feed, now_unix)
}

fn make_spinner(config: &Config, message: &'static str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    let style = ProgressStyle::with_template("{spinner:.cyan} {msg}")
        .expect("hard-coded spinner template is valid")
        .tick_chars("⣷⣯⣟⡿⢿⣻⣽⣾ ");
    pb.set_style(style);
    pb.set_message(message);
    if config.output.show_progress && std::io::stderr().is_terminal() {
        pb.enable_steady_tick(Duration::from_millis(100));
    } else {
        pb.set_draw_target(ProgressDrawTarget::hidden());
    }
    pb
}

fn detect_feed_kind(path: &Path) -> FeedKind {
    if path.is_dir() {
        return FeedKind::Schedule;
    }
    let mut buf = [0u8; 4];
    if let Ok(mut f) = File::open(path)
        && f.read(&mut buf).is_ok()
        && buf[0] == b'P'
        && buf[1] == b'K'
    {
        return FeedKind::Schedule;
    }
    FeedKind::RealTime
}
