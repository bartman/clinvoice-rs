use crate::data::TimeData;
use crate::Format;
use chrono::Datelike; // Added to bring Datelike trait into scope
use std::collections::HashMap;

pub fn run(format: Format, directory: &Option<String>, _config: &Option<String>) {
    let dir_path = directory.as_ref().expect("Directory path is required");
    let time_data = TimeData::new(dir_path, None, None).expect("Failed to load data");

    match format {
        Format::Full => {
            let mut dates: Vec<_> = time_data.entries.keys().collect();
            dates.sort();
            for date in dates {
                for entry in &time_data.entries[date] {
                    println!(
                        "{:04}.{:02}.{:02}  {:4}  {}",
                        date.year(),
                        date.month(),
                        date.day(),
                        entry.hours,
                        entry.description
                    );
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
                println!(
                    "{:04}.{:02}.{:02}  {:4}  {}",
                    date.year(),
                    date.month(),
                    date.day(),
                    total_hours,
                    desc_str
                );
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
                println!("{:04}.{:02}  {}", year, month, total);
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
                println!("{:04}  {}", year, total);
            }
        }
    }
}
