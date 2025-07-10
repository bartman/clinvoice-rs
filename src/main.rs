use clap::{Parser, Subcommand, ValueEnum};
use clap::CommandFactory;

#[derive(ValueEnum, Clone, Debug)]
enum ColorOption {
    Always,
    Auto,
    Never,
}

#[derive(Parser)]
struct Cli {
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
        output: String,
        #[clap(short, long)]
        r#type: OutputType,
        #[clap(short, long)]
        sequence: u32,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum LogFormat {
    Full,
    Day,
    Month,
    Year,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputType {
    Pdf,
    Txt,
}

mod parse;
mod data;
mod log;
mod generate;
mod config;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        None => {
            Cli::command().print_long_help().unwrap();
        }
        Some(Command::Log { format, dates }) => {
            log::run(format, &cli.directory, &cli.color, &dates)
        },
        Some(Command::Generate { output, r#type, sequence }) => {
            generate::run(output, r#type, sequence, &cli.directory, &cli.config, &cli.color)
        }
    }
}
