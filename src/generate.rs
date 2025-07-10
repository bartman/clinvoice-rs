use crate::OutputType;
use crate::ColorOption;
use std::path::{Path, PathBuf};
use crate::config::Config;
use std::env;

pub fn run(
    output: String,
    r#type: OutputType,
    sequence: u32,
    directory: &Option<String>,
    config_file: &Option<String>,
    _color: &ColorOption
) {
    let config_path = if let Some(path) = config_file {
        Path::new(path)

    } else {
        // if config file was not provided, use the first one that's found
        // - directory/clinvoice.toml
        // - clinvoice.toml
        // - ~/.config/clinvoice/clinvoice.toml

        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Some(dir) = directory {
            candidates.push(Path::new(dir).join("clinvoice.toml"));
        }
        candidates.push(PathBuf::from("clinvoice.toml"));
        if let Ok(home) = env::var("HOME") {
            candidates.push(Path::new(&home).join(".config").join("clinvoice").join("clinvoice.toml"));
        }
        &candidates.into_iter().find(|p| p.exists()).expect("No config file found")
    };
    let config = Config::new(config_path).expect("Failed to load config");

    println!(
        "Generate command with output: {}, type: {:?}, sequence: {}",
        output, r#type, sequence
    );
    // Add implementation here
}
