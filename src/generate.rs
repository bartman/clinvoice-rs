use crate::ColorOption;
use crate::config::Config;
use crate::data::{DateSelector, TimeData};
use crate::parse::parse_date_arg;
use chrono::Datelike;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tera::{Context, Tera};
use toml::Value as TomlValue;

fn to_json(value: &TomlValue) -> JsonValue {
    match value {
        TomlValue::String(s) => JsonValue::String(s.clone()),
        TomlValue::Integer(i) => JsonValue::Number(serde_json::Number::from(*i)),
        TomlValue::Float(f) => JsonValue::Number(serde_json::Number::from_f64(*f).unwrap()),
        TomlValue::Boolean(b) => JsonValue::Bool(*b),
        TomlValue::Array(arr) => JsonValue::Array(arr.iter().map(to_json).collect()),
        TomlValue::Table(table) => {
            let mut map = serde_json::Map::new();
            for (k, v) in table {
                map.insert(k.clone(), to_json(v));
            }
            JsonValue::Object(map)
        }
        TomlValue::Datetime(dt) => JsonValue::String(dt.to_string()),
    }
}

pub fn run(
    output_option: Option<String>,
    generator_option: &Option<String>,
    _sequence_option: &Option<u32>,
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
    let config_table = config.as_table();
    for (key, value) in config_table.iter() {
        context.insert(key, &to_json(value));
    }

    let mut days = Vec::new();
    let mut subtotal_amount = 0.0;
    let rate = config.get_f64("rate").unwrap_or(0.0);

    let mut sorted_dates: Vec<_> = time_data.entries.keys().collect();
    sorted_dates.sort();

    for (index, date) in sorted_dates.iter().enumerate() {
        let entries = &time_data.entries[date];
        let total_hours: f32 = entries.iter().map(|e| e.hours).sum();
        let cost = total_hours as f64 * rate;
        subtotal_amount += cost;

        let mut day_data = HashMap::new();
        day_data.insert("index".to_string(), JsonValue::Number(serde_json::Number::from(index + 1)));
        day_data.insert("date".to_string(), JsonValue::String(date.format("%Y-%m-%d").to_string()));
        day_data.insert("hours".to_string(), JsonValue::Number(serde_json::Number::from_f64(total_hours as f64).unwrap()));
        day_data.insert("cost".to_string(), JsonValue::Number(serde_json::Number::from_f64(cost).unwrap()));
        days.push(day_data);
    }

    context.insert("days", &days);
    context.insert("subtotal_amount", &subtotal_amount);

    let tax_rate = config.get_f64("tax_rate").unwrap_or(0.0);
    let tax_amount = subtotal_amount * tax_rate;
    let total_amount = subtotal_amount + tax_amount;

    context.insert("tax_amount", &tax_amount);
    context.insert("total_amount", &total_amount);

    let mut tera = Tera::default();
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
