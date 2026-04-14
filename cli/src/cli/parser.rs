use std::fmt;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use headway_core::validation::Severity;

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

    /// Path to a TOML config file. Overrides `./headway.toml` in the
    /// lookup chain. The global `~/.config/headway/config.toml` is still
    /// consulted as a lower-priority layer.
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Disable colored output, even when stdout is a TTY.
    #[arg(long, global = true, conflicts_with = "force_color")]
    pub no_color: bool,

    /// Force colored output, even when stdout is not a TTY.
    #[arg(long, global = true, conflicts_with = "no_color")]
    pub force_color: bool,

    /// Number of worker threads for parallel validation. Auto-detected
    /// when omitted.
    #[arg(long, global = true, value_name = "N")]
    pub threads: Option<usize>,
}

/// CLI alias for [`headway_core::validation::Severity`].
///
/// Lives here so clap can derive `ValueEnum` without dragging clap into
/// `headway-core`. Maps 1:1 to the core enum via [`SeverityArg::to_core`].
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum SeverityArg {
    Error,
    Warning,
    Info,
}

impl SeverityArg {
    #[must_use]
    pub fn to_core(self) -> Severity {
        match self {
            Self::Error => Severity::Error,
            Self::Warning => Severity::Warning,
            Self::Info => Severity::Info,
        }
    }
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
        /// May be omitted if `[default] feed` is set in the config file.
        #[arg(short, long, value_name = "FEED", help = "Path to GTFS feed")]
        feed: Option<PathBuf>,
        /// Output format for the validation report.
        #[arg(
            long,
            help = "Output format: json, csv, xml, html and text",
            hide_possible_values = true
        )]
        format: Option<OutputFormat>,
        /// Write the report to a file instead of stdout.
        #[arg(short, long, value_name = "PATH", help = "Output path")]
        output: Option<PathBuf>,
        /// Minimum severity to display in the report. Findings below this
        /// level are filtered from both the listing and the summary counts.
        #[arg(
            long,
            value_name = "LEVEL",
            help = "Minimum severity: error, warning, info"
        )]
        min_severity: Option<SeverityArg>,
        /// Disable a validation rule by ID. May be passed multiple times.
        /// Appends to the `disabled_rules` list from the config file.
        /// Shell completion suggests every registered rule ID.
        #[arg(
            long = "disable-rule",
            value_name = "RULE_ID",
            num_args = 1..,
            value_parser = clap::builder::PossibleValuesParser::new(
                headway_core::validation::all_rule_ids().iter().copied(),
            ),
        )]
        disable_rule: Vec<String>,
    },
    /// Read and query data from a GTFS file.
    #[command(about = "Read and query GTFS fields")]
    Read {
        /// Path to the GTFS feed. Optional when `[default] feed` is set.
        #[arg(short, long, value_name = "FEED", help = "Path to GTFS feed")]
        feed: Option<PathBuf>,
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
            help = "Output format: json, csv, xml, html and text",
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
        /// Path to the GTFS feed. Optional when `[default] feed` is set.
        #[arg(short, long, value_name = "FEED", help = "Path to GTFS feed")]
        feed: Option<PathBuf>,
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
        /// Path to the GTFS feed. Optional when `[default] feed` is set.
        #[arg(short, long, value_name = "FEED", help = "Path to GTFS feed")]
        feed: Option<PathBuf>,
        /// Filter expression to select records to update (required).
        #[arg(
            short,
            long = "where",
            value_name = "QUERY",
            required = true,
            help = "SQL-like query (required)"
        )]
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
        /// Path to the GTFS feed. Optional when `[default] feed` is set.
        #[arg(short, long, value_name = "FEED", help = "Path to GTFS feed")]
        feed: Option<PathBuf>,
        /// Filter expression to select records to delete (required).
        #[arg(
            short,
            long = "where",
            value_name = "QUERY",
            required = true,
            help = "SQL-like query (required)"
        )]
        where_query: String,
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
    /// Inspect the validation rules registered with this build.
    #[command(about = "List or inspect validation rules")]
    Rules {
        /// The `rules` subcommand to execute.
        #[command(subcommand)]
        command: RulesCommand,
    },
    /// Generate or install a shell completion script.
    #[command(about = "Generate or install a shell completion script")]
    Completion {
        /// Target shell (bash, zsh, fish, elvish, powershell).
        #[arg(value_name = "SHELL", help = "Shell to generate completion for")]
        shell: clap_complete::Shell,
        /// Install the script to the shell's standard completion directory
        /// instead of printing to stdout. Only bash, zsh and fish are
        /// supported for installation.
        #[arg(long, help = "Install to the shell's standard completion directory")]
        install: bool,
    },
}

/// Subcommands of `headway rules`.
#[derive(Debug, Subcommand)]
pub enum RulesCommand {
    /// List every validation rule registered with this build.
    #[command(about = "List every registered validation rule")]
    List {
        /// Restrict the listing to rules with this severity.
        #[arg(
            long,
            value_name = "LEVEL",
            help = "Filter by severity: error, warning, info"
        )]
        severity: Option<SeverityArg>,
        /// Output format. Defaults to text.
        #[arg(
            long,
            help = "Output format: json, csv, xml, html and text",
            hide_possible_values = true
        )]
        format: Option<OutputFormat>,
        /// Write the listing to a file instead of stdout.
        #[arg(short, long, value_name = "PATH", help = "Output path")]
        output: Option<PathBuf>,
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
    /// Self-contained HTML report with inlined CSS/JS.
    Html,
    /// Human-readable colored terminal text (default).
    Text,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Xml => "xml",
            Self::Html => "html",
            Self::Text => "text",
        })
    }
}

impl OutputFormat {
    /// Parses an `[default] format` value loaded from the config file.
    /// Returns `None` for unrecognized values — the caller decides whether
    /// that is a hard error or a fall-through to the default.
    #[must_use]
    pub fn from_config_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "json" => Some(Self::Json),
            "csv" => Some(Self::Csv),
            "xml" => Some(Self::Xml),
            "html" => Some(Self::Html),
            "text" => Some(Self::Text),
            _ => None,
        }
    }
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
