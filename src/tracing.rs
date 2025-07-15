use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use clap::{ValueEnum};
use std::fs::File;
use crate::color;

/// Defines the available tracing levels for logging.
#[derive(ValueEnum, Clone, Debug)]
pub enum TraceLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl TraceLevel {
    /// Returns the string representation of the trace level.
    pub fn as_str(&self) -> &'static str {
        match self {
            TraceLevel::Error => "error",
            TraceLevel::Warn  => "warn",
            TraceLevel::Info  => "info",
            TraceLevel::Debug => "debug",
            TraceLevel::Trace => "trace",
        }
    }
}

/// Initializes the tracing subscriber for logging.
///
/// Configures the logging level and output destination (stderr or a file).
pub fn init(trace_level : &TraceLevel, trace_output : &String) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(trace_level.as_str()));

    let fmt_layer = fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .without_time()
        .with_ansi(color::color_enabled().stderr);

    if trace_output == "-" {
        tracing_subscriber::registry()
            .with(fmt_layer.with_writer(std::io::stderr))
            .with(filter)
            .init();
    } else {
        let file = File::create(trace_output)
            .unwrap_or_else(|e| panic!("failed to create trace output file '{}': {}", trace_output, e));
        tracing_subscriber::registry()
            .with(fmt_layer.with_writer(file))
            .with(filter)
            .init();
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::color::{self, ColorOption};
    use ctor::ctor;

    #[ctor]
    fn test_init() {
        color::init(&ColorOption::Auto);
        init(&TraceLevel::Info, &"-".to_string());
    }
}
