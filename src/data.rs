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

impl TimeData {
    pub fn new(dir_path: &str, start: Option<NaiveDate>, end: Option<NaiveDate>) -> Result<Self, std::io::Error> {
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
                        match parse_entry(line) {
                            Ok(entry) => {
                                if (start.is_none() || date >= start.unwrap())
                                    && (end.is_none() || date <= end.unwrap())
                                {
                                    entries.entry(date).or_insert_with(Vec::new).push(entry);
                                }
                            }
                            Err(err) => {
                                eprintln!(
                                    "{}:{}: {}\n\t{}",
                                    file_path.display(),
                                    line_number + 1,
                                    err,
                                    line
                                );
                            }
                        }
                    } else {
                        eprintln!(
                            "{}:{}: Expected date, found:\n\t{}",
                            file_path.display(),
                            line_number + 1,
                            line
                        );
                    }
                }
            }
        }
        Ok(TimeData { entries })
    }
}

fn parse_date(line: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(line, "%Y.%m.%d")
        .or_else(|_| NaiveDate::parse_from_str(line, "%Y%m%d"))
        .or_else(|_| NaiveDate::parse_from_str(line, "%Y-%m-%d"))
        .ok()
}

fn parse_time_spec(time_spec: &str) -> Result<f32, String> {
    let time_spec = time_spec.trim();
    if time_spec.ends_with('h') {
        let hours_str = time_spec.trim_end_matches('h');
        let hours = hours_str
            .parse::<f32>()
            .map_err(|_| "Invalid hour format".to_string())?;
        if hours >= 0.0 {
            Ok(hours)
        } else {
            Err("Negative hours are invalid".to_string())
        }
    } else if time_spec.contains('-') {
        let parts: Vec<&str> = time_spec.split('-').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err("Time range must have exactly two parts".to_string());
        }
        let start_str = parts[0];
        let end_str = parts[1];

        // Normalize: append ":00" if no minutes are specified
        let start_str = if start_str.contains(':') {
            start_str.to_string()
        } else {
            format!("{}:00", start_str)
        };
        let end_str = if end_str.contains(':') {
            end_str.to_string()
        } else {
            format!("{}:00", end_str)
        };

        // Parse as NaiveTime with "%H:%M"
        let start = NaiveTime::parse_from_str(&start_str, "%H:%M")
            .map_err(|_| "Invalid start time".to_string())?;
        let end = NaiveTime::parse_from_str(&end_str, "%H:%M")
            .map_err(|_| "Invalid end time".to_string())?;

        let duration = end.signed_duration_since(start);
        if duration.num_minutes() < 0 {
            return Err("End time before start time".to_string());
        }
        let hours = duration.num_minutes() as f32 / 60.0;
        Ok(hours)
    } else {
        Err("Invalid time specification format".to_string())
    }
}

fn parse_entry(line: &str) -> Result<Entry, String> {
    let parts: Vec<&str> = line.splitn(2, '=').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return Err("Entry must have exactly two parts: time specs and description".to_string());
    }
    let left = parts[0];
    let description = parts[1].to_string();
    let time_specs: Vec<&str> = left.split(',').map(|s| s.trim()).collect();
    if time_specs.is_empty() {
        return Err("No time specifications provided".to_string());
    }
    let mut total_hours = 0.0;
    for time_spec in time_specs {
        let hours = parse_time_spec(time_spec)?;
        if hours < 0.0 {
            return Err("Negative hours are invalid".to_string());
        }
        total_hours += hours;
    }
    Ok(Entry {
        hours: total_hours,
        description,
    })
}