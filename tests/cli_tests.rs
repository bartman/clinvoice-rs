use std::env;
use std::fs;
use std::path::{Path,PathBuf};
use std::process::Command;
use regex::Regex;
use clinvoice::color::*;
use colored::Color;
use rstest::rstest;

fn copy_files_to_directory(src_dir : &Path, dst_dir : &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(&src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap();
            let dst_path = dst_dir.join(file_name);
            fs::copy(&path, &dst_path)?;
        }
    }
    Ok(())
}

fn run_one_cli_test(test_case_dir : PathBuf) {
    let test_name = test_case_dir.file_name().unwrap().to_str().unwrap();
    println!("Running test: {}", test_name.colored(Color::BrightBlue));

    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let temp_dir_path = temp_dir.path();

    println!("  test_case_dir: {:?}", test_case_dir);
    println!("  temp_dir_path:      {:?}", temp_dir_path);

    copy_files_to_directory(&test_case_dir, temp_dir_path).expect("Failed to copy files");

    let args_path = test_case_dir.join("args.txt");
    let args_str = fs::read_to_string(&args_path).unwrap();
    let args: Vec<&str> = args_str.split_whitespace().collect();

    let clinvoice_bin = env!("CARGO_BIN_EXE_clinvoice");
    let mut command = Command::new(clinvoice_bin);
    command
        .arg("--color")
        .arg("never")
        .arg("--config")
        .arg(temp_dir_path.join("clinvoice.toml"))
        .arg("--directory")
        .arg(".") // Set directory to current working directory
        .current_dir(&temp_dir_path) // Set working directory for the command
        .args(&args);

    let output = command.output().expect("Failed to execute command");

    println!("  stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("  stderr:\n{}", String::from_utf8_lossy(&output.stderr));

    let mut is_generate_test = false;
    let mut output_file_name_from_cli: Option<String> = None;

    // Check if it's a generate command and extract output file name from stderr
    if args.contains(&"generate") {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        let re = Regex::new(r"Generating (.+)").unwrap();
        if let Some(captures) = re.captures(&stderr_str) {
            output_file_name_from_cli = Some(captures[1].to_string());
            is_generate_test = true;
        }
    }

    if is_generate_test {
        let generated_file_path = temp_dir_path.join(Path::new(&output_file_name_from_cli.unwrap()).file_name().unwrap());
        let expected_output_path = test_case_dir.join("expected.output");

        println!("  generated_file_path: {:?}", generated_file_path);
        println!("  expected_output_path: {:?}", expected_output_path);

        let ls_output = Command::new("ls").arg("-l").arg(&temp_dir_path).output().expect("Failed to run ls");
        println!("  ls -l {}:\n{}", temp_dir_path.display(), String::from_utf8_lossy(&ls_output.stdout));

        assert!(generated_file_path.exists(), "Generated file does not exist: {:?}", generated_file_path);

        let generated_content = fs::read_to_string(&generated_file_path).unwrap();
        let expected_content = fs::read_to_string(&expected_output_path).unwrap();

        assert_eq!(generated_content, expected_content, "Generated file content mismatch for test: {}", test_name);
    } else {
        let expected_stdout_path = test_case_dir.join("expected.stdout");
        let expected_stdout = fs::read_to_string(&expected_stdout_path).unwrap();
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected_stdout, "Stdout mismatch for test: {}", test_name);
    }

    assert!(output.status.success(), "Command failed for test: {}", test_name);

    println!("Successfully completed: {}", test_name.colored(Color::BrightGreen));
}

#[rstest]
#[case("01_log_year_all")]
#[case("02_log_year_2010")]
#[case("03_log_year_2011")]
#[case("04_log_month_all")]
#[case("05_log_month_2010-11")]
#[case("06_log_month_2010-12_2011-01")]
#[case("07_log_day_all")]
#[case("08_log_day_2010-11-01")]
#[case("09_log_day_2010-11-01_2010-12-01")]
#[case("10_log_full_all")]
#[case("11_log_full_2010-11-01")]
#[case("12_log_full_2010-11-01_2010-12-01")]
#[case("13_generate_txt_single_file")]
#[case("14_generate_latex_single_file")]
#[case("15_generate_txt_multiple_input_files")]
#[case("16_generate_txt_index_seq_1")]
#[case("17_generate_txt_index_seq_2_same_dates")]
#[case("18_generate_txt_index_seq_3_diff_dates")]

fn cli_test_case(#[case] test_name : &str) {
    let test_dir_base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli");

    let test_case_dir = test_dir_base.join(test_name);
    if test_case_dir.is_dir() {
        run_one_cli_test(test_case_dir);
    }
}

