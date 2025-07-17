use crate::parse::{parse_date, parse_line};
use crate::color::*;
use chrono::{NaiveDate};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use colored::Color;

/// Represents a single entry in a timesheet, which can be time worked, a fixed cost, or a note.
#[derive(Debug)]
pub enum Entry {
    Time(f32, String),
    FixedCost(f32, String),
    Note(String),
}

/// Represents a range of dates, inclusive of start and end dates.
#[derive(Debug)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

/// Selects and filters dates based on specified ranges.
#[derive(Debug)]
pub struct DateSelector {
    pub ranges: Vec<DateRange>,
}

impl Default for DateSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl DateSelector {
    /// Creates a new, empty `DateSelector`.
    pub fn new() -> Self {
        DateSelector { ranges: Vec::new() }
    }

    /// Creates a `DateSelector` from a list of date argument strings.
    ///
    /// Each string can represent a single date or a date range.
    ///
    /// # Arguments
    ///
    /// * `dates` - A slice of strings, where each string is a date argument.
    ///
    /// # Errors
    ///
    /// Returns a `String` error if any date argument is invalid.
    pub fn from_dates(dates: &[String]) -> Result<Self, String> {
        let mut selector = DateSelector::new();
        for date_arg in dates {
            match crate::parse::parse_date_arg(date_arg) {
                Ok(range) => selector.add_range(range),
                Err(err) => {
                    return Err(format!("Invalid date argument: {} - {}", date_arg, err));
                }
            }
        }
        Ok(selector)
    }

    /// Adds a `DateRange` to the selector.
    pub fn add_range(&mut self, range: DateRange) {
        self.ranges.push(range);
    }

    /// Checks if a given date falls within any of the selected date ranges.
    ///
    /// If no ranges are specified, all dates are considered selected.
    pub fn selected(&self, date: &NaiveDate) -> bool {
        if self.ranges.is_empty() {
            true
        } else {
            self.ranges.iter().any(|range| date >= &range.start && date <= &range.end)
        }
    }
}

/// Stores time entries organized by date.
#[derive(Debug)]
pub struct TimeData {
    pub entries: HashMap<NaiveDate, Vec<Entry>>,
}

impl TimeData {
    /// Creates a new `TimeData` instance by reading and parsing .cli files from a directory.
    ///
    /// Only entries within the dates specified by the `DateSelector` are included.
    ///
    /// # Arguments
    ///
    /// * `dir_path` - The path to the directory containing .cli files.
    /// * `selector` - A `DateSelector` to filter entries by date.
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if the directory cannot be read or files cannot be parsed.
    pub fn new(dir_path: &str, selector: &DateSelector) -> Result<Self, std::io::Error> {
        let mut entries = HashMap::new();
        let path = Path::new(dir_path);

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.is_file() && file_path.extension().and_then(|s| s.to_str()) == Some("cli") {
                tracing::trace!("FILE  {}", file_path.display());

                let content = fs::read_to_string(&file_path)?;
                let mut current_date: Option<NaiveDate> = None;

                for (line_number, line) in content.lines().enumerate() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                        continue;
                    }

                    tracing::trace!("LINE {}  {}", line_number+1, line);

                    if let Some(date) = parse_date(line) {
                        current_date = Some(date);
                    } else if let Some(date) = current_date {
                        if selector.selected(&date) {
                            match parse_line(line) {
                                Ok(entry) => {
                                    entries.entry(date).or_insert_with(Vec::new).push(entry);
                                }
                                Err(err) => {
                                    let path_line = format!("{}:{}", file_path.display(), line_number + 1);
                                    tracing::warn!("{}\n\t{}: {}",
                                        err.err_colored(Color::Yellow),
                                        path_line, line);
                                }
                            }
                        }
                    } else {
                        let path_line = format!("{}:{}", file_path.display(), line_number + 1);

                        let err = "Expected date, found:";
                        tracing::warn!("{}\n\t{}: {}",
                            err.err_colored(Color::Yellow),
                            path_line, line);
                    }
                }
            }
        }
        Ok(TimeData { entries })
    }
}
