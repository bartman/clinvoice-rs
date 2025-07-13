use crate::config::Config;
use crate::data::{DateSelector, TimeData};
use crate::latex::latex_escape;
use crate::parse::parse_date_arg;
use crate::color::*;
use chrono::{Local, NaiveDate};
use colored::Color;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use tera::{Context, Tera, to_value, try_get_value, Value};

pub struct TeraContextBuilder {
    data: HashMap<String, Value>,
}

impl TeraContextBuilder {
    pub fn new() -> Self {
        TeraContextBuilder {
            data: HashMap::new(),
        }
    }

    pub fn insert<T: Serialize + ?Sized>(&mut self, key: &str, value: &T) {
        self.data.insert(key.to_string(), to_value(value).unwrap());
    }

    pub fn build(&self, escape_mode: &str) -> Context {
        let mut context = Context::new();
        for (key, value) in &self.data {
            if escape_mode == "latex" {
                if let Some(s) = value.as_str() {
                    let e = latex_escape(s);
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

fn left_filter(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    justify_string(value, args, "left")
}

fn right_filter(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    justify_string(value, args, "right")
}

fn center_filter(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    justify_string(value, args, "center")
}

fn decimal_filter(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    let num = try_get_value!("decimal_filter", "value", f64, value);
    let precision = try_get_value!("decimal_filter", "precision", usize, args["precision"]);
    Ok(to_value(format!("{:.precision$}", num, precision = precision)).unwrap())
}

pub fn run(
    output_option: Option<String>,
    generator_option: &Option<String>,
    sequence_option: &Option<u32>,
    directory_option: &Option<String>,
    config_file: &Option<String>,
    dates: &[String],
) {
    let config = Config::new(config_file.as_deref(), directory_option.as_deref())
        .expect("Failed to load config");
    let directory = directory_option.as_deref().unwrap_or(".");

    let use_generator = if let Some(selected) = generator_option {
        selected.clone()
    } else {
        config.get_string("generator.default").expect("generator.default is not defined in config")
    };

    let use_sequence:u32 = if let Some(selected) = sequence_option {
        *selected
    } else {
        1
    };

    let generator_prefix = format!("generator.{}", use_generator);
    let mut template_path = config
        .get_string(&format!("{}.template", generator_prefix))
        .expect("template not specified in config");
    let path = Path::new(directory).join(template_path.clone());
    template_path = path.to_str().unwrap().to_string();

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

    let escape_mode = config.get_string(&format!("{}.escape", generator_prefix)).unwrap_or("none".to_string());
    tracing::info!("Escape mode {}", escape_mode);
    let mut context_builder = TeraContextBuilder::new();

    context_builder.insert("directory", directory);

    let flat_config_table = config.get_flattened_values("_");
    for (key, value) in flat_config_table.iter() {
        context_builder.insert(key, value);
    }
    context_builder.insert("sequence", &use_sequence);

    let sequence = if let Some(selected) = sequence_option {
        *selected
    } else {
        0
    };
    context_builder.insert("sequence", &sequence);

    let flat_config_table = config.get_flattened_values("_");
    for (key, value) in flat_config_table.iter() {
        context_builder.insert(key, value);
        tracing::trace!("VAR  {:30}  =>  {}", key, *value);
    }

    let mut days = Vec::new();
        let mut subtotal_amount = 0.0f64;
    let hourly_rate = config.get_f64("contract.hourly_rate").unwrap_or(0.0);

    let mut sorted_dates: Vec<_> = time_data.entries.keys().collect();
    sorted_dates.sort();

    let now = Local::now();
    let today = now.date_naive();
    let invoice_date = today;
    let due_date = today + chrono::Duration::days(config.get_i64("contract.payment_days").unwrap_or(30));
    let period_start = sorted_dates.first().map(|d| *d).unwrap_or(&today);
    let period_end = sorted_dates.last().map(|d| *d).unwrap_or(&today);

    context_builder.insert("now", &now.to_rfc3339());
    context_builder.insert("today", &today.format("%Y-%m-%d").to_string());
    context_builder.insert("invoice_date", &invoice_date.format("%Y-%m-%d").to_string());
    context_builder.insert("due_date", &due_date.format("%Y-%m-%d").to_string());
    context_builder.insert("period_start", &period_start.format("%Y-%m-%d").to_string());
    context_builder.insert("period_end", &period_end.format("%Y-%m-%d").to_string());

    for (index, date) in sorted_dates.iter().enumerate() {
        let entries = &time_data.entries[date];
        let mut total_hours = 0.0;
                let mut cost = 0.0f64;
        let mut descriptions = Vec::new();

        for entry in entries {
            match entry {
                crate::data::Entry::Time(h, d) => {
                    total_hours += h;
                    descriptions.push(d.clone());
                }
                crate::data::Entry::FixedCost(c, d) => {
                    cost += *c as f64;
                    descriptions.push(d.clone());
                }
                crate::data::Entry::Note(n) => {
                    descriptions.push(n.clone());
                }
            }
        }

        cost += total_hours as f64 * hourly_rate;
        subtotal_amount += cost;

        tracing::trace!("DAY  {} {:3}  {}", date, total_hours, cost);

        let mut description = descriptions.join("; ");
        if escape_mode == "latex" {
            description = latex_escape(&description);
        }

        days.push(Day {
            index: index + 1,
            date: date.format("%Y-%m-%d").to_string(),
            hours: total_hours,
            cost: cost as f64,
            description: description,
        });
    }

    context_builder.insert("subtotal_amount", &subtotal_amount);

    let tax_percent = config.get_f64("tax.percent").unwrap_or(0.0);
    let tax_amount = subtotal_amount * tax_percent / 100.0;
    let total_amount = subtotal_amount + tax_amount;

    context_builder.insert("tax_amount", &tax_amount);
    context_builder.insert("total_amount", &total_amount);

    let total_hours: f32 = days.iter().map(|d| d.hours).sum();
    context_builder.insert("total_hours", &total_hours);

    let mut tera = Tera::default();
    tera.register_filter("date", date_filter);
    tera.register_filter("left", left_filter);
    tera.register_filter("right", right_filter);
    tera.register_filter("center", center_filter);
    tera.register_filter("decimal", decimal_filter);

    let template_content = fs::read_to_string(&template_path).expect("Unable to read template file");
    tera.add_raw_template("invoice", &template_content)
        .expect("Failed to add template");

    let output_path = match output_option {
        Some(path) => path,
        None => {
            let output_file_template_string = config
                .get_string(&format!("{}.output", generator_prefix))
                .expect("output not specified in config, use --output option.");

            let mut output_file_tera = Tera::default();
            output_file_tera
                .add_raw_template("output", &output_file_template_string)
                .unwrap();

            let rendered = output_file_tera.render("output", &context_builder.build("none"))
                .expect("Failed to render output filename");

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
            build_cmd_tera
                .add_raw_template("build_command", &cmd)
                .unwrap();

            let rendered = build_cmd_tera.render("build_command", &context_builder.build("none"))
                .expect("Failed to render build command");

            Some(rendered)
        },
    };

    // days are not made available to the output_path Tera context,
    // but must be available for the template processing.
    context_builder.insert("days", &days);

    let final_context = context_builder.build(&escape_mode);
    let rendered = tera.render("invoice", &final_context).expect("Failed to render template");

    if output_path == "-" {
        println!("{}", rendered);
        return;
    }

    tracing::info!("Generating {}", output_path);
    let mut file = File::create(output_path).expect("Failed to create output file");
    file.write_all(rendered.as_bytes())
        .expect("Failed to write to output file");

    if let Some(builder) = build_command {
        process_builder(builder);
    }
}

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

    let negative_words = vec!["error", "fail", "fatal", "warn", "undefined", "missing"];

    for line in rx {
        full_output.push(line.clone());

        let line_has_negative = negative_words.iter().any(|w| line.to_lowercase().contains(w));

        if show_output {
            if line_has_negative {
                eprintln!("{}", line.colored(Color::BrightRed));
            } else {
                eprintln!("{}", line.colored(Color::BrightBlack));
            }
        } else {
            if line_has_negative {
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