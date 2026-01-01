use clap::{Parser, Subcommand, CommandFactory};
use crate::tracing::TraceLevel;
use crate::color::*;
use crate::log::LogFormat;

mod color;
mod config;
mod data;
mod generate;
mod heatmap;
mod index;
mod latex;
mod log;
mod markdown;
mod parse;
mod tracing;

use clap::builder::styling::{Styles, AnsiColor, Effects, Style};
pub(crate) const HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub(crate) const USAGE: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub(crate) const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub(crate) const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
pub(crate) const ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);
pub(crate) const VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub(crate) const INVALID: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);
pub(crate) const CLAP_STYLING: Styles = Styles::styled()
    .header(HEADER)
    .usage(USAGE)
    .literal(LITERAL)
    .placeholder(PLACEHOLDER)
    .error(ERROR)
    .valid(VALID)
    .invalid(INVALID);

/// Command-line interface arguments for the clinvoice application.
#[derive(Parser)]
#[clap(about,version)]
#[clap(styles = CLAP_STYLING)]
struct Cli {
    #[clap(short = 'l', long, help = "select log level (error, warn, [info], debug, trace)", default_value = "info")]
    log_level: TraceLevel,
    #[clap(short = 'L', long, help = "select log destination file (- is stderr", default_value = "-")]
    log_file: String,
    #[clap(short, long, help = "select directory with .cli files")]
    directory: Option<String>,
    #[clap(short, long, help = "select configuration file, use .clinvoice otherwise")]
    config: Option<String>,
    #[clap(short = 'C', long, default_value = "auto")]
    color: ColorOption,
    #[clap(subcommand)]
    command: Option<Command>,
}

/// Subcommands for the clinvoice application.
#[derive(Subcommand)]
enum Command {

    /// Display existing entries
    #[clap(about = "Display existing entries")]
    Log {
        #[clap(short, long, default_value = "day")]
        format: LogFormat,
        #[clap(value_parser)]
        dates: Vec<String>,
    },

    /// Generate an invoice
    #[clap(about = "Generate an invoice")]
    Generate {
        #[clap(short, long)]
        output: Option<String>,
        #[clap(short, long)]
        generator: Option<String>,
        #[clap(short, long)]
        sequence: Option<u32>,
        #[clap(short = 'b', long, help = "Backdate invoice to specified date (YYYYMMDD, YYYY-MM-DD, or YYYY.MM.DD)")]
        backdate: Option<String>,
        #[clap(value_parser)]
        dates: Vec<String>,
    },

    /// Display a heatmap of entries
    #[clap(about = "Display a heatmap of entries")]
    Heatmap {
        #[clap(value_parser)]
        dates: Vec<String>,
    },
}

/// Main entry point of the clinvoice application.
fn main() {
    let cli = Cli::parse();
    color::init(&cli.color);
    tracing::init(&cli.log_level, &cli.log_file);
    match cli.command {
        None => {
            Cli::command().print_long_help().unwrap();
        }
        Some(Command::Log { format, dates }) => {
            log::run(format, &cli.directory, &dates)
        },
        Some(Command::Generate { output, generator, sequence, backdate, dates }) => {
            generate::run(output, &generator, &sequence, &backdate, &cli.directory, &cli.config, &dates)
        },
        Some(Command::Heatmap { dates }) => {
            heatmap::run(&cli.directory, &dates)
        }
    }
}
