use crate::ColorOption;
use crate::config::Config;
use crate::data::{DateSelector, TimeData};
use crate::parse::parse_date_arg;
use chrono::{Datelike, Local, NaiveDate};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tera::{Context, Tera, to_value, try_get_value, Value};

#[derive(Serialize)]
struct Day {
    index: usize,
    date: String,
    hours: f32,
    cost: f64,
    description: String,
}

fn date_filter(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = try_get_value!("date_filter", "value", String, value);
    let format = match args.get("format") {
        Some(val) => try_get_value!("date_filter", "format", String, val),
        None => "%Y-%m-%d".to_string(),
    };
    let date = NaiveDate::parse_from_str(&s, "%Y-%m-%d").unwrap();
    Ok(to_value(date.format(&format).to_string()).unwrap())
}

pub fn run(
    output_option: Option<String>,
    generator_option: &Option<String>,
    sequence_option: &Option<u32>,
    directory: &Option<String>,
    config_file: &Option<String>,
    _color: &ColorOption,
    dates: &[String],
) {
    let config = Config::new(config_file.as_deref(), directory.as_deref())
        .expect("Failed to load config");

    let use_generator = if let Some(selected) = generator_option {
        selected.clone()
    } else {
        config.get_string("generator.default").expect("generator.default is not defined in config")
    };

    let generator_prefix = format!("generator.{}", use_generator);
    let mut template_path = config
        .get_string(&format!("{}.template", generator_prefix))
        .expect("template not specified in config");

    if let Some(dir) = directory {
        let path = Path::new(dir).join(template_path.clone());
        template_path = path.to_str().unwrap().to_string();
    }

    let mut selector = DateSelector::new();
    for date_arg in dates {
        match parse_date_arg(date_arg) {
            Ok(range) => selector.add_range(range),
            Err(err) => {
                eprintln!("Invalid date argument: {} - {}", date_arg, err);
                std::process::exit(1);
            }
        }
    }

    let dir_path = directory.as_ref().expect("Directory path is required");
    let time_data = TimeData::new(dir_path, &selector, false).expect("Failed to load data");

    let mut context = Context::new();

    let sequence = if let Some(selected) = sequence_option {
        *selected
    } else {
        0
    };
    context.insert("sequence", &sequence);

    let flat_config_table = config.get_flattened_values();
    for (key, value) in flat_config_table.iter() {
        context.insert(key, value);
    }

    let mut days = Vec::new();
    let mut subtotal_amount = 0.0;
    let rate = config.get_f64("rate").unwrap_or(0.0);

    let mut sorted_dates: Vec<_> = time_data.entries.keys().collect();
    sorted_dates.sort();

    let now = Local::now();
    let today = now.date_naive();
    let invoice_date = today;
    let due_date = today + chrono::Duration::days(config.get_i64("contract.payment-days").unwrap_or(30));
    let period_start = sorted_dates.first().map(|d| *d).unwrap_or(&today);
    let period_end = sorted_dates.last().map(|d| *d).unwrap_or(&today);

    context.insert("now", &now.to_rfc3339());
    context.insert("today", &today.format("%Y-%m-%d").to_string());
    context.insert("invoice_date", &invoice_date.format("%Y-%m-%d").to_string());
    context.insert("due_date", &due_date.format("%Y-%m-%d").to_string());
    context.insert("period_start", &period_start.format("%Y-%m-%d").to_string());
    context.insert("period_end", &period_end.format("%Y-%m-%d").to_string());

    for (index, date) in sorted_dates.iter().enumerate() {
        let entries = &time_data.entries[date];
        let total_hours: f32 = entries.iter().map(|e| e.hours).sum();
        let cost = total_hours as f64 * rate;
        subtotal_amount += cost;

        let descriptions: Vec<_> = entries.iter().map(|e| e.description.as_str()).collect();
        days.push(Day {
            index: index + 1,
            date: date.format("%Y-%m-%d").to_string(),
            hours: total_hours,
            cost,
            description: descriptions.join("; "),
        });
    }

    context.insert("days", &days);
    context.insert("subtotal_amount", &subtotal_amount);

    let tax_rate = config.get_f64("tax_rate").unwrap_or(0.0);
    let tax_amount = subtotal_amount * tax_rate;
    let total_amount = subtotal_amount + tax_amount;

    context.insert("tax_amount", &tax_amount);
    context.insert("total_amount", &total_amount);

    let total_hours: f32 = time_data.entries.values().flat_map(|entries| entries.iter().map(|e| e.hours)).sum();
    context.insert("total_hours", &total_hours);

    let mut tera = Tera::default();
    tera.register_filter("date", date_filter);
    let template_content = fs::read_to_string(&template_path).expect("Unable to read template file");
    tera.add_raw_template("invoice", &template_content)
        .expect("Failed to add template");

    let output_path = match output_option {
        Some(path) => path,
        None => {
            let output_template = config
                .get_string(&format!("{}.output", generator_prefix))
                .expect("output not specified in config");
            let mut output_tera = Tera::default();
            output_tera
                .add_raw_template("output_path", &output_template)
                .unwrap();
            if let Some(last_date) = sorted_dates.last() {
                let mut output_context = Context::new();
                output_context.insert("year", &last_date.year());
                output_context.insert("month", &last_date.month());
                output_context.insert("day", &last_date.day());
                output_tera.render("output_path", &output_context).unwrap()
            } else {
                "invoice.out".to_string()
            }
        }
    };

    let rendered = tera.render("invoice", &context).expect("Failed to render template");

    let mut file = File::create(output_path).expect("Failed to create output file");
    file.write_all(rendered.as_bytes())
        .expect("Failed to write to output file");
}
