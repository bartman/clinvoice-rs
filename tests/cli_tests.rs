
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn run_cli_tests() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli");

    for entry in fs::read_dir(test_dir).unwrap() {
        let entry = entry.unwrap();
        let test_case_dir = entry.path();
        if !test_case_dir.is_dir() {
            continue;
        }

        let test_name = test_case_dir.file_name().unwrap().to_str().unwrap();
        println!("Running test: {}", test_name);

        let args_path = test_case_dir.join("args.txt");
        let expected_stdout_path = test_case_dir.join("expected.stdout");

        let args_str = fs::read_to_string(&args_path).unwrap();
        let args: Vec<&str> = args_str.split_whitespace().collect();

        let clinvoice_bin = env!("CARGO_BIN_EXE_clinvoice");
        let output = Command::new(clinvoice_bin)
            .arg("--color")
            .arg("never")
            .arg("--config")
            .arg(test_case_dir.join("clinvoice.toml"))
            .arg("--directory")
            .arg(&test_case_dir)
            .args(&args)
            .output()
            .expect("Failed to execute command");

        let expected_stdout = fs::read_to_string(&expected_stdout_path).unwrap();

        assert_eq!(String::from_utf8_lossy(&output.stdout), expected_stdout);
        assert!(output.status.success());
    }
}
