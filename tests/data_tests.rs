use clinvoice::data::{DateSelector, Entry, TimeData};
use clinvoice::parse::parse_date_arg;
use chrono::NaiveDate;
use tempfile::tempdir;

fn create_test_cli_files(dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let file1_content = r#"
2025.01.01
8h = Project Alpha
-2h = Discount
$50 = Fixed Fee
-$10 = Fixed Discount
- This is a note
* Another note
"#;
    std::fs::write(dir.join("test1.cli"), file1_content)?;

    let file2_content = r#"
2025.01.02
4h = Project Beta
"#;
    std::fs::write(dir.join("test2.cli"), file2_content)?;

    let file3_content = r#"
2025.02.01
6h = Project Gamma
"#;
    std::fs::write(dir.join("test3.cli"), file3_content)?;

    Ok(())
}

#[test]
fn test_time_data_new_with_comments() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let file_content = r#"
# This is a full line comment
2025.01.01
// This is also a full line comment
8h = Project Alpha # This is an inline comment, should not be ignored
-2h = Discount // This is also an inline comment
    # indented comment
    // indented comment
"#;
    std::fs::write(dir.path().join("test.cli"), file_content)?;

    let selector = DateSelector::new();
    let time_data = TimeData::new(dir.path().to_str().unwrap(), &selector)?;

    assert_eq!(time_data.entries.len(), 1);

    let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let entries = time_data.entries.get(&date).unwrap();
    assert_eq!(entries.len(), 2);
    assert!(matches!(entries[0], Entry::Time(h, _) if h == 8.0));
    assert!(matches!(entries[1], Entry::Time(h, _) if h == -2.0));

    Ok(())
}

#[test]
fn test_time_data_new_all_files() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    create_test_cli_files(dir.path())?;

    let selector = DateSelector::new(); // Selects all dates by default
    let time_data = TimeData::new(dir.path().to_str().unwrap(), &selector)?;

    assert_eq!(time_data.entries.len(), 3);

    // Test 2025.01.01 entries
    let date1 = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let entries1 = time_data.entries.get(&date1).unwrap();
    assert_eq!(entries1.len(), 6);
    assert!(matches!(entries1[0], Entry::Time(h, _) if h == 8.0));
    assert!(matches!(entries1[1], Entry::Time(h, _) if h == -2.0));
    assert!(matches!(entries1[2], Entry::FixedCost(c, _) if c == 50.0));
    assert!(matches!(entries1[3], Entry::FixedCost(c, _) if c == -10.0));
    assert!(matches!(entries1[4], Entry::Note(_)));
    assert!(matches!(entries1[5], Entry::Note(_)));

    // Test 2025.01.02 entries
    let date2 = NaiveDate::from_ymd_opt(2025, 1, 2).unwrap();
    let entries2 = time_data.entries.get(&date2).unwrap();
    assert_eq!(entries2.len(), 1);
    assert!(matches!(entries2[0], Entry::Time(h, _) if h == 4.0));

    // Test 2025.02.01 entries
    let date3 = NaiveDate::from_ymd_opt(2025, 2, 1).unwrap();
    let entries3 = time_data.entries.get(&date3).unwrap();
    assert_eq!(entries3.len(), 1);
    assert!(matches!(entries3[0], Entry::Time(h, _) if h == 6.0));

    Ok(())
}

#[test]
fn test_time_data_new_with_date_selector() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    create_test_cli_files(dir.path())?;

    let mut selector = DateSelector::new();
    selector.add_range(parse_date_arg("2025.01").unwrap()); // Select only January 2025
    let time_data = TimeData::new(dir.path().to_str().unwrap(), &selector)?;

    assert_eq!(time_data.entries.len(), 2);
    assert!(time_data.entries.contains_key(&NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()));
    assert!(time_data.entries.contains_key(&NaiveDate::from_ymd_opt(2025, 1, 2).unwrap()));
    assert!(!time_data.entries.contains_key(&NaiveDate::from_ymd_opt(2025, 2, 1).unwrap()));

    Ok(())
}

#[test]
fn test_time_data_new_empty_directory() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let selector = DateSelector::new();
    let time_data = TimeData::new(dir.path().to_str().unwrap(), &selector)?;
    assert!(time_data.entries.is_empty());
    Ok(())
}

#[test]
fn test_time_data_new_non_existent_directory() {
    let selector = DateSelector::new();
    let result = TimeData::new("/non/existent/path", &selector);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
}
