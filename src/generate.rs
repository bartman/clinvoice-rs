use crate::ColorOption;
use crate::config::Config;

pub fn run(
    output: String,
    generator: &Option<String>,
    sequence: &Option<u32>,
    directory: &Option<String>,
    config_file: &Option<String>,
    _color: &ColorOption
) {
    let config = Config::new(
        config_file.as_deref(),
        directory.as_deref()
    ).expect("Failed to load config");

    println!(
        "Generate command with output: {}, generator: {:?}, sequence: {}",
        output,
        generator.as_ref().expect("generator not specified"),
        sequence.as_ref().expect("sequence not specified")
    );
    // Add implementation here
}
