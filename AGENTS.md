# Gemini Agent Context: `clinvoice-rs`

This document provides a summary of the `clinvoice-rs` project to guide the Gemini agent.

## Project Overview

`clinvoice-rs` is a command-line interface (CLI) application written in Rust for generating invoices. It processes timesheet data from `.cli` files, uses a TOML configuration file for company and bank data, and generates invoices using templates processed by the Tera crate. Multiple generators are supported, including one for LaTeX output and another for plain text.

## Core Components & Logic

- **Enduser documentation:**
  - Use `README.md` to learn about how the user will use this project.
  - Update `README.md` after adding features.

- **Code and comments:**
  - Keep functions short and use descriptive names.
  - Prioritize code legibility over optimization.
  - Document all public functions/structs/enums/etc in rustdoc format.
  - Add short 1-2 line descriptions for non-exported items.
  - Use minimal comments in function bodies, only if necessary.

- **Entry Point (`src/main.rs`):**
  - Uses the `clap` crate to parse command-line arguments:
    - `--directory`: Path to the invoice data files.
    - `--config`: Path to the TOML config file (defaults to `clinvoice.toml`).
    - `--output`: Path for the generated output file (default uses config file to generate path).
    - `--log-level`: Selects logging verbosity (error, warn, info, debug, trace).
    - `--log-file`: Destination for log output (defaults to stderr).
    - `--color`: Controls color output (always, auto, never).
  - Supports subcommands:
    - `log`: Displays existing timesheet entries.
    - `generate`: Generates an invoice.

- **Data Structures (`src/data.rs`):**
  - Defines core data models: `Entry` (Time, FixedCost, Note), `DateRange`, `DateSelector`, and `TimeData`.
  - `TimeData` holds all records from `.cli` timesheet files.
  - `DateSelector` filters dates based on specified ranges.

- **Configuration (`src/config.rs`):**
  - Defines `Config` struct for holding data from TOML config file.
  - `read_config` function reads and parses the specified TOML file.
  - Configuration includes invoice issuer, recipient, contract, and tax information used as variables in templates.
  - Supports dot notation for nested keys and provides methods for type-specific value retrieval.

- **Parsing (`src/parse.rs`):**
  - Contains helper functions for parsing dates, time ranges, and `.cli` file entries.
  - Supports various date formats (YYYY.MM.DD, YYYYMMDD, YYYY-MM-DD).
  - Parses time specifications (e.g., "8h", "9:00-17:00") into hours.
  - Processes lines into `Entry` types (Time, FixedCost, Note).

- **Hour Log (`src/log.rs`):**
  - Generates a log of hours worked from `.cli` timesheet files in the specified directory.
  - Supports 4 levels of detail via `--format` flag:
    - `full`: Shows every timesheet entry.
    - `day`: Provides a per-day summary.
    - `month`: Summarizes to each month.
    - `year`: Summarizes to each year.

- **Generation (`src/generate.rs` & `src/latex.rs`):**
  - `generate.rs` orchestrates invoice generation, including sequence management, data loading, and template rendering.
  - Supports caps on hours per day and per invoice, calculates totals, taxes, and discounts.
  - Uses `tera` templating engine for rendering templates.
  - `latex.rs` provides LaTeX-specific escaping for special characters.
  - Supports custom build commands post-generation (e.g., compiling LaTeX to PDF).
  - Custom Tera filters for date formatting and text justification.

- **Index Management (`src/index.rs`):**
  - Manages invoice sequence numbers to ensure uniqueness using an index file.
  - Supports locking for concurrent access safety.
  - Maps sequence numbers to associated date ranges.

- **Debug Logging (`src/tracing.rs`, `src/color.rs`):**
  - `tracing.rs` configures logging with customizable levels and output destinations using the `tracing` crate.
  - `color.rs` manages colored terminal output based on user preference or terminal detection.
  - Provides `DynamicColorize` trait for conditional coloring of strings.

## Testing

- Unit tests are kept in code files for simple function testing.
- Integration tests are in `tests/*.rs` files for complex tasks and file operations.

## Dependencies

- **`clap`**: CLI argument parsing.
- **`serde` / `toml`**: Deserializing `clinvoice.toml` config.
- **`chrono`**: Date parsing and handling.
- **`tera`**: Templating engine for invoice generation.
- **`colored` / `atty`**: For styled (colored) terminal output.
- **`tracing` / `tracing-subscriber`**: For application logging.
- **`fs2`**: File locking for index management.

## Build & Run

- The project is built using a standard `cargo build` command, as defined in the `Makefile`.
- To generate a log output: `cargo run -- --directory examples log`
- To generate a text invoice to stdout: `cargo run -- --directory examples generate -g txt -o -`
- To generate a LaTeX invoice to stdout: `cargo run -- --directory examples generate -g latex -o -`
- To invoke pdflatex to generate a PDF: `cargo run -- --directory examples generate -g pdf -o tmp.tex` (produces `tmp.pdf`)