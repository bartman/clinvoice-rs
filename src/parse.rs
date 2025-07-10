use crate::data::{DateRange, Entry};
use chrono::{NaiveDate, NaiveTime};

pub fn parse_date(line: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(line, "%Y.%m.%d")
        .or_else(|_| NaiveDate::parse_from_str(line, "%Y%m%d"))
        .or_else(|_| NaiveDate::parse_from_str(line, "%Y-%m-%d"))
        .ok()
}

pub fn last_day_of_month(year: i32, month: u32) -> NaiveDate {
    if month == 12 {
        NaiveDate::from_ymd_opt(year, 12, 31).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap().pred_opt().unwrap()
    }
}

pub fn parse_specifier_to_range(spec: &str) -> Result<DateRange, String> {
    let parts: Vec<&str> = spec.split('.').collect();
    match parts.len() {
        1 => {
            let year: i32 = parts[0].parse().map_err(|_| "Invalid year".to_string())?;
            let start = NaiveDate::from_ymd_opt(year, 1, 1).ok_or("Invalid date".to_string())?;
            let end = NaiveDate::from_ymd_opt(year, 12, 31).ok_or("Invalid date".to_string())?;
            Ok(DateRange { start, end })
        }
        2 => {
            let year: i32 = parts[0].parse().map_err(|_| "Invalid year".to_string())?;
            let month: u32 = parts[1].parse().map_err(|_| "Invalid month".to_string())?;
            if month < 1 || month > 12 {
                return Err("Month out of range".to_string());
            }
            let start = NaiveDate::from_ymd_opt(year, month, 1).ok_or("Invalid date".to_string())?;
            let end = last_day_of_month(year, month);
            Ok(DateRange { start, end })
        }
        3 => {
            let year: i32 = parts[0].parse().map_err(|_| "Invalid year".to_string())?;
            let month: u32 = parts[1].parse().map_err(|_| "Invalid month".to_string())?;
            let day: u32 = parts[2].parse().map_err(|_| "Invalid day".to_string())?;
            let date = NaiveDate::from_ymd_opt(year, month, day).ok_or("Invalid date".to_string())?;
            Ok(DateRange { start: date, end: date })
        }
        _ => Err("Invalid date specifier".to_string()),
    }
}

pub fn parse_date_arg(arg: &str) -> Result<DateRange, String> {
    if let Some((start_spec, end_spec)) = arg.split_once('-') {
        let start_range = parse_specifier_to_range(start_spec)?;
        let end_range = parse_specifier_to_range(end_spec)?;
        let start = start_range.start;
        let end = end_range.end;
        if start > end {
            return Err("Start date after end date".to_string());
        }
        Ok(DateRange { start, end })
    } else {
        parse_specifier_to_range(arg)
    }
}

pub fn parse_time_spec(time_spec: &str) -> Result<f32, String> {
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

pub fn parse_entry(line: &str) -> Result<Entry, String> {
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
