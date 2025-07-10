use crate::parse::{parse_date, parse_entry};
use chrono::{NaiveDate};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use colored::*;

#[derive(Debug)]
pub struct Entry {
    pub hours: f32,
    pub description: String,
}

#[derive(Debug)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

#[derive(Debug)]
pub struct DateSelector {
    pub ranges: Vec<DateRange>,
}

impl DateSelector {
    pub fn new() -> Self {
        DateSelector { ranges: Vec::new() }
    }

    pub fn add_range(&mut self, range: DateRange) {
        self.ranges.push(range);
    }

    pub fn selected(&self, date: &NaiveDate) -> bool {
        if self.ranges.is_empty() {
            true
        } else {
            self.ranges.iter().any(|range| date >= &range.start && date <= &range.end)
        }
    }
}

#[derive(Debug)]
pub struct TimeData {
    pub entries: HashMap<NaiveDate, Vec<Entry>>,
}

impl TimeData {
    pub fn new(dir_path: &str, selector: &DateSelector, use_color: bool) -> Result<Self, std::io::Error> {
        let mut entries = HashMap::new();
        let path = Path::new(dir_path);

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.is_file() && file_path.extension().and_then(|s| s.to_str()) == Some("cli") {
                let content = fs::read_to_string(&file_path)?;
                let mut current_date: Option<NaiveDate> = None;

                for (line_number, line) in content.lines().enumerate() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    if let Some(date) = parse_date(line) {
                        current_date = Some(date);
                    } else if let Some(date) = current_date {
                        if selector.selected(&date) {
                            match parse_entry(line) {
                                Ok(entry) => {
                                    entries.entry(date).or_insert_with(Vec::new).push(entry);
                                }
                                Err(err) => {
                                    let path_line = format!("{}:{}", file_path.display(), line_number + 1);
                                    if use_color {
                                        eprintln!(
                                            "{}: {}\n\t{}",
                                            path_line.yellow().bold(),
                                            err.red().bold(),
                                            line
                                        );
                                    } else {
                                        eprintln!(
                                            "{}: {}\n\t{}",
                                            path_line,
                                            err,
                                            line
                                        );
                                    }
                                }
                            }
                        }
                    } else {
                        let path_line = format!("{}:{}", file_path.display(), line_number + 1);
                        if use_color {
                            eprintln!(
                                "{}: Expected date, found:\n\t{}",
                                path_line.yellow().bold(),
                                line
                            );
                        } else {
                            eprintln!(
                                "{}: Expected date, found:\n\t{}",
                                path_line,
                                line
                            );
                        }
                    }
                }
            }
        }
        Ok(TimeData { entries })
    }
}
