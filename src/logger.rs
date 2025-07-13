use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use clap::{ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
pub enum TraceLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl TraceLevel {
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

pub fn init(trace_level : &TraceLevel) {
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                // Display the source code location of the log message.
                .with_file(true)
                .with_line_number(true)
                // Do not display the target (module path) of the log message.
                .with_target(false)
        )
        .with(
            // Filter logs based on the RUST_LOG env var, or info level by default.
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(trace_level.as_str())),
        )
        .init();
}
