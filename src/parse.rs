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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use crate::data::{DateRange, Entry};

    #[test]
    fn test_parse_date_valid_formats() {
        assert_eq!(parse_date("2023.01.15"), Some(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()));
        assert_eq!(parse_date("20230115"), Some(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()));
        assert_eq!(parse_date("2023-01-15"), Some(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()));
    }

    #[test]
    fn test_parse_date_invalid_formats() {
        assert_eq!(parse_date("2023/01/15"), None);
        assert_eq!(parse_date("invalid-date"), None);
        assert_eq!(parse_date(""), None);
    }

    #[test]
    fn test_parse_date_edge_cases() {
        assert_eq!(parse_date("2024.02.29"), Some(NaiveDate::from_ymd_opt(2024, 2, 29).unwrap())); // Leap year
        assert_eq!(parse_date("2023.02.29"), None); // Non-leap year
    }

    #[test]
    fn test_last_day_of_month_valid_months() {
        assert_eq!(last_day_of_month(2023, 1), NaiveDate::from_ymd_opt(2023, 1, 31).unwrap());
        assert_eq!(last_day_of_month(2023, 2), NaiveDate::from_ymd_opt(2023, 2, 28).unwrap());
        assert_eq!(last_day_of_month(2024, 2), NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()); // Leap year
        assert_eq!(last_day_of_month(2023, 12), NaiveDate::from_ymd_opt(2023, 12, 31).unwrap());
    }

    #[test]
    fn test_last_day_of_month_edge_cases() {
        assert_eq!(last_day_of_month(2023, 1), NaiveDate::from_ymd_opt(2023, 1, 31).unwrap());
        assert_eq!(last_day_of_month(2023, 12), NaiveDate::from_ymd_opt(2023, 12, 31).unwrap());
    }

    #[test]
    #[should_panic]
    fn test_last_day_of_month_invalid_month() {
        last_day_of_month(2023, 13); // Should panic due to unwrap on invalid date
    }

    #[test]
    fn test_parse_specifier_to_range_valid() {
        let range = parse_specifier_to_range("2023").unwrap();
        assert_eq!(range.start, NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
        assert_eq!(range.end, NaiveDate::from_ymd_opt(2023, 12, 31).unwrap());

        let range = parse_specifier_to_range("2023.03").unwrap();
        assert_eq!(range.start, NaiveDate::from_ymd_opt(2023, 3, 1).unwrap());
        assert_eq!(range.end, NaiveDate::from_ymd_opt(2023, 3, 31).unwrap());

        let range = parse_specifier_to_range("2023.02.15").unwrap();
        assert_eq!(range.start, NaiveDate::from_ymd_opt(2023, 2, 15).unwrap());
        assert_eq!(range.end, NaiveDate::from_ymd_opt(2023, 2, 15).unwrap());
    }

    #[test]
    fn test_parse_specifier_to_range_invalid() {
        assert!(parse_specifier_to_range("2023.13").is_err()); // Invalid month
        assert!(parse_specifier_to_range("2023.02.30").is_err()); // Invalid day
        assert!(parse_specifier_to_range("invalid").is_err());
        assert!(parse_specifier_to_range("2023.01.01.01").is_err()); // Too many parts
    }

    #[test]
    fn test_parse_specifier_to_range_edge_cases() {
        let range = parse_specifier_to_range("2024.02").unwrap(); // Leap year February
        assert_eq!(range.start, NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        assert_eq!(range.end, NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
    }

    #[test]
    fn test_parse_date_arg_valid() {
        let range = parse_date_arg("2023").unwrap();
        assert_eq!(range.start, NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
        assert_eq!(range.end, NaiveDate::from_ymd_opt(2023, 12, 31).unwrap());

        let range = parse_date_arg("2023.01-2023.03").unwrap();
        assert_eq!(range.start, NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
        assert_eq!(range.end, NaiveDate::from_ymd_opt(2023, 3, 31).unwrap());

        let range = parse_date_arg("2023.01.01-2023.01.01").unwrap();
        assert_eq!(range.start, NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
        assert_eq!(range.end, NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
    }

    #[test]
    fn test_parse_date_arg_invalid() {
        assert!(parse_date_arg("2023.03-2023.01").is_err()); // Start date after end date
        assert!(parse_date_arg("invalid-date").is_err());
        assert!(parse_date_arg("2023.01-invalid").is_err());
    }

    #[test]
    fn test_parse_date_arg_single_specifier() {
        let range = parse_date_arg("2023.05").unwrap();
        assert_eq!(range.start, NaiveDate::from_ymd_opt(2023, 5, 1).unwrap());
        assert_eq!(range.end, NaiveDate::from_ymd_opt(2023, 5, 31).unwrap());
    }

    #[test]
    fn test_parse_time_spec_valid_hours() {
        assert_eq!(parse_time_spec("8h").unwrap(), 8.0);
        assert_eq!(parse_time_spec("0.5h").unwrap(), 0.5);
        assert_eq!(parse_time_spec("10.0h").unwrap(), 10.0);
    }

    #[test]
    fn test_parse_time_spec_valid_range() {
        assert_eq!(parse_time_spec("9:00-17:00").unwrap(), 8.0);
        assert_eq!(parse_time_spec("9-17").unwrap(), 8.0);
        assert_eq!(parse_time_spec("9:30-10:00").unwrap(), 0.5);
        assert_eq!(parse_time_spec("17:00-9:00").unwrap_err(), "End time before start time".to_string());
    }

    #[test]
    fn test_parse_time_spec_invalid() {
        assert!(parse_time_spec("-5h").is_err()); // Negative hours
        assert!(parse_time_spec("invalid").is_err());
        assert!(parse_time_spec("9:00").is_err()); // Not a range or hours
        assert!(parse_time_spec("9:00-").is_err()); // Incomplete range
        assert!(parse_time_spec("-17:00").is_err()); // Incomplete range
    }

    #[test]
    fn test_parse_entry_valid() {
        let entry = parse_entry("8h = Development").unwrap();
        assert_eq!(entry.hours, 8.0);
        assert_eq!(entry.description, "Development");

        let entry = parse_entry("9:00-17:00 = Testing").unwrap();
        assert_eq!(entry.hours, 8.0);
        assert_eq!(entry.description, "Testing");

        let entry = parse_entry("2h, 3h = Meeting").unwrap();
        assert_eq!(entry.hours, 5.0);
        assert_eq!(entry.description, "Meeting");
    }

    #[test]
    fn test_parse_entry_invalid() {
        assert!(parse_entry("8h").is_err()); // Missing description
        assert!(parse_entry("= Description").is_err()); // Missing time spec
        assert!(parse_entry("invalid = Description").is_err()); // Invalid time spec
        assert!(parse_entry("").is_err()); // Empty string
        assert!(parse_entry("9:00-17:00, -1h = Invalid Entry").is_err()); // Negative hours in multiple specs
    }

    #[test]
    fn test_parse_entry_multiple_time_specs() {
        let entry = parse_entry("1h, 2h, 3h = Multiple Tasks").unwrap();
        assert_eq!(entry.hours, 6.0);
        assert_eq!(entry.description, "Multiple Tasks");

        let entry = parse_entry("9:00-10:00, 11:00-12:00 = Another Multiple").unwrap();
        assert_eq!(entry.hours, 2.0);
        assert_eq!(entry.description, "Another Multiple");
    }
}
