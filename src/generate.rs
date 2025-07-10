use crate::ColorOption;
use crate::config::Config;

pub fn run(
    output: String,
    generator_option: &Option<String>,
    sequence_option: &Option<u32>,
    directory: &Option<String>,
    config_file: &Option<String>,
    _color: &ColorOption
) {
    let config = Config::new(
        config_file.as_deref(),
        directory.as_deref()
    ).expect("Failed to load config");

    let use_generator = if let Some(selected) = generator_option {
        selected
    } else {
        &config.get("generator.default").unwrap().to_string()
    };

    let use_sequence:u32 = if let Some(selected) = sequence_option {
        *selected
    } else {
        // write code that looks it up
        1
    };

    println!("Generate command with output: {}, generator: {}, sequence: {}",
        output, use_generator, use_sequence);

}
