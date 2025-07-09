use crate::OutputType;
use crate::ColorOption;

pub fn run(
    output: String,
    r#type: OutputType,
    sequence: u32,
    _directory: &Option<String>,
    _config: &Option<String>,
    _color: &ColorOption
) {
    println!(
        "Generate command with output: {}, type: {:?}, sequence: {}",
        output, r#type, sequence
    );
    // Add implementation here
}
