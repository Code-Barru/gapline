# Changelog

All notable changes to `cli` will be documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

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
