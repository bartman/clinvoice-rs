use chrono::{NaiveDate, NaiveTime};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct Entry {
    pub hours: f32,
    pub description: String,
}

#[derive(Debug)]
pub struct TimeData {
    pub entries: HashMap<NaiveDate, Vec<Entry>>,
}

enum ParseState {
    ExpectingDate,
    ExpectingEntry,
}

impl TimeData {
    /// Creates a new TimeData instance by loading entries from the specified directory.
    /// Only processes files with the '.cli' extension.
    /// If start and/or end dates are provided, only entries within that range are loaded.
    /// If none are provided, all entries are loaded.
    /// Prints error messages for parsing failures to stderr.
    pub fn new(dir_path: &str, start: Option<NaiveDate>, end: Option<NaiveDate>) -> Result<Self, std::io::Error> {
        let mut entries = HashMap::new();
        let path = Path::new(dir_path);

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.is_file() && file_path.extension().and_then(|s| s.to_str()) == Some("cli") {
                let content = fs::read_to_string(&file_path)?;
                let mut current_date: Option<NaiveDate> = None;
                let mut state = ParseState::ExpectingDate;

                for (line_number, line) in content.lines().enumerate() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    match state {
                        ParseState::ExpectingDate => {
                            if let Some(date) = parse_date(line) {
                                current_date = Some(date);
                                state = ParseState::ExpectingEntry;
                            } else {
                                eprintln!(
                                    "{}:{}: Expected date in YYYY.MM.DD or YYYYMMDD format, but found '{}'",
                                    file_path.display(),
                                    line_number + 1,
                                    line
                                );
                            }
                        }
                        ParseState::ExpectingEntry => {
                            if let Some(entry) = parse_entry(line) {
                                if let Some(date) = current_date {
                                    if (start.is_none() || date >= start.unwrap())
                                        && (end.is_none() || date <= end.unwrap())
                                    {
                                        entries.entry(date).or_insert_with(Vec::new).push(entry);
                                    }
                                }
                            } else if let Some(date) = parse_date(line) {
                                current_date = Some(date);
                                // State remains ExpectingEntry
                            } else {
                                eprintln!(
                                    "{}:{}: Expected entry in 'Xh = description' or 'HH:MM-HH:MM = description' format, but found '{}'",
                                    file_path.display(),
                                    line_number + 1,
                                    line
                                );
                            }
                        }
                    }
                }
            }
        }
        Ok(TimeData { entries })
    }
}

/// Parses a date string in either YYYY.MM.DD or YYYYMMDD format.
fn parse_date(line: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(line, "%Y.%m.%d")
        .or_else(|_| NaiveDate::parse_from_str(line, "%Y%m%d"))
        .ok()
}

/// Parses an entry line into an Entry struct.
/// Supports "Xh = description" or "HH:MM-HH:MM = description" formats.
fn parse_entry(line: &str) -> Option<Entry> {
    let parts: Vec<&str> = line.splitn(2, '=').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return None;
    }
    let left = parts[0];
    let description = parts[1].to_string();

    if left.contains('-') {
        let times: Vec<&str> = left.split('-').collect();
        if times.len() != 2 {
            return None;
        }
        let start = NaiveTime::parse_from_str(times[0], "%H:%M").ok()?;
        let end = NaiveTime::parse_from_str(times[1], "%H:%M").ok()?;
        let duration = end.signed_duration_since(start);
        let hours = duration.num_minutes() as f32 / 60.0;
        Some(Entry { hours, description })
    } else {
        let hours_str = left.trim_end_matches('h');
        let hours = hours_str.parse::<f32>().ok()?;
        Some(Entry { hours, description })
    }
}