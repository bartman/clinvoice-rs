use crate::data::{TimeData, DateSelector};
use crate::parse::{parse_date_arg};
use crate::color::*;
use crate::LogFormat;
use chrono::Datelike;
use std::collections::HashMap;
use colored::Color;

pub fn run(
    format: LogFormat,
    directory_option: &Option<String>,
    dates: &[String],
) {
    let directory = directory_option.as_deref().unwrap_or(".");

    let mut selector = DateSelector::new();
    for date_arg in dates {
        match parse_date_arg(date_arg) {
            Ok(range) => selector.add_range(range),
            Err(err) => {
                tracing::error!("Invalid date argument: {} - {}", date_arg, err);
                std::process::exit(1);
            }
        }
    }

    let time_data = TimeData::new(directory, &selector).expect("Failed to load data");

    let mut grand_total: f32 = 0.0;
    let grand_total_indent;
    match format {
        LogFormat::Full => {
            let mut dates: Vec<_> = time_data.entries.keys().collect();
            dates.sort();
            for date in dates {
                for entry in &time_data.entries[date] {
                    let date_str = format!("{:04}.{:02}.{:02}", date.year(), date.month(), date.day());
                    let hours_str = format!("{:8.2}", entry.hours);
                    println!(
                        "{}  {}  {}",
                        date_str.out_colored(Color::Blue),
                        hours_str.out_colored(Color::Green),
                        entry.description
                    );
                    grand_total = grand_total + entry.hours;
                }
            }
            grand_total_indent = 12;
        }
        LogFormat::Day => {
            let mut dates: Vec<_> = time_data.entries.keys().collect();
            dates.sort();
            for date in dates {
                let entries = &time_data.entries[date];
                let total_hours: f32 = entries.iter().map(|e| e.hours).sum();
                let descriptions: Vec<_> = entries.iter().map(|e| e.description.as_str()).collect();
                let desc_str = descriptions.join("; ");
                let date_str = format!("{:04}.{:02}.{:02}", date.year(), date.month(), date.day());
                let hours_str = format!("{:8.2}", total_hours);
                println!(
                    "{}  {}  {}",
                    date_str.out_colored(Color::Blue),
                    hours_str.out_colored(Color::Green),
                    desc_str
                );
                grand_total = grand_total + total_hours;
            }
            grand_total_indent = 12;
        }
        LogFormat::Month => {
            let mut monthly_totals: HashMap<(i32, u32), f32> = HashMap::new();
            let mut monthly_counts: HashMap<(i32, u32), u64> = HashMap::new();
            for (date, entries) in &time_data.entries {
                let year = date.year();
                let month = date.month();
                let day = date.day();
                let total: f32 = entries.iter().map(|e| e.hours).sum();
                *monthly_totals.entry((year, month)).or_insert(0.0) += total;
                *monthly_counts.entry((year, month)).or_insert(0) |= 1u64 << day;
                grand_total = grand_total + total;
            }

            let mut months: Vec<_> = monthly_totals.keys().collect();
            months.sort();
            for (year, month) in months {
                let total_hours = monthly_totals[&(*year, *month)];
                let day_mask = monthly_counts[&(*year, *month)];
                let day_count = day_mask.count_ones();
                let date_str = format!("{:04}.{:02}", year, month);
                let hours_str = format!("{:8.2}", total_hours);
                let count_str = format!("{} day{}", day_count, match day_count { 1 => "", _ => "s" });
                println!(
                    "{}  {}  ({})",
                    date_str.out_colored(Color::Blue),
                    hours_str.out_colored(Color::Green),
                    count_str.out_colored(Color::Yellow),
                );
            }
            grand_total_indent = 9;
        }
        LogFormat::Year => {
            let mut yearly_totals: HashMap<i32, f32> = HashMap::new();
            let mut monthly_counts: HashMap<(i32, u32), u64> = HashMap::new();
            for (date, entries) in &time_data.entries {
                let year = date.year();
                let month = date.month();
                let day = date.day();
                let total: f32 = entries.iter().map(|e| e.hours).sum();
                *yearly_totals.entry(year).or_insert(0.0) += total;
                *monthly_counts.entry((year, month)).or_insert(0) |= 1u64 << day;
                grand_total = grand_total + total;
            }

            let mut yearly_counts: HashMap<i32, u32> = HashMap::new();
            let months: Vec<_> = monthly_counts.keys().collect();
            for (year, month) in months {
                let day_mask = monthly_counts[&(*year, *month)];
                let day_count = day_mask.count_ones();
                *yearly_counts.entry(*year).or_insert(0) += day_count;
            }

            let mut years: Vec<_> = yearly_totals.keys().collect();
            years.sort();
            for year in years {
                let total_hours = yearly_totals[year];
                let day_count = yearly_counts[year];
                let year_str = format!("{:04}", year);
                let hours_str = format!("{:8.2}", total_hours);
                let count_str = format!("{} day{}", day_count, match day_count { 1 => "", _ => "s" });
                println!(
                    "{}  {}  ({})",
                    year_str.out_colored(Color::Blue),
                    hours_str.out_colored(Color::Green),
                    count_str.out_colored(Color::Yellow),
                );
            }
            grand_total_indent = 6;
        }
    }
    let grand_total_str = format!("{:8.2}", grand_total);
    println!("{:<width$}{}",
        "Total:".out_colored(Color::Red),
        grand_total_str.out_colored(Color::Green),
        width = grand_total_indent);
}
