use crate::data::TimeData;
use crate::Format;
use crate::ColorOption;
use chrono::Datelike;
use std::collections::HashMap;
use colored::*;
use atty;

pub fn run(
    format: Format,
    directory: &Option<String>,
    _config: &Option<String>,
    color: &ColorOption
) {
    let dir_path = directory.as_ref().expect("Directory path is required");

    let use_color_stdout = match color {
        ColorOption::Always => true,
        ColorOption::Never => false,
        ColorOption::Auto => atty::is(atty::Stream::Stdout),
    };

    let use_color_stderr = match color {
        ColorOption::Always => true,
        ColorOption::Never => false,
        ColorOption::Auto => atty::is(atty::Stream::Stderr),
    };

    let time_data = TimeData::new(dir_path, None, None, use_color_stderr).expect("Failed to load data");

    match format {
        Format::Full => {
            let mut dates: Vec<_> = time_data.entries.keys().collect();
            dates.sort();
            for date in dates {
                for entry in &time_data.entries[date] {
                    let date_str = format!("{:04}.{:02}.{:02}", date.year(), date.month(), date.day());
                    let hours_str = format!("{:4}", entry.hours);
                    if use_color_stdout {
                        println!(
                            "{}  {}  {}",
                            date_str.blue().bold(),
                            hours_str.green().bold(),
                            entry.description
                        );
                    } else {
                        println!(
                            "{}  {}  {}",
                            date_str,
                            hours_str,
                            entry.description
                        );
                    }
                }
            }
        }
        Format::Day => {
            let mut dates: Vec<_> = time_data.entries.keys().collect();
            dates.sort();
            for date in dates {
                let entries = &time_data.entries[date];
                let total_hours: f32 = entries.iter().map(|e| e.hours).sum();
                let descriptions: Vec<_> = entries.iter().map(|e| e.description.as_str()).collect();
                let desc_str = descriptions.join("; ");
                let date_str = format!("{:04}.{:02}.{:02}", date.year(), date.month(), date.day());
                let hours_str = format!("{:4}", total_hours);
                if use_color_stdout {
                    println!(
                        "{}  {}  {}",
                        date_str.blue().bold(),
                        hours_str.green().bold(),
                        desc_str
                    );
                } else {
                    println!(
                        "{}  {}  {}",
                        date_str,
                        hours_str,
                        desc_str
                    );
                }
            }
        }
        Format::Month => {
            let mut monthly_totals: HashMap<(i32, u32), f32> = HashMap::new();
            for (date, entries) in &time_data.entries {
                let year = date.year();
                let month = date.month();
                let total: f32 = entries.iter().map(|e| e.hours).sum();
                *monthly_totals.entry((year, month)).or_insert(0.0) += total;
            }
            let mut months: Vec<_> = monthly_totals.keys().collect();
            months.sort();
            for (year, month) in months {
                let total = monthly_totals[&(*year, *month)];
                let date_str = format!("{:04}.{:02}", year, month);
                if use_color_stdout {
                    println!(
                        "{}  {}",
                        date_str.blue().bold(),
                        total
                    );
                } else {
                    println!(
                        "{}  {}",
                        date_str,
                        total
                    );
                }
            }
        }
        Format::Year => {
            let mut yearly_totals: HashMap<i32, f32> = HashMap::new();
            for (date, entries) in &time_data.entries {
                let year = date.year();
                let total: f32 = entries.iter().map(|e| e.hours).sum();
                *yearly_totals.entry(year).or_insert(0.0) += total;
            }
            let mut years: Vec<_> = yearly_totals.keys().collect();
            years.sort();
            for year in years {
                let total = yearly_totals[year];
                let year_str = format!("{:04}", year);
                if use_color_stdout {
                    println!(
                        "{}  {}",
                        year_str.blue().bold(),
                        total
                    );
                } else {
                    println!(
                        "{}  {}",
                        year_str,
                        total
                    );
                }
            }
        }
    }
}
