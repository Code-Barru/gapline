# Changelog

All notable changes to `cli` will be documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [0.15.0] - 2026-04-14

### Features

- feat(cli): dynamic --disable-rule completion from registered rules
- feat(config): semantic validation for threshold values
- feat(cli): tracing-based logging + structured exit codes

## [0.14.1] - 2026-04-14

### Bug Fixes

- perf: remove unnecessary String allocations in CRUD create

## [0.14.0] - 2026-04-14

### Features

- feat: html reports
- feat: csv & xml output format
- feat: autocompletion
- feat: LIKE syntax in --where clause

## [0.13.0] - 2026-04-10

### Features

- feat: configuration management system

## [0.12.0] - 2026-04-09

### Features

- feat: run batch commands

## [0.11.0] - 2026-04-08

### Features

- feat: delete command
- feat: update command
- feat: create command
- feat: read command
- feat: query language parser

### Bug Fixes

- fix: create command processing time improve

## [0.10.0] - 2026-04-07

### Features

- feat: third party validation rules
- feat: best practices validation

### Bug Fixes

- fix(tests): now correctly handle changes in codebase

## [0.9.0] - 2026-04-07

### Features

- feat: block id, coordinates & unused entities

## [0.8.1] - 2026-04-06

### Bug Fixes

- fix: FK_VIOLATION in calendar_dates now as WARNING

## [0.8.0] - 2026-04-06

### Features

- feat: transfer, pathways & speed validation

## [0.7.0] - 2026-04-06

### Features

- feat: stop hierarchy & route type validation

## [0.6.0] - 2026-04-05

### Features

- feat: calendar & date logic

### Bug Fixes

- fix(ci): ferrflow bumps version BEFORE building releases

## [0.5.0] - 2026-04-04

### Features

- feat: shape & distance validation

### Bug Fixes

- fix: ci now correctly hashes releases

## [0.4.0] - 2026-04-04

### Features

- feat: schedule time validation
- feat: install script

### Bug Fixes

- fix: windows ps1 file now uses anonymous block
- fix: updated headway path installation
- fix: updated clippy errors + clippy fails on warning

## [0.3.0] - 2026-04-04

### Features

- feat: updated report rendering message

### Bug Fixes

- fix(ci): changed macOs identifier
- fix: tests weren't updated with severity changes
- fix: some rules had bad severity + clippy warnings

## [0.2.0] - 2026-04-03

### Features

- feat: update README.md
- feat: foreign key validation extended
- feat: foreign key validation
- feat: integrity index
- feat: primary key uniqueness validation
- feat: secondary field definition validation
- feat: field definition validation
- feat: spinner when loading gtfs into memory
- feat: field type validation
- feat: csv struct parser
- feat: gtfs data model
- feat: validation engine + validate command
- feat: csv formatting and encoding validation
- feat: structural file validation
- feat: ZIP archive and directory feed loading
- feat: output formatting layer
- feat: validation error structure and severity system
- feat: cli argument passing with subcommands

### Bug Fixes

- fix: missing column validation + in_quoted check
- fix: now handle accents like printable chars
