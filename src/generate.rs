use crate::config::Config;
use crate::data::{DateSelector, TimeData};
use crate::latex::latex_escape;
use crate::markdown::markdown_escape;

use crate::color::*;
use crate::index::Index;
use chrono::{Local, NaiveDate};
use colored::Color;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::error::Error;
use tera::{Context, Tera, to_value, try_get_value, Value};

/// A builder for creating Tera contexts, allowing for insertion of serializable data
/// and conditional escaping based on the output mode (e.g., LaTeX).
pub struct TeraContextBuilder {
    data: HashMap<String, Value>,
}

impl Default for TeraContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TeraContextBuilder {
    /// Creates a new, empty `TeraContextBuilder`.
    pub fn new() -> Self {
        TeraContextBuilder {
            data: HashMap::new(),
        }
    }

    /// Inserts a serializable value into the context builder.
    /// The value is converted to a `tera::Value`.
    pub fn insert<T: Serialize + ?Sized>(&mut self, key: &str, value: &T) {
        self.data.insert(key.to_string(), to_value(value).unwrap());
    }

    /// Builds the Tera context from the accumulated data.
    /// Applies LaTeX escaping to string values if `escape_mode` is "latex".
    pub fn build(&self, escape_mode: &str) -> Context {
        let mut context = Context::new();
        for (key, value) in &self.data {
            if escape_mode == "latex" || escape_mode == "tex" {
                if let Some(s) = value.as_str() {
                    let e = latex_escape(s);
                    context.insert(key.as_str(), &e);
                } else {
                    context.insert(key.as_str(), value);
                }
            } else if escape_mode == "markdown" || escape_mode == "md" {
                if let Some(s) = value.as_str() {
                    let e = markdown_escape(s);
                    context.insert(key.as_str(), &e);
                } else {
                    context.insert(key.as_str(), value);
                }
            } else {
                context.insert(key.as_str(), value);
            }
        }
        context
    }
}

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

fn justify_string(value: &Value, args: &HashMap<String, Value>, alignment: &str) -> tera::Result<Value> {
    let s = if value.is_string() {
        value.as_str().unwrap().to_string()
    } else {
        value.to_string()
    };
    let width = try_get_value!("justify_string", "width", usize, args["width"]);

    let mut result = s;
    if result.len() > width {
        result.truncate(width);
    } else {
        match alignment {
            "left" => result = format!("{:<width$}", result, width = width),
            "right" => result = format!("{:>width$}", result, width = width),
            "center" => {
                let padding = width - result.len();
                let pad_left = padding / 2;
                let pad_right = padding - pad_left;
                result = format!("{: >pad_left$}{: <pad_right$}", result, "", pad_left = pad_left, pad_right = pad_right);
            }
            _ => (),
        }
    }
    Ok(to_value(result).unwrap())
}

// Tera filter for left justification.
fn left_filter(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    justify_string(value, args, "left")
}

// Tera filter for right justification.
fn right_filter(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    justify_string(value, args, "right")
}

// Tera filter for center justification.
fn center_filter(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    justify_string(value, args, "center")
}

// Tera filter to format numbers to a specified decimal precision.
fn decimal_filter(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    let num = try_get_value!("decimal_filter", "value", f64, value);
    let precision = try_get_value!("decimal_filter", "precision", usize, args["precision"]);
    Ok(to_value(format!("{:.precision$}", num, precision = precision)).unwrap())
}

fn format_tera_error(e: tera::Error) -> String {
    let mut error_message = String::new();
    error_message.push_str(&format!("{}\n", e));

    let mut current_error = e.source();
    while let Some(source) = current_error {
        error_message.push_str(&format!("{}\n", source));
        current_error = source.source();
    }

    error_message
}

/// Runs the invoice generation process.
///
/// This function orchestrates the entire invoice generation, including:
/// - Loading configuration.
/// - Managing invoice sequencing.
/// - Loading time data.
/// - Building the Tera context with all necessary data.
/// - Rendering the invoice template.
/// - Writing the output file.
/// - Executing a build command if specified in the configuration.
pub fn run(
    output_option: Option<String>,
    generator_option: &Option<String>,
    sequence_option: &Option<u32>,
    directory_option: &Option<String>,
    config_file: &Option<String>,
    dates: &[String],
) {
    let directory = directory_option.as_deref().unwrap_or(".");
    let config = Config::new(config_file.as_deref(), Some(directory))
        .expect("Failed to load config");
    let use_generator = if let Some(selected) = generator_option {
        selected.clone()
    } else {
        config.get_string("generator.default").expect("generator.default is not defined in config")
    };

    let generator_prefix = format!("generator.{}", use_generator);

    let index_file_name = config.get_string("index.file").unwrap_or(".index".to_string());
    let index_file_path = Path::new(directory).join(index_file_name);
    tracing::info!("Index file {}", index_file_path.display());
    let mut index = Index::new(&index_file_path).expect("Failed to open or lock index file");

    let sequence:u32 = if let Some(seq) = sequence_option {
        index.add_sequence(*seq, dates)
    } else {
        index.find_sequence(dates)
    };
    tracing::info!("Sequence is {}", sequence);
    let mut template_path = config
        .get_string(&format!("{}.template", generator_prefix))
        .expect("template not specified in config");
    let path = Path::new(directory).join(template_path.clone());
    template_path = path.to_str().unwrap().to_string();
    let template_name = path.file_name().unwrap().to_str().unwrap();

    let selector = DateSelector::from_dates(dates).unwrap_or_else(|err| {
        tracing::error!("{}", err);
        std::process::exit(1);
    });

    let time_data = TimeData::new(directory, &selector).expect("Failed to load data");

    let escape_mode = config.get_string(&format!("{}.escape", generator_prefix)).unwrap_or("none".to_string());
    tracing::info!("Escape mode {}", escape_mode);
    let mut context_builder = TeraContextBuilder::new();

    context_builder.insert("directory", directory);

    let flat_config_table = config.get_flattened_values("_");
    for (key, value) in flat_config_table.iter() {
        context_builder.insert(key, value);
    }
    context_builder.insert("sequence", &sequence);

    let flat_config_table = config.get_flattened_values("_");
    for (key, value) in flat_config_table.iter() {
        context_builder.insert(key, value);
        tracing::trace!("VAR  {:30}  =>  {}", key, *value);
    }

    let mut days = Vec::new();
    let mut total_hours_worked = 0.0f64;
    let mut total_hours_counted = 0.0f64;
    let mut total_fees = 0.0f64;
    let mut total_discounts = 0.0f64;
    let hourly_rate = config.get_f64("contract.hourly_rate").unwrap_or(0.0);

    let mut sorted_dates: Vec<_> = time_data.entries.keys().collect();
    sorted_dates.sort();

    let now = Local::now();
    let today = now.date_naive();
    let invoice_date = today;
    let due_date = today + chrono::Duration::days(config.get_i64("contract.payment_days").unwrap_or(30));
    let period_start = sorted_dates.first().copied().unwrap_or(&today);
    let period_end = sorted_dates.last().copied().unwrap_or(&today);

    context_builder.insert("now", &now.to_rfc3339());
    context_builder.insert("today", &today.format("%Y-%m-%d").to_string());
    context_builder.insert("invoice_date", &invoice_date.format("%Y-%m-%d").to_string());
    context_builder.insert("due_date", &due_date.format("%Y-%m-%d").to_string());
    context_builder.insert("period_start", &period_start.format("%Y-%m-%d").to_string());
    context_builder.insert("period_end", &period_end.format("%Y-%m-%d").to_string());

    let cap_hours_per_day = config.get_f64("contract.cap_hours_per_day").unwrap_or(0.0);
    let cap_hours_per_invoice = config.get_f64("contract.cap_hours_per_invoice").unwrap_or(0.0);

    for (index, date) in sorted_dates.iter().enumerate() {
        let entries = &time_data.entries[date];
        let mut total_hours = 0.0f64;
        let mut day_cost = 0.0f64;
        let mut descriptions = Vec::new();

        for entry in entries {
            match entry {
                crate::data::Entry::Time(h, d) => {
                    total_hours += *h as f64;
                    descriptions.push(d.clone());
                }
                crate::data::Entry::FixedCost(c, d) => {
                    let entry_cost = *c as f64;
                    descriptions.push(d.clone());
                    if entry_cost > 0.0 {
                        total_fees += entry_cost;
                    } else {
                        total_discounts += entry_cost;
                    }
                }
                crate::data::Entry::Note(n) => {
                    descriptions.push(n.clone());
                }
            }
        }

        let mut desc_text = descriptions.join("; ");

        total_hours_worked += total_hours;

        if cap_hours_per_day > 0.0 && total_hours > 0.0 && total_hours > cap_hours_per_day {
            desc_text.push_str(&format!(" ({} worked, {} billed)",
                total_hours, cap_hours_per_day));
            total_hours = cap_hours_per_day;
        }

        total_hours_counted += total_hours;

        day_cost += total_hours * hourly_rate;

        tracing::trace!("DAY  {} {:3}  {}", date, total_hours, day_cost);

        if escape_mode == "latex" {
            desc_text = latex_escape(&desc_text);
        } else if escape_mode == "markdown" || escape_mode == "md" {
            desc_text = markdown_escape(&desc_text);
        }

        days.push(Day {
            index: index + 1,
            date: date.format("%Y-%m-%d").to_string(),
            hours: total_hours as f32,
            cost: day_cost,
            description: desc_text,
        });
    }

    context_builder.insert("total_fixed_fees", &total_fees);
    context_builder.insert("total_discounts", &total_discounts);

    context_builder.insert("total_hours_worked", &total_hours_worked);
    context_builder.insert("total_hours_counted", &total_hours_counted);

    let counted_amount = total_hours_counted * hourly_rate;
    context_builder.insert("counted_amount", &counted_amount);

    let mut overage_hours = 0.0;
    let mut overage_discount = 0.0;
    if cap_hours_per_invoice > 0.0 && total_hours_counted > cap_hours_per_invoice  {
        overage_hours = total_hours_counted - cap_hours_per_invoice;
        overage_discount = - (overage_hours * hourly_rate);
    }

    context_builder.insert("overage_hours", &overage_hours);
    context_builder.insert("overage_discount", &overage_discount);

    let total_hours_billed = total_hours_counted - overage_hours;
    context_builder.insert("total_hours_billed", &total_hours_billed);

    let billed_amount = total_hours_billed * hourly_rate;
    context_builder.insert("billed_amount", &billed_amount);

    let subtotal_amount = billed_amount + total_fees + total_discounts;
    context_builder.insert("subtotal_amount", &subtotal_amount);

    let tax_percent = config.get_f64("tax.percent").unwrap_or(0.0);
    let tax_amount = subtotal_amount * tax_percent / 100.0;
    let total_amount = subtotal_amount + tax_amount;

    context_builder.insert("tax_amount", &tax_amount);
    context_builder.insert("total_amount", &total_amount);

    let total_hours: f32 = days.iter().map(|d| d.hours).sum();
    assert!(total_hours as f64 == total_hours_counted);
    context_builder.insert("total_hours", &total_hours);

    let mut tera = Tera::default();
    tera.register_filter("date", date_filter);
    tera.register_filter("left", left_filter);
    tera.register_filter("right", right_filter);
    tera.register_filter("center", center_filter);
    tera.register_filter("decimal", decimal_filter);

    let template_content = fs::read_to_string(&template_path).expect("Unable to read template file");
    if let Err(e) = tera.add_raw_template(template_name, &template_content) {
        tracing::error!("{}", format_tera_error(e));
        std::process::exit(1);
    }

    let output_path = match output_option {
        Some(path) => path,
        None => {
            let output_file_template_string = config
                .get_string(&format!("{}.output", generator_prefix))
                .expect("output not specified in config, use --output option.");

            let mut output_file_tera = Tera::default();
            if let Err(e) = output_file_tera
                .add_raw_template("output", &output_file_template_string) {
                tracing::error!("{}", format_tera_error(e));
                std::process::exit(1);
            }

            tracing::trace!("output template: {}", output_file_template_string);

            let rendered = match output_file_tera.render("output", &context_builder.build("none")) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("{}", format_tera_error(e));
                    std::process::exit(1);
                }
            };

            tracing::trace!("output filename: {}", rendered);

            let path = Path::new(directory).join(rendered.clone());
            path.to_str().unwrap().to_string()
        }
    };

    context_builder.insert("output", &output_path);

    let build_command_template_string = config.get_string(&format!("{}.build", generator_prefix));
    let build_command :Option<String> = match build_command_template_string {
        None => None,
        Some(cmd) => {
            let mut build_cmd_tera = Tera::default();
            if let Err(e) = build_cmd_tera
                .add_raw_template("build_command", &cmd) {
                tracing::error!("{}", format_tera_error(e));
                std::process::exit(1);
            }

            let rendered = match build_cmd_tera.render("build_command", &context_builder.build("none")) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("{}", format_tera_error(e));
                    std::process::exit(1);
                }
            };

            Some(rendered)
        },
    };

    // days are not made available to the output_path Tera context,
    // but must be available for the template processing.
    context_builder.insert("days", &days);

    let final_context = context_builder.build(&escape_mode);
    let rendered = match tera.render(template_name, &final_context) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("{}", format_tera_error(e));
            std::process::exit(1);
        }
    };

    if output_path == "-" {
        println!("{}", rendered);
        return;
    }

    tracing::info!("Generating {}", output_path);
    let mut file = File::create(&output_path).expect("Failed to create output file");
    file.write_all(rendered.as_bytes())
        .expect("Failed to write to output file");

    index.save().expect("Failed to save index file");

    if let Some(builder) = build_command {
        process_builder(builder);
    }
}

// Executes an external build command and streams its output.
fn process_builder(builder : String) {
    tracing::info!("Build with {}", builder.to_string());

    let mut cmd = Command::new("sh")
        .arg("-c")
        .arg(&builder)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute build command");

    let stdout = cmd.stdout.take().unwrap();
    let stderr = cmd.stderr.take().unwrap();

    let (tx, rx) = std::sync::mpsc::channel();

    let stdout_tx = tx.clone();
    let stdout_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            stdout_tx.send(line.unwrap()).unwrap();
        }
    });

    let stderr_tx = tx.clone();
    let stderr_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            stderr_tx.send(line.unwrap()).unwrap();
        }
    });

    drop(tx);

    let mut show_output = false;
    let success_show_lines = 2;
    let negative_show_lines = 5;
    let mut history: VecDeque<String> = VecDeque::with_capacity(negative_show_lines);
    let mut full_output = Vec::new();

    let negative_words = ["error", "fail", "fatal", "warn", "undefined", "missing"];

    for line in rx {
        full_output.push(line.clone());

        let line_has_negative = negative_words.iter().any(|w| line.to_lowercase().contains(w));

        if show_output {
            if line_has_negative {
                eprintln!("{}", line.colored(Color::BrightRed));
            } else {
                eprintln!("{}", line.colored(Color::BrightBlack));
            }

        } else if line_has_negative {
            show_output = true;
            for past_line in &history {
                eprintln!("{}", past_line.colored(Color::BrightBlack));
            }
            eprintln!("{}", line.colored(Color::BrightRed));
        } else {
            if history.len() == negative_show_lines {
                history.pop_front();
            }
            history.push_back(line);
        }
    }

    stdout_thread.join().unwrap();
    stderr_thread.join().unwrap();

    let status = cmd.wait().expect("Failed to wait for build command");

    if !status.success() {
        if !show_output {
            for line in full_output {
                eprintln!("{}", line.colored(Color::BrightBlack));
            }
        }
        tracing::error!("Build command failed with status: {:?}", status);
        std::process::exit(1);
    }

    let start_line = full_output.len().saturating_sub(success_show_lines);
    for line in &full_output[start_line..] {
        eprintln!("{}", line.colored(Color::BrightBlack));
    }

    tracing::info!("Build command successful");
}
