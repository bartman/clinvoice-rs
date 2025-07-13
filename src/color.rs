use std::sync::OnceLock;
use clap::ValueEnum;
use colored::*;

#[derive(ValueEnum, Clone, Debug)]
pub enum ColorOption {
    Always,
    Auto,
    Never,
}

#[derive(Clone, Debug)]
pub struct ColorEnable {
    pub stdout : bool,
    pub stderr : bool,
}

impl ColorEnable {
    pub fn new(color_option: &ColorOption) -> Self {
        let use_color_stdout = match color_option {
            ColorOption::Always => true,
            ColorOption::Never => false,
            ColorOption::Auto => atty::is(atty::Stream::Stdout),
        };

        let use_color_stderr = match color_option {
            ColorOption::Always => true,
            ColorOption::Never => false,
            ColorOption::Auto => atty::is(atty::Stream::Stderr),
        };

        ColorEnable { stdout: use_color_stdout, stderr: use_color_stderr }
    }
}

static G_COLOR_ENABLED: OnceLock<ColorEnable> = OnceLock::new();

pub fn init(color_option: &ColorOption) {
    G_COLOR_ENABLED.set(ColorEnable::new(color_option)).expect("init called multiple times");
}

pub fn color_enabled() -> ColorEnable {
    G_COLOR_ENABLED.get().expect("init not called").clone()
}

pub trait DynamicColorize {
    fn colored(&self, color: Color) -> ColoredString;
    fn out_colored(&self, color: Color) -> ColoredString;
    fn err_colored(&self, color: Color) -> ColoredString;
}

impl DynamicColorize for str {
    fn colored(&self, color: Color) -> ColoredString {
        match color {
            Color::Black => self.black(),
            Color::Red => self.red(),
            Color::Green => self.green(),
            Color::Yellow => self.yellow(),
            Color::Blue => self.blue(),
            Color::Magenta => self.magenta(),
            Color::Cyan => self.cyan(),
            Color::White => self.white(),
            Color::BrightBlack => self.bright_black(),
            Color::BrightRed => self.bright_red(),
            Color::BrightGreen => self.bright_green(),
            Color::BrightYellow => self.bright_yellow(),
            Color::BrightBlue => self.bright_blue(),
            Color::BrightMagenta => self.bright_magenta(),
            Color::BrightCyan => self.bright_cyan(),
            Color::BrightWhite => self.bright_white(),
            Color::TrueColor { r, g, b } => self.truecolor(r, g, b)
        }
    }
    fn out_colored(&self, color: Color) -> ColoredString {
        if color_enabled().stdout {
            self.colored(color)
        } else {
            self.normal()
        }
    }
    fn err_colored(&self, color: Color) -> ColoredString {
        if color_enabled().stderr {
            self.colored(color)
        } else {
            self.normal()
        }
    }
}

impl DynamicColorize for String {
    fn colored(&self, color: Color) -> ColoredString {
        self.as_str().colored(color)
    }
    fn out_colored(&self, color: Color) -> ColoredString {
        self.as_str().out_colored(color)
    }
    fn err_colored(&self, color: Color) -> ColoredString {
        self.as_str().err_colored(color)
    }
}
