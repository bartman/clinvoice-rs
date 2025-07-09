use clap::{Parser, Subcommand, ValueEnum};
use clap::CommandFactory;

#[derive(Parser)]
struct Cli {
    #[clap(short, long)]
    directory: Option<String>,
    #[clap(short, long)]
    config: Option<String>,
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Log {
        #[clap(short, long)]
        format: Format,
    },
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
pub enum Format {
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

mod data;
mod log;
mod generate;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        None => {
            Cli::command().print_long_help().unwrap();
        }
        Some(Command::Log { format }) => {
            log::run(format, &cli.directory, &cli.config)
        },
        Some(Command::Generate { output, r#type, sequence }) => {
            generate::run(output, r#type, sequence, &cli.directory, &cli.config)
        }
    }
}
