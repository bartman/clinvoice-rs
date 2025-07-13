use clap::{Parser, Subcommand, ValueEnum};
use clap::CommandFactory;
use crate::logger::TraceLevel;
use crate::color::*;

#[derive(Parser)]
struct Cli {
    #[clap(short = 'D', long, help = "select trace level (error, warn, [info], debug, trace)", default_value = "info")]
    debug: TraceLevel,
    #[clap(short, long, help = "select directory with .cli files")]
    directory: Option<String>,
    #[clap(short, long, help = "select configuration file, use .clinvoice otherwise")]
    config: Option<String>,
    #[clap(short = 'C', long, default_value = "auto")]
    color: ColorOption,
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    #[clap(about = "Display existing entries")]
    Log {
        #[clap(short, long, default_value = "day")]
        format: LogFormat,
        #[clap(value_parser)]
        dates: Vec<String>,
    },
    #[clap(about = "Generate an invoice")]
    Generate {
        #[clap(short, long)]
        output: Option<String>,
        #[clap(short, long)]
        generator: Option<String>,
        #[clap(short, long)]
        sequence: Option<u32>,
        #[clap(value_parser)]
        dates: Vec<String>,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum LogFormat {
    Full,
    Day,
    Month,
    Year,
}

mod color;
mod config;
mod data;
mod generate;
mod latex;
mod log;
mod logger;
mod parse;

fn main() {
    let cli = Cli::parse();
    color::init(&cli.color);
    logger::init(&cli.debug);
    match cli.command {
        None => {
            Cli::command().print_long_help().unwrap();
        }
        Some(Command::Log { format, dates }) => {
            log::run(format, &cli.directory, &dates)
        },
        Some(Command::Generate { output, generator, sequence, dates }) => {
            generate::run(output, &generator, &sequence, &cli.directory, &cli.config, &dates)
        }
    }
}
