# Headway

A high-performance, all-in-one CLI tool for manipulating GTFS (General Transit Feed Specification) files.

## About

Headway replaces the fragmented ecosystem of current GTFS tools with a single unified, fast, and local-first binary written in Rust.

### Problem Solved

Transit data engineers and application developers face:

- **Tool fragmentation**: different tools for each operation (validation, editing, merging)
- **Performance issues**: Java/Python solutions are slow on large feeds
- **Privacy concerns**: cloud validators require sending data to third parties
- **Heavy maintenance**: custom ad-hoc scripts for each operation
- **Outdated tools**: validators that don't always follow the latest GTFS specifications

### Key Features

**Available now:**

- **Comprehensive validation**: 6-section gated pipeline with 60+ rules covering file structure, CSV formatting, field types, field definitions, foreign keys, and primary key uniqueness
- **17 GTFS file types parsed**: agency, stops, routes, trips, stop_times, calendar, calendar_dates, shapes, frequencies, transfers, pathways, levels, feed_info, fare_attributes, fare_rules, translations, attributions
- **Multi-format output**: colored text (default) and JSON
- **Performance**: parallel rule execution and file parsing via rayon
- **Integrity protection**: bidirectional referential integrity index with recursive dependency tracking

**Coming soon:**

- **CRUD operations**: create, read, update, and delete on core GTFS files
- **Batch processing**: `.hw` files to automate GTFS workflows
- **CSV and XML output formats**
- **TOML configuration system**: three-tier config (project > user > defaults)

## Installation

### Quick Install (recommended)

**Linux / macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/Code-Barru/headway/main/scripts/install.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/Code-Barru/headway/main/scripts/install.ps1 | iex
```

### Install a Specific Version

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/Code-Barru/headway/main/scripts/install.sh | sh -s -- --version 0.3.0

# Windows (PowerShell)
$env:HEADWAY_VERSION="0.3.0"; irm https://raw.githubusercontent.com/Code-Barru/headway/main/scripts/install.ps1 | iex
```

### Build from Source

**Prerequisites:** Rust 1.70 or higher

```bash
cargo build --release
```

The binary will be available at `target/release/headway`.

## Usage

### Validate a GTFS feed

```bash
# Validate with colored terminal output
headway validate -f ./feed.zip

# Validate and export as JSON
headway validate -f ./feed.zip --format json -o report.json
```

### CRUD operations (coming soon)

```bash
# Read data
headway read stops -f ./feed.zip --where "location_type=1"

# Create data
headway create stops -f ./feed.zip --set stop_id=S99 --set stop_name="New Stop"

# Update data
headway update stops -f ./feed.zip --where stop_id=S01 --set stop_name="New Station"

# Delete data
headway delete stop_times -f ./feed.zip --where "trip_id=OLD AND stop_sequence>10"
```

### Batch execution (coming soon)

```bash
headway run weekly-fix.hw
```

## Development

### Run in development mode

```bash
cargo run
```

### Run tests

```bash
cargo test
```

### Run benchmarks

```bash
cargo bench -p headway-core
```

Criterion HTML reports are generated in `target/criterion/`.

### Flamegraph

Requires [`cargo-flamegraph`](https://github.com/flamegraph-rs/flamegraph) and `perf` (Linux):

```bash
cargo install flamegraph
cargo flamegraph --bench core_bench -p headway-core -- --bench
```

The SVG flamegraph is generated at the project root (`flamegraph.svg`).

### Format code

```bash
cargo fmt
```

### Lint

```bash
cargo clippy
```

## License

This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.
