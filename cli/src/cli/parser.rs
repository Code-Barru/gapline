use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

/// Top-level CLI argument parser for headway.
///
/// Uses [clap](https://docs.rs/clap) derive API with git-style subcommands.
/// Version is propagated from `Cargo.toml` to all subcommands.
///
/// # Usage
///
/// ```no_run
/// use clap::Parser;
/// use headway::cli::Cli;
///
/// let cli = Cli::parse();
/// ```
#[derive(Debug, Parser)]
#[command(version, about = "A high-performance, all-in-one CLI tool for manipulating and validating GTFS files.", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands for headway.
///
/// Follows a git-style pattern: `headway <subcommand> [options]`.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Validate a GTFS feed against the full specification.
    #[command(about = "Validates a GTFS feed")]
    Validate {
        /// Path to the GTFS feed (`.zip` archive or decompressed directory).
        #[arg(short, long, value_name = "FEED", help = "GTFS path feed")]
        feed: PathBuf,
        /// Output format for the validation report.
        #[arg(
            long,
            help = "Output format: json, csv, xml and text",
            hide_possible_values = true
        )]
        format: Option<OutputFormat>,
        /// Write the report to a file instead of stdout.
        #[arg(short, long, value_name = "PATH", help = "Output path")]
        output: Option<PathBuf>,
    },
    /// Read and query data from a GTFS file.
    #[command(about = "Read and query GTFS fields")]
    Read {
        /// Path to the GTFS feed.
        #[arg(short, long, value_name = "FEED", help = "GTFS path feed")]
        feed: PathBuf,
        /// Filter expression using the mini query language.
        #[arg(short, long = "where", value_name = "QUERY", help = "SQL-like query")]
        where_query: Option<String>,
        /// Which GTFS file to read from.
        #[arg(
            help = "GTFS file (e.g. calendar, calendar-dates, stops, stop-times)",
            hide_possible_values = true
        )]
        target: CrudTarget,
        /// Output format for the results.
        #[arg(
            long,
            help = "Output format: json, csv, xml and text",
            hide_possible_values = true
        )]
        format: Option<OutputFormat>,
        /// Write the results to a file instead of stdout.
        #[arg(short, long, value_name = "PATH", help = "Output path")]
        output: Option<PathBuf>,
    },
    /// Insert new records into a GTFS file.
    #[command(about = "Insert GTFS fields into a feed")]
    Create {
        /// Path to the GTFS feed.
        #[arg(short, long, value_name = "FEED", help = "GTFS path feed")]
        feed: PathBuf,
        /// Field values to set on the new record (e.g. `stop_id=NEW_01`).
        #[arg(short, long, num_args = 1.., help = "Fields to set (e.g. stop_id=NEW_01 stop_name=\"Test\")")]
        set: Vec<String>,
        /// Which GTFS file to insert into.
        #[arg(
            help = "GTFS file (e.g. calendar, calendar-dates, stops, stop-times)",
            hide_possible_values = true
        )]
        target: CrudTarget,
        /// Skip the interactive confirmation prompt.
        #[arg(long, help = "Skip confirm prompt")]
        confirm: bool,
        /// Write the modified feed to this path instead of overwriting the original.
        #[arg(short, long, value_name = "PATH", help = "Output path")]
        output: Option<PathBuf>,
    },
    /// Update existing records in a GTFS file.
    #[command(about = "Update GTFS field in a feed")]
    Update {
        /// Path to the GTFS feed.
        #[arg(short, long, value_name = "FEED", help = "GTFS path feed")]
        feed: PathBuf,
        /// Filter expression to select records to update (required).
        #[arg(short, long = "where", value_name = "QUERY", help = "SQL-like query")]
        where_query: String,
        /// Field values to set on matched records (required).
        #[arg(short, long, num_args = 1.., required = true, help = "Fields to set (e.g. stop_id=NEW_01 stop_name=\"Test\")")]
        set: Vec<String>,
        /// Which GTFS file to update.
        #[arg(
            help = "GTFS file (e.g. calendar, calendar-dates, stops, stop-times)",
            hide_possible_values = true
        )]
        target: CrudTarget,
        /// Skip the interactive confirmation prompt.
        #[arg(long, help = "Skip confirm prompt")]
        confirm: bool,
        /// Cascade PK changes to referencing records in dependent files.
        #[arg(long, help = "Cascade PK changes to dependent records")]
        cascade: bool,
        /// Write the modified feed to this path instead of overwriting the original.
        #[arg(short, long, value_name = "PATH", help = "Output path")]
        output: Option<PathBuf>,
    },
    /// Delete records from a GTFS file.
    #[command(about = "Delete GTFS records from a feed")]
    Delete {
        /// Path to the GTFS feed.
        #[arg(short, long, value_name = "FEED", help = "GTFS path feed")]
        feed: PathBuf,
        /// Filter expression to select records to delete.
        #[arg(short, long = "where", value_name = "QUERY", help = "SQL-like query")]
        where_query: Option<String>,
        /// Which GTFS file to delete from.
        #[arg(
            help = "GTFS file (e.g. calendar, calendar-dates, stops, stop-times)",
            hide_possible_values = true
        )]
        target: CrudTarget,
        /// Skip the interactive confirmation prompt.
        #[arg(long, help = "Skip confirm prompt")]
        confirm: bool,
        /// Write the modified feed to this path instead of overwriting the original.
        #[arg(short, long, value_name = "PATH", help = "Output path")]
        output: Option<PathBuf>,
    },
    /// Execute a sequence of headway commands from a `.hw` batch file.
    #[command(about = "Execute headway commands from a .hw file")]
    Run {
        /// Path to the `.hw` batch file.
        #[arg(value_name = "file.hw", help = "Headway file path")]
        file: PathBuf,
    },
}

/// Supported output formats for validation reports and query results.
///
/// Selectable via the `--format` CLI flag. When not specified, defaults to
/// colored terminal text. Colors are automatically disabled when stdout is not
/// a TTY (e.g. pipe or redirect).
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    /// Machine-readable JSON.
    Json,
    /// Flat tabular CSV.
    Csv,
    /// Standard XML.
    Xml,
    /// Human-readable colored terminal text (default).
    Text,
}

/// GTFS files that support CRUD operations.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum CrudTarget {
    /// `agency.txt` -- Transit agencies.
    Agency,
    /// `stops.txt` -- Stop/station locations.
    Stops,
    /// `routes.txt` -- Route definitions.
    Routes,
    /// `trips.txt` -- Trip definitions.
    Trips,
    /// `stop_times.txt` -- Arrival/departure times at stops.
    #[value(alias = "stop_times")]
    StopTimes,
    /// `calendar.txt` -- Weekly service schedules.
    Calendar,
    /// `calendar_dates.txt` -- Service exceptions by date.
    #[value(alias = "calendar_dates")]
    CalendarDates,
    /// `shapes.txt` -- Geographic shape points.
    Shapes,
    /// `frequencies.txt` -- Headway-based service.
    Frequencies,
    /// `transfers.txt` -- Transfer rules between stops.
    Transfers,
    /// `pathways.txt` -- Station pathways.
    Pathways,
    /// `levels.txt` -- Station levels.
    Levels,
    /// `feed_info.txt` -- Feed metadata.
    #[value(alias = "feed_info")]
    FeedInfo,
    /// `fare_attributes.txt` -- Fare definitions.
    #[value(alias = "fare_attributes")]
    FareAttributes,
    /// `fare_rules.txt` -- Fare assignment rules.
    #[value(alias = "fare_rules")]
    FareRules,
    /// `translations.txt` -- Translated field values.
    Translations,
    /// `attributions.txt` -- Dataset attributions.
    Attributions,
}

impl CrudTarget {
    /// Converts this CLI target to the core [`GtfsTarget`](headway_core::crud::read::GtfsTarget).
    #[must_use]
    pub fn to_target(self) -> headway_core::crud::read::GtfsTarget {
        use headway_core::crud::read::GtfsTarget;
        match self {
            Self::Agency => GtfsTarget::Agency,
            Self::Stops => GtfsTarget::Stops,
            Self::Routes => GtfsTarget::Routes,
            Self::Trips => GtfsTarget::Trips,
            Self::StopTimes => GtfsTarget::StopTimes,
            Self::Calendar => GtfsTarget::Calendar,
            Self::CalendarDates => GtfsTarget::CalendarDates,
            Self::Shapes => GtfsTarget::Shapes,
            Self::Frequencies => GtfsTarget::Frequencies,
            Self::Transfers => GtfsTarget::Transfers,
            Self::Pathways => GtfsTarget::Pathways,
            Self::Levels => GtfsTarget::Levels,
            Self::FeedInfo => GtfsTarget::FeedInfo,
            Self::FareAttributes => GtfsTarget::FareAttributes,
            Self::FareRules => GtfsTarget::FareRules,
            Self::Translations => GtfsTarget::Translations,
            Self::Attributions => GtfsTarget::Attributions,
        }
    }
}
