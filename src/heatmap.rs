use std::collections::HashMap;
use chrono::{Datelike, NaiveDate, Weekday};
use crate::data::{DateRange, DateSelector, TimeData, Entry};

/// Runs the heatmap generation.
pub fn run(directory: &Option<String>, dates: &[String]) {
    let date_selector = DateSelector::from_dates(dates).unwrap_or_else(|err| {
        tracing::error!("{}", err);
        std::process::exit(1);
    });
    let dir_path = directory.as_deref().unwrap_or(".");
    let time_data = TimeData::new(dir_path, &date_selector).unwrap_or_else(|err| {
        tracing::error!("Failed to load data: {}", err);
        std::process::exit(1);
    });
    let daily_hours = get_daily_hours(&time_data, &date_selector.ranges);
    if !daily_hours.is_empty() {
        let (start_date, end_date) = get_date_range(&daily_hours);
        let max_hours = get_max_hours(&daily_hours);
        draw_heatmap(daily_hours, start_date, end_date, max_hours);
    }
}

/// Calculates the total hours worked per day.
fn get_daily_hours(time_data: &TimeData, date_ranges: &[DateRange]) -> HashMap<NaiveDate, f64> {
    let mut daily_hours: HashMap<NaiveDate, f64> = HashMap::new();
    for (date, entries) in &time_data.entries {
        if date_ranges.is_empty() || date_ranges.iter().any(|dr| dr.start <= *date && dr.end >= *date) {
            for entry in entries {
                if let Entry::Time(hours, _) = entry {
                    *daily_hours.entry(*date).or_insert(0.0) += *hours as f64;
                }
            }
        }
    }
    daily_hours
}

/// Determines the start and end dates from the collected data.
fn get_date_range(daily_hours: &HashMap<NaiveDate, f64>) -> (NaiveDate, NaiveDate) {
    let mut dates: Vec<NaiveDate> = daily_hours.keys().cloned().collect();
    dates.sort();
    (*dates.first().unwrap(), *dates.last().unwrap())
}

/// Finds the maximum hours worked in a single day.
fn get_max_hours(daily_hours: &HashMap<NaiveDate, f64>) -> f64 {
    daily_hours.values().cloned().fold(0.0, f64::max)
}

/// Draws the heatmap to the console.
fn draw_heatmap(
    daily_hours: HashMap<NaiveDate, f64>,
    start_date: NaiveDate,
    end_date: NaiveDate,
    max_hours: f64,
) {
    let mut current_date = start_date;
    while current_date.weekday() != Weekday::Mon {
        current_date = current_date.pred_opt().unwrap();
    }

    let mut weeks: Vec<Vec<Option<f64>>> = Vec::new();
    let mut current_week = vec![None; 7];

    while current_date <= end_date {
        let day_of_week = current_date.weekday().num_days_from_monday() as usize;
        current_week[day_of_week] = daily_hours.get(&current_date).cloned();

        if current_date.weekday() == Weekday::Sun {
            weeks.push(current_week);
            current_week = vec![None; 7];
        }
        current_date = current_date.succ_opt().unwrap();
    }
    if !current_week.iter().all(Option::is_none) {
        weeks.push(current_week);
    }

    for day_of_week in 0..7 {
        for week in &weeks {
            let cell = week[day_of_week];
            let (r, g, b) = match cell {
                Some(hours) if hours > 0.0 => {
                    let intensity = (hours / max_hours * 230.0) as u8 + 25;
                    (0, intensity, 0)
                }
                _ => (20, 20, 20),
            };
            print!("\u{1b}[38;2;{};{};{}m ◀▶\u{1b}[0m", r, g, b);
        }
        println!();
    }
}
