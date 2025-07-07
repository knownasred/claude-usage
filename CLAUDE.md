# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development
- `nix develop` - Enter development shell with all dependencies
- `nix develop -c cargo run` - Run the main application directly
- `cargo run` - Run the main CLI application (requires dev shell)
- `cargo run -- --help` - Show CLI help and options

### Build & Test
- `nix build` - Build the project using Nix
- `cargo build` - Build using Cargo (requires dev shell)
- `cargo test` - Run tests
- `cargo run --example basic_usage` - Run the basic usage example

### Development Tools
- `just` - Show available just commands
- `just run [ARGS]` - Run cargo with optional arguments
- `just watch [ARGS]` - Auto-recompile and run using bacon
- `just pre-commit-all` - Run pre-commit hooks on all files (formatting, linting)
- `pre-commit run -a` - Alternative way to run pre-commit hooks

### Nix Operations
- `nix flake update` - Update all flake inputs
- `nix --accept-flake-config run github:juspay/omnix ci` - Build all outputs

## Architecture

This is a Rust workspace with two main crates:

### claude-usage-lib (Core Library)
Located in `crates/claude-usage-lib/`, this is the main functionality library for Claude API usage monitoring and analysis. Key modules:
- `calculator.rs` - Usage calculations and projections
- `data_structures.rs` - Core data types (UsageEntry, TokenCounts, BurnRate, etc.)
- `identifier.rs` - Session identification
- `loader.rs` - Data loading functionality
- `monitor.rs` - Usage monitoring
- `pricing.rs` - Pricing calculations

The library exports a convenient `prelude` module with commonly used types.

### claude-usage (CLI Application)
Located in `crates/claude-usage/`, this is a simple CLI app that uses clap for argument parsing. Currently serves as a basic "hello world" style application but can be extended to use the core library.

## Development Environment

- Uses Nix flakes for reproducible development environment
- Includes pre-commit hooks for automatic formatting and linting
- VSCode configuration included for immediate IDE experience
- Uses rust-flake and crane for Rust/Nix integration
- Supports both Nix-based and rustup-based CI workflows

## Key Dependencies

- **serde/serde_json** - JSON serialization
- **chrono** - Date/time handling with serde support
- **anyhow** - Error handling
- **clap** - CLI argument parsing (main app)
- **tokio** - Async runtime (dev dependencies)