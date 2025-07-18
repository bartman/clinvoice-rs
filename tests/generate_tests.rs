//use clinvoice::config::Config;
//use clinvoice::data::DateSelector;
use clinvoice::generate;
//use clinvoice::parse::parse_date_arg;
use std::collections::HashMap;
use tempfile::tempdir;

// Helper function to create a temporary test environment
fn create_test_env(
    cli_contents: &HashMap<&str, &str>,
    config_content: &str,
) -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    tracing::trace!("tmpdir: {}", temp_dir.path().display());

    // Create .cli files
    for (filename, content) in cli_contents {
        std::fs::write(temp_dir.path().join(filename), content)?;
    }

    // Create clinvoice.toml
    std::fs::write(temp_dir.path().join("clinvoice.toml"), config_content)?;

    Ok(temp_dir)
}

#[test]
fn test_generate_basic_invoice() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli_contents = HashMap::new();
    cli_contents.insert(
        "timesheet.cli",
        r#"
2025.01.01
8h = Development
"#,
    );
    let config_content = r#"
[contract]
hourly_rate = 100.0

[generator.txt]
template = "template.txt"
output = "invoice.txt"
"#;
    let template_content = r#"
Total worked: {{ total_hours_worked  }}
Total counted: {{ total_hours_counted }} {{ counted_amount }}
Total billed: {{ total_hours_billed  }} {{ billed_amount }}

Fixed fees: {{ total_fixed_fees }} {{ total_discounts }}
Overage: {{ overage_hours }} {{ overage_discount }}

Total hours: {{ total_hours }}
Total amount: {{ total_amount }}
"#;

    let temp_dir = create_test_env(&cli_contents, config_content)?;
    std::fs::write(temp_dir.path().join("template.txt"), template_content)?;

    let output_path = temp_dir.path().join("invoice.txt");
    let directory_option = Some(temp_dir.path().to_str().unwrap().to_string());
    let config_file_option = Some(temp_dir.path().join("clinvoice.toml").to_str().unwrap().to_string());

    generate::run(
        Some(output_path.to_str().unwrap().to_string()),
        &Some("txt".to_string()),
        &None,
        &directory_option,
        &config_file_option,
        &[],
    );

    let generated_content = std::fs::read_to_string(&output_path)?;
    println!("{}", generated_content);

    assert!(generated_content.contains("Total hours: 8"));
    assert!(generated_content.contains("Total amount: 800"));

    assert!(generated_content.contains("Total worked: 8"));
    assert!(generated_content.contains("Total counted: 8 800"));
    assert!(generated_content.contains("Total billed: 8 800"));

    assert!(generated_content.contains("Fixed fees: 0 0"));
    assert!(generated_content.contains("Overage: 0 0"));

    Ok(())
}

#[test]
fn test_generate_with_date_selection() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli_contents = HashMap::new();
    cli_contents.insert(
        "timesheet1.cli",
        r#"
2025.01.01
8h = Project A
"#,
    );
    cli_contents.insert(
        "timesheet2.cli",
        r#"
2025.01.02
4h = Project B
"#,
    );
    cli_contents.insert(
        "timesheet3.cli",
        r#"
2025.02.01
6h = Project C
2025.02.02
$100 = fixed price (not included)
2025.02.03
-$50 = discount (not included)
"#,
    );
    let config_content = r#"
[contract]
hourly_rate = 100.0

[generator.txt]
template = "template.txt"
output = "invoice.txt"
"#;
    let template_content = r#"
Total worked: {{ total_hours_worked  }}
Total counted: {{ total_hours_counted }} {{ counted_amount }}
Total billed: {{ total_hours_billed  }} {{ billed_amount }}

Fixed fees: {{ total_fixed_fees }} {{ total_discounts }}
Overage: {{ overage_hours }} {{ overage_discount }}

Total hours: {{ total_hours }}
Total amount: {{ total_amount }}
"#;

    let temp_dir = create_test_env(&cli_contents, config_content)?;
    std::fs::write(temp_dir.path().join("template.txt"), template_content)?;

    let output_path = temp_dir.path().join("invoice.txt");
    let directory_option = Some(temp_dir.path().to_str().unwrap().to_string());
    let config_file_option = Some(temp_dir.path().join("clinvoice.toml").to_str().unwrap().to_string());

    generate::run(
        Some(output_path.to_str().unwrap().to_string()),
        &Some("txt".to_string()),
        &None,
        &directory_option,
        &config_file_option,
        &["2025.01".to_string()], // Select only January
    );

    let generated_content = std::fs::read_to_string(&output_path)?;
    println!("{}", generated_content);

    assert!(generated_content.contains("Total hours: 12")); // 8h + 4h
    assert!(generated_content.contains("Total amount: 1200"));

    assert!(generated_content.contains("Total worked: 12"));
    assert!(generated_content.contains("Total counted: 12 1200"));
    assert!(generated_content.contains("Total billed: 12 1200"));

    assert!(generated_content.contains("Fixed fees: 0 0"));
    assert!(generated_content.contains("Overage: 0 0"));

    Ok(())
}

#[test]
fn test_generate_with_mixed_entry_types() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli_contents = HashMap::new();
    cli_contents.insert(
        "timesheet.cli",
        r#"
2025.01.01
8h = Regular work
-2h = Discount for client
$50 = Fixed fee for setup
-$10 = Discount for early payment
- This is a note about the day
* Another note
"#,
    );
    let config_content = r#"
[contract]
hourly_rate = 100.0

[generator.txt]
template = "template.txt"
output = "invoice.txt"
"#;
    let template_content = r#"
Total worked: {{ total_hours_worked  }}
Total counted: {{ total_hours_counted }} {{ counted_amount }}
Total billed: {{ total_hours_billed  }} {{ billed_amount }}

Fixed fees: {{ total_fixed_fees }} {{ total_discounts }}
Overage: {{ overage_hours }} {{ overage_discount }}

Total hours: {{ total_hours }}
Total amount: {{ total_amount }}
"#;

    let temp_dir = create_test_env(&cli_contents, config_content)?;
    std::fs::write(temp_dir.path().join("template.txt"), template_content)?;

    let output_path = temp_dir.path().join("invoice.txt");
    let directory_option = Some(temp_dir.path().to_str().unwrap().to_string());
    let config_file_option = Some(temp_dir.path().join("clinvoice.toml").to_str().unwrap().to_string());

    generate::run(
        Some(output_path.to_str().unwrap().to_string()),
        &Some("txt".to_string()),
        &None,
        &directory_option,
        &config_file_option,
        &[],
    );

    let generated_content = std::fs::read_to_string(&output_path)?;
    println!("{}", generated_content);

    // 8h - 2h = 6h
    assert!(generated_content.contains("Total hours: 6"));
    // (6h * 100) + 50 - 10 = 600 + 40 = 640
    assert!(generated_content.contains("Total amount: 640"));

    assert!(generated_content.contains("Total worked: 6"));
    assert!(generated_content.contains("Total counted: 6 600"));
    assert!(generated_content.contains("Total billed: 6 600"));

    assert!(generated_content.contains("Fixed fees: 50 -10"));
    assert!(generated_content.contains("Overage: 0 0"));

    Ok(())
}

#[test]
fn test_generate_with_non_default_generator() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli_contents = HashMap::new();
    cli_contents.insert(
        "timesheet.cli",
        r#"
2025.01.01
1h = Test
"#,
    );
    let config_content = r#"
[contract]
hourly_rate = 50.0

[generator]
default = "default_one"

[generator.default_one]
template = "default_template.txt"
output = "default_invoice.txt"

[generator.custom]
template = "custom_template.txt"
output = "custom_invoice.txt"
"#;
    let default_template_content = "Default: {{ total_amount }}";
    let custom_template_content = "Custom: {{ total_amount }}";

    let temp_dir = create_test_env(&cli_contents, config_content)?;
    std::fs::write(temp_dir.path().join(r#"default_template.txt"#), default_template_content)?;
    std::fs::write(temp_dir.path().join(r#"custom_template.txt"#), custom_template_content)?;

    // Test default generator
    let default_output_path = temp_dir.path().join("default_invoice.txt");
    let directory_option = Some(temp_dir.path().to_str().unwrap().to_string());
    let config_file_option = Some(temp_dir.path().join("clinvoice.toml").to_str().unwrap().to_string());

    generate::run(
        Some(default_output_path.to_str().unwrap().to_string()),
        &None, // Use default generator
        &None,
        &directory_option,
        &config_file_option,
        &[],
    );
    let generated_content = std::fs::read_to_string(&default_output_path)?;
    println!("{}", generated_content);

    assert!(generated_content.contains("Default: 50"));

    // Test custom generator
    let custom_output_path = temp_dir.path().join("custom_invoice.txt");
    generate::run(
        Some(custom_output_path.to_str().unwrap().to_string()),
        &Some("custom".to_string()), // Use custom generator
        &None,
        &directory_option,
        &config_file_option,
        &[],
    );
    let generated_content = std::fs::read_to_string(&custom_output_path)?;
    assert!(generated_content.contains("Custom: 50"));

    Ok(())
}

#[test]
fn test_generate_with_invalid_config() -> Result<(), Box<dyn std::error::Error>> {
    let cli_contents = HashMap::new(); // Not relevant for this test
    let config_content = r#"
[contract]
hourly_rate = "invalid"
"#;
    let temp_dir = create_test_env(&cli_contents, config_content)?;
    let directory_option = Some(temp_dir.path().to_str().unwrap().to_string());
    let config_file_option = Some(temp_dir.path().join("clinvoice.toml").to_str().unwrap().to_string());

    let result = std::panic::catch_unwind(|| {
        generate::run(
            None,
            &None,
            &None,
            &directory_option,
            &config_file_option,
            &[],
        );
    });

    assert!(result.is_err());
    // You might want to check the error message more specifically if generate::run
    // returns a Result type instead of panicking.

    Ok(())
}

#[test]
fn test_cap_hours_per_day() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli_contents = HashMap::new();
    cli_contents.insert(
        "timesheet.cli",
        r#"
2025.01.01
10h = Project X
"#,
    );
    let config_content = r#"
[contract]
hourly_rate = 100.0
cap_hours_per_day = 8.0

[generator.txt]
template = "template.txt"
output = "invoice.txt"
"#;
    let template_content = r#"
Total worked: {{ total_hours_worked  }}
Total counted: {{ total_hours_counted }} {{ counted_amount }}
Total billed: {{ total_hours_billed  }} {{ billed_amount }}

Fixed fees: {{ total_fixed_fees }} {{ total_discounts }}
Overage: {{ overage_hours }} {{ overage_discount }}

Total hours: {{ total_hours }}
Total amount: {{ total_amount }}
Day 1: {{ days.0.hours }} {{ days.0.description }}
"#;

    let temp_dir = create_test_env(&cli_contents, config_content)?;
    std::fs::write(temp_dir.path().join("template.txt"), template_content)?;

    let output_path = temp_dir.path().join("invoice.txt");
    let directory_option = Some(temp_dir.path().to_str().unwrap().to_string());
    let config_file_option = Some(temp_dir.path().join("clinvoice.toml").to_str().unwrap().to_string());

    generate::run(
        Some(output_path.to_str().unwrap().to_string()),
        &Some("txt".to_string()),
        &None,
        &directory_option,
        &config_file_option,
        &[],
    );

    let generated_content = std::fs::read_to_string(&output_path)?;
    println!("{}", generated_content);

    assert!(generated_content.contains("Total worked: 10"));
    assert!(generated_content.contains("Total counted: 8 800")); // Capped at 8 hours
    assert!(generated_content.contains("Total billed: 8 800"));
    assert!(generated_content.contains("Total hours: 8"));
    assert!(generated_content.contains("Total amount: 800"));
    assert!(generated_content.contains("Day 1: 8 Project X (10 worked, 8 billed)"));

    Ok(())
}

#[test]
fn test_cap_hours_per_invoice() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli_contents = HashMap::new();
    cli_contents.insert(
        "timesheet.cli",
        r#"
2025.01.01
8h = Project A
2025.01.02
8h = Project B
"#,
    );
    let config_content = r#"
[contract]
hourly_rate = 100.0
cap_hours_per_invoice = 12.0

[generator.txt]
template = "template.txt"
output = "invoice.txt"
"#;
    let template_content = r#"
Total worked: {{ total_hours_worked  }}
Total counted: {{ total_hours_counted }} {{ counted_amount }}
Total billed: {{ total_hours_billed  }} {{ billed_amount }}

Fixed fees: {{ total_fixed_fees }} {{ total_discounts }}
Overage: {{ overage_hours }} {{ overage_discount }}

Total hours: {{ total_hours }}
Total amount: {{ total_amount }}
"#;

    let temp_dir = create_test_env(&cli_contents, config_content)?;
    std::fs::write(temp_dir.path().join("template.txt"), template_content)?;

    let output_path = temp_dir.path().join("invoice.txt");
    let directory_option = Some(temp_dir.path().to_str().unwrap().to_string());
    let config_file_option = Some(temp_dir.path().join("clinvoice.toml").to_str().unwrap().to_string());

    generate::run(
        Some(output_path.to_str().unwrap().to_string()),
        &Some("txt".to_string()),
        &None,
        &directory_option,
        &config_file_option,
        &[],
    );

    let generated_content = std::fs::read_to_string(&output_path)?;
    println!("{}", generated_content);

    assert!(generated_content.contains("Total worked: 16"));
    assert!(generated_content.contains("Total counted: 16 1600"));
    assert!(generated_content.contains("Total billed: 12 1200")); // Capped at 12 hours
    assert!(generated_content.contains("Total hours: 16")); // This is total_hours_counted, not billed
    assert!(generated_content.contains("Total amount: 1200"));
    assert!(generated_content.contains("Overage: 4 -400")); // 16 - 12 = 4 hours overage, 4 * 100 = 400 discount

    Ok(())
}