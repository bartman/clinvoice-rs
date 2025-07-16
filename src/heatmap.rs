use std::collections::HashMap;
use chrono::{Datelike, NaiveDate, Weekday, Month};
use crate::data::{DateRange, DateSelector, TimeData, Entry};
use num_traits::FromPrimitive;

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
    let mut first_monday = start_date;
    while first_monday.weekday() != Weekday::Mon {
        first_monday = first_monday.pred_opt().unwrap();
    }

    let mut weeks: Vec<Vec<Option<f64>>> = Vec::new();
    let mut week_dates: Vec<NaiveDate> = Vec::new();
    let mut current_week = vec![None; 7];
    let mut current_date = first_monday;

    while current_date <= end_date {
        let day_of_week = current_date.weekday().num_days_from_monday() as usize;
        if day_of_week == 0 {
            week_dates.push(current_date);
        }
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

    let terminal_width = if let Some((w, _)) = term_size::dimensions() {
        w
    } else {
        80 // Default width
    };
    let max_weeks = (terminal_width - 5) / 3;

    if weeks.len() > max_weeks {
        let start_index = weeks.len() - max_weeks;
        weeks = weeks.into_iter().skip(start_index).collect();
        week_dates = week_dates.into_iter().skip(start_index).collect();
    }

    // Print header
    print!("     ");
    for date in &week_dates {
        print!("{:2} ", date.day());
    }
    println!();

    // Print body
    for day_of_week in 0..7 {
        let day_label = match day_of_week {
            0 => "Mon ",
            2 => "Wed ",
            4 => "Fri ",
            6 => "Sun ",
            _ => "    ",
        };
        print!("{}", day_label);

        for (week_index, week) in weeks.iter().enumerate() {
            let cell = week[day_of_week];
            let current_day = week_dates[week_index] + chrono::Duration::days(day_of_week as i64);

            if current_day < start_date || current_day > end_date {
                print!("   ");
            } else {
                let (r, g, b) = match cell {
                    Some(hours) if hours > 0.0 => {
                        let intensity = (hours / max_hours * 230.0) as u8 + 25;
                        (0, intensity, 0)
                    }
                    _ => (20, 20, 20),
                };
                print!("\u{1b}[38;2;{};{};{}m ◀▶\u{1b}[0m", r, g, b);
            }
        }
        println!();
    }

    // Print footer
    print!("     ");
    let mut last_month = 0;
    for date in &week_dates {
        if date.month() != last_month {
            print!("{:3}", Month::from_u32(date.month()).unwrap().name()[..3].to_string());
            last_month = date.month();
        } else {
            print!("   ");
        }
    }
    println!();
}
