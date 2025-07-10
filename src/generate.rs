use crate::OutputType;
use crate::ColorOption;
use crate::config::Config;

pub fn run(
    output: String,
    r#type: OutputType,
    sequence: u32,
    directory: &Option<String>,
    config_file: &Option<String>,
    _color: &ColorOption
) {
    let config = Config::new(
        config_file.as_deref(),
        directory.as_deref()
    ).expect("Failed to load config");

    println!(
        "Generate command with output: {}, type: {:?}, sequence: {}",
        output, r#type, sequence
    );
    // Add implementation here
}