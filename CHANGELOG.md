# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.1] - 2026-03-20

### Added
- Added a project changelog in Keep a Changelog format.

### Changed
- Aligned `grpr --version` with the Cargo package version and release tag policy.
- Updated the crate to Rust 2024 and raised the documented minimum supported Rust version to 1.85.
- Refreshed direct dependencies to newer current releases, including `clap`, `rayon`, `walkdir`, and `tempfile`.

## [2.0.0] - 2026-03-20

### Added
- Added git worktree support alongside regular repository detection.
- Added regression coverage for repository discovery, traversal boundaries, and multi-argument git command handling.
- Expanded README documentation with behavior, examples, development notes, and compatibility details.

### Changed
- Reworked repository discovery and traversal to better match the Go `grp` implementation while remaining idiomatic Rust.
- Made sequential execution the default and kept threaded execution as an explicit opt-in with `--threads`.
- Tightened repository validation so regular repositories require `.git/config` and worktrees require a valid `gitdir:` reference.

## [1.1.0] - 2024-10-26

### Added
- Added concurrent repository processing for improved performance.
- Added a `--threads` option to control the worker count for parallel execution.

### Changed
- Updated Cargo.lock compatibility for older Cargo consumers.

## [1.0.0] - 2022-02-23

### Added
- Initial Rust release of `grpr`.
- Recursive git command execution across repositories in the current directory tree.
- Basic CLI scaffolding and project packaging for building and installing the tool.
