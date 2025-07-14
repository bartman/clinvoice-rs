use clinvoice::index::Index;
use tempfile::TempDir;
use std::fs;
use std::path::PathBuf;

// Helper function to create a temporary directory and an index file path within it
fn setup_test_env() -> (TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let index_file_path = temp_dir.path().join(".index");
    (temp_dir, index_file_path)
}

#[test]
fn test_index_new_loads_existing_file() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, index_file_path) = setup_test_env();
    let initial_content = "1 2023.01.01 2023.01.02\n2 2023.02.01\n";
    fs::write(&index_file_path, initial_content)?;

    let mut index = Index::new(&index_file_path)?;
    assert_eq!(index.find_sequence(&["2023.01.01".to_string(), "2023.01.02".to_string()]), 1);
    assert_eq!(index.find_sequence(&["2023.02.01".to_string()]), 2);
    Ok(())
}

#[test]
fn test_index_new_fails_if_file_cannot_be_created() {
    // Create a directory that we can make read-only
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let restricted_dir = temp_dir.path().join("restricted");
    fs::create_dir(&restricted_dir).expect("Failed to create restricted directory");
    // Set permissions to read-only for the owner, and no access for others
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&restricted_dir).unwrap().permissions();
        perms.set_mode(0o555); // r-xr-xr-x
        fs::set_permissions(&restricted_dir, perms).unwrap();
    }

    let index_file_path = restricted_dir.join(".index");
    let result = Index::new(&index_file_path);
    assert!(result.is_err());
    // Restore permissions so temp_dir can be cleaned up
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&restricted_dir).unwrap().permissions();
        perms.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(&restricted_dir, perms).unwrap();
    }
}

#[test]
fn test_index_find_sequence_new_sequence() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, index_file_path) = setup_test_env();
    let mut index = Index::new(&index_file_path)?;
    let dates = vec!["2023.03.01".to_string()];
    let sequence = index.find_sequence(&dates);
    assert_eq!(sequence, 1);
    Ok(())
}

#[test]
fn test_index_find_sequence_existing_sequence() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, index_file_path) = setup_test_env();
    let mut index = Index::new(&index_file_path)?;
    let dates1 = vec!["2023.03.01".to_string()];
    let dates2 = vec!["2023.04.01".to_string()];
    index.add_sequence(1, &dates1);
    index.add_sequence(2, &dates2);
    assert_eq!(index.find_sequence(&dates1), 1);
    assert_eq!(index.find_sequence(&dates2), 2);
    Ok(())
}

#[test]
fn test_index_add_sequence_new_entry() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, index_file_path) = setup_test_env();
    let mut index = Index::new(&index_file_path)?;
    let dates = vec!["2023.05.01".to_string()];
    let sequence = index.add_sequence(10, &dates);
    assert_eq!(sequence, 10);
    assert_eq!(index.find_sequence(&dates), 10);
    Ok(())
}

#[test]
fn test_index_add_sequence_replace_existing() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, index_file_path) = setup_test_env();
    let mut index = Index::new(&index_file_path)?;
    let dates1 = vec!["2023.06.01".to_string()];
    let dates2 = vec!["2023.06.02".to_string()];
    index.add_sequence(1, &dates1);
    assert_eq!(index.find_sequence(&dates1), 1);
    index.add_sequence(1, &dates2); // Replace sequence 1 with new dates
    assert_eq!(index.find_sequence(&dates2), 1);
    assert_eq!(index.find_sequence(&dates1), 2); // Old dates should now get a new sequence
    Ok(())
}

#[test]
fn test_index_with_multiple_dates() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, index_file_path) = setup_test_env();
    let mut index = Index::new(&index_file_path)?;
    let dates = vec!["2023.07.01".to_string(), "2023.07.02".to_string(), "2023.07.03".to_string()];
    let sequence = index.add_sequence(1, &dates);
    assert_eq!(sequence, 1);
    assert_eq!(index.find_sequence(&dates), 1);
    Ok(())
}

#[test]
fn test_index_save_and_reload() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, index_file_path) = setup_test_env();
    let mut index = Index::new(&index_file_path)?;
    let dates1 = vec!["2023.08.01".to_string()];
    let dates2 = vec!["2023.08.02".to_string(), "2023.08.03".to_string()];
    index.add_sequence(5, &dates1);
    index.add_sequence(6, &dates2);
    index.save()?;

    // Re-open the index to simulate a new run
    let mut reloaded_index = Index::new(&index_file_path)?;
    assert_eq!(reloaded_index.find_sequence(&dates1), 5);
    assert_eq!(reloaded_index.find_sequence(&dates2), 6);
    Ok(())
}
