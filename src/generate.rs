use crate::OutputType;

pub fn run(
    output: String,
    r#type: OutputType,
    sequence: u32,
    _directory: &Option<String>,
    _config: &Option<String>,
) {
    println!(
        "Generate command with output: {}, type: {:?}, sequence: {}",
        output, r#type, sequence
    );
    // Add implementation here
}
