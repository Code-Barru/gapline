use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(version, about = "A high-performance, all-in-one CLI tool for manipulating and validating GTFS files.", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Validates a GTFS feed")]
    Validate {
        #[arg(short, long, value_name = "FEED", help = "GTFS path feed")]
        feed: PathBuf,
        #[arg(
            long,
            help = "Output format: json, csv, xml and text",
            hide_possible_values = true
        )]
        format: Option<OutputFormat>,
        #[arg(short, long, value_name = "PATH", help = "Output path")]
        output: Option<PathBuf>,
    },
    #[command(about = "Read and query GTFS fields")]
    Read {
        #[arg(short, long, value_name = "FEED", help = "GTFS path feed")]
        feed: PathBuf,
        #[arg(short, long = "where", value_name = "QUERY", help = "SQL-like query")]
        where_query: Option<String>,
        #[arg(
            help = "GTFS file (e.g. calendar, calendar-dates, stops, stop-times)",
            hide_possible_values = true
        )]
        target: CrudTarget,
        #[arg(
            long,
            help = "Output format: json, csv, xml and text",
            hide_possible_values = true
        )]
        format: Option<OutputFormat>,
        #[arg(short, long, value_name = "PATH", help = "Output path")]
        output: Option<PathBuf>,
    },
    #[command(about = "Insert GTFS fields into a feed")]
    Create {
        #[arg(short, long, value_name = "FEED", help = "GTFS path feed")]
        feed: PathBuf,
        #[arg(short, long, help = "Fields to set (e.g. stop_id=NEW_01)")]
        set: Option<String>,
        #[arg(
            help = "GTFS file (e.g. calendar, calendar-dates, stops, stop-times)",
            hide_possible_values = true
        )]
        target: CrudTarget,
        #[arg(long, help = "Skip confirm prompt")]
        confirm: bool,
    },
    #[command(about = "Update GTFS field in a feed")]
    Update {
        #[arg(short, long, value_name = "FEED", help = "GTFS path feed")]
        feed: PathBuf,
        #[arg(short, long = "where", value_name = "QUERY", help = "SQL-like query")]
        where_query: Option<String>,
        #[arg(short, long, help = "Fields to set (e.g. stop_id=NEW_01)")]
        set: Option<String>,
        #[arg(
            help = "GTFS file (e.g. calendar, calendar-dates, stops, stop-times)",
            hide_possible_values = true
        )]
        target: CrudTarget,
        #[arg(long, help = "Skip confirm prompt")]
        confirm: bool,
    },
    #[command(about = "Delete GTFS field in a feed")]
    Delete {
        #[arg(short, long, value_name = "FEED", help = "GTFS path feed")]
        feed: PathBuf,
        #[arg(short, long = "where", value_name = "QUERY", help = "SQL-like query")]
        where_query: Option<String>,
        #[arg(
            help = "GTFS file (e.g. calendar, calendar-dates, stops, stop-times)",
            hide_possible_values = true
        )]
        target: CrudTarget,
        #[arg(long, help = "Skip confirm prompt")]
        confirm: bool,
    },
    #[command(about = "Execute headway commands from a .hw file")]
    Run {
        #[arg(value_name = "file.hw", help = "Headway file path")]
        file: PathBuf,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum OutputFormat {
    Json,
    Csv,
    Xml,
    Text,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum CrudTarget {
    Trips,
    Stops,
    StopTimes,
    Calendar,
    CalendarDates,
}
