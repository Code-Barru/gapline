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

**MVP Phase :**

- **Complete validation**: engine implementing 200+ rules from the GTFS Schedule specification
- **CRUD operations**: create, read, update, and delete on core GTFS files
- **Batch processing**: `.hw` files to automate GTFS workflows
- **Multi-format**: output in colored text, JSON, XML, or CSV
- **Performance**: parallel rule execution, target ≥2x faster than MobilityData validator
- **Integrity protection**: automatic referential constraint verification
- **Configuration**: three-tier TOML configuration system (CLI > project > user > defaults)

**Future vision:**

- Interactive TUI interface for feed exploration
- GTFS feed merging
- GTFS Fares v2, GTFS-Flex, and GTFS-Realtime support
- Plugin ecosystem

## Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo

### Build

```bash
cargo build --release
```

The binary will be available at `target/release/headway`.

### Install via Cargo (coming soon)

```bash
cargo install headway
```

## Usage

### Command Examples

```bash
# Validate a GTFS feed
headway validate -f ./feed.zip --format json -o report.json

# Read data
headway read stops -f ./feed.zip --where "location_type=1"

# Update data
headway update stops -f ./feed.zip --where stop_id=S01 --set stop_name="New Station"

# Delete data
headway delete stop_times -f ./feed.zip --where "trip_id=OLD AND stop_sequence>10"

# Execute a batch file
headway run weekly-fix.hw
```

### Development

```bash
# Run in development mode
cargo run
```

## Development

### Run tests

```bash
cargo test
```

### Run benchmarks

```bash
cargo bench
```

### Format code

```bash
cargo fmt
```

### Lint

```bash
cargo clippy
```

## Current Status

The project is still in incubation and the validation engine is being written.

## License

This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.
