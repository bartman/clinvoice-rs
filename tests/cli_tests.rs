use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use regex::Regex;
use rstest::rstest;
use tempfile::TempDir;

// --- Helper Functions --- //

/// Recursively copies the contents of a source directory to a destination directory.
fn copy_dir_contents(src: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !dest.exists() {
        fs::create_dir_all(dest)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().ok_or("Invalid file name")?;
        let dest_path = dest.join(file_name);

        if path.is_dir() {
            copy_dir_contents(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

/// Reads the arguments from args.txt for a given test case directory.
fn read_test_case_args(test_case_dir: &Path) -> Vec<String> {
    let args_path = test_case_dir.join("args.txt");
    fs::read_to_string(&args_path)
        .expect(&format!("Failed to read args.txt in {:?}", test_case_dir))
        .split_whitespace()
        .map(String::from)
        .collect()
}

/// Executes the clinvoice command for a given test case.
fn execute_clinvoice_command(test_case_dir: &Path, args: &[String]) -> Output {
    let clinvoice_bin = env!("CARGO_BIN_EXE_clinvoice");
    Command::new(clinvoice_bin)
        .arg("--color")
        .arg("never")
        .arg("--config")
        .arg(test_case_dir.join("clinvoice.toml"))
        .arg("--directory")
        .arg(".") // Set directory to current working directory
        .current_dir(test_case_dir) // Set working directory for the command
        .args(args)
        .output()
        .expect("Failed to execute clinvoice command")
}

/// Extracts the generated filename from clinvoice's stderr output.
fn extract_generated_filename(stderr: &str) -> Option<String> {
    let re = Regex::new(r"Generating (.+)").unwrap();
    re.captures(stderr).map(|captures| captures[1].to_string())
}

// --- Parameterized Test --- //

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
fn cli_test_case(#[case] test_name: &str) {
    let test_dir_base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli");
    let original_test_case_dir = test_dir_base.join(test_name);

    // Create a temporary directory for this test case
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let temp_test_case_path = temp_dir.path();

    // Copy test case files to the temporary directory
    copy_dir_contents(&original_test_case_dir, &temp_test_case_path)
        .expect("Failed to copy test case files to temporary directory");

    println!("Running test: {}", test_name);

    let args = read_test_case_args(&temp_test_case_path);
    let output = execute_clinvoice_command(&temp_test_case_path, &args);

    println!("  stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("  stderr:\n{}", String::from_utf8_lossy(&output.stderr));

    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let is_generate_test = args.iter().any(|arg| arg == "generate");

    if is_generate_test {
        let generated_filename = extract_generated_filename(&stderr_str)
            .expect("Failed to extract generated filename from stderr");
        let generated_file_path = temp_test_case_path.join(Path::new(&generated_filename).file_name().unwrap());
        let expected_output_path = original_test_case_dir.join("expected.output");

        println!("  test_case_dir: {:?}", temp_test_case_path);
        println!("  generated_file_path: {:?}", generated_file_path);
        println!("  expected_output_path: {:?}", expected_output_path);

        let ls_output = Command::new("ls").arg("-l").arg(&temp_test_case_path).output().expect("Failed to run ls");
        println!("  ls -l {}:\n{}", temp_test_case_path.display(), String::from_utf8_lossy(&ls_output.stdout));

        assert!(generated_file_path.exists(), "Generated file does not exist: {:?}", generated_file_path);

        let generated_content = fs::read_to_string(&generated_file_path).unwrap();
        let expected_content = fs::read_to_string(&expected_output_path).unwrap();

        assert_eq!(generated_content, expected_content, "Generated file content mismatch for test: {}", test_name);
    } else {
        let expected_stdout_path = original_test_case_dir.join("expected.stdout");
        let expected_stdout = fs::read_to_string(&expected_stdout_path).unwrap();
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected_stdout, "Stdout mismatch for test: {}", test_name);
    }

    assert!(output.status.success(), "Command failed for test: {}", test_name);

    // TempDir automatically cleans up when it goes out of scope
}