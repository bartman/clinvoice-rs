use std::collections::HashMap;
use std::path::{Path, PathBuf};
use toml::Value;
use std::fs;
use std::env;

/// Represents the application's configuration loaded from a TOML file.
#[allow(dead_code)]
pub struct Config {
    value: Value,
}

impl Config {
    /// Creates a new `Config` instance by loading and parsing a TOML configuration file.
    ///
    /// The function attempts to find the configuration file based on the provided `config_file`
    /// path or by searching predefined locations relative to `data_directory` and system defaults.
    ///
    /// # Arguments
    ///
    /// * `config_file` - An optional path to a specific configuration file.
    /// * `data_directory` - An optional base directory to search for `clinvoice.toml`.
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if the file cannot be found, read, or if the TOML content is invalid.
    pub fn new(config_file: Option<&str>, data_directory: Option<&str>) -> Result<Self, std::io::Error> {
        let config_path = Self::find_config_path(config_file, data_directory)?;
        let content = fs::read_to_string(&config_path)?;
        let value: Value = toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Config { value })
    }

    // Attempts to find the configuration file based on provided path or predefined locations.
    fn find_config_path(config_file: Option<&str>, data_directory: Option<&str>) -> Result<PathBuf, std::io::Error> {
        if let Some(path) = config_file {
            let path = PathBuf::from(path);
            if path.exists() {
                tracing::trace!("user specified config_file={} exists", path.display());
                return Ok(path.canonicalize()?);
            } else {
                tracing::trace!("user specified config_file={} does not exist", path.display());
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Specified config file does not exist"));
            }
        }

        let mut candidates: Vec<PathBuf> = Vec::new();
        // Add other predefined locations
        if let Some(dir) = data_directory {
            tracing::trace!("user specified directory={}", dir);
            candidates.push(Path::new(dir).join("clinvoice.toml"));
        }
        candidates.push(PathBuf::from("./clinvoice.toml"));
        if let Ok(home) = env::var("HOME") {
            tracing::trace!("environment HOME={}", home);
            candidates.push(Path::new(&home).join(".config").join("clinvoice").join("clinvoice.toml"));
        }

        for candidate in candidates {
            tracing::trace!("checking candidate {}", candidate.display());
            if candidate.exists() {
                tracing::debug!("found configuration {}", candidate.display());
                return Ok(candidate.canonicalize()?);
            }
        }

        tracing::trace!("configuration not found");
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No config file found in searched locations"))
    }

    /// Checks if a key exists in the configuration.
    #[allow(dead_code)]
    pub fn has(&self, key: &str) -> bool {
        self.get_value(key).is_some()
    }

    /// Returns the type of the value associated with a key, if it exists.
    #[allow(dead_code)]
    pub fn kind(&self, key: &str) -> Option<&'static str> {
        self.get_value(key).map(|v| match v {
            Value::String(_) => "string",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::Boolean(_) => "boolean",
            Value::Datetime(_) => "datetime",
            Value::Array(_) => "array",
            Value::Table(_) => "table",
        })
    }

    /// Retrieves a raw `toml::Value` for a given key.
    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.get_value(key)
    }

    /// Retrieves a value for a given key, with a default fallback if the key is not found or conversion fails.
    #[allow(dead_code)]
    pub fn get_with_default<T>(&self, key: &str, default: T) -> T
    where
        T: FromValue,
    {
        self.get_value(key)
            .and_then(|v| T::from_value(v))
            .unwrap_or(default)
    }

    /// Retrieves a string value for a given key.
    #[allow(dead_code)]
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get_value(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Retrieves an `f64` value for a given key, converting from integer if necessary.
    #[allow(dead_code)]
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.get_value(key).and_then(|v| {
            if let Some(f) = v.as_float() {
                Some(f)
            } else if let Some(i) = v.as_integer() {
                Some(i as f64)
            } else {
                None
            }
        })
    }

    /// Retrieves an `i64` value for a given key.
    #[allow(dead_code)]
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.get_value(key).and_then(|v| v.as_integer())
    }

    /// Retrieves a table (map) value for a given key.
    #[allow(dead_code)]
    pub fn get_table(&self, key: &str) -> Option<&toml::map::Map<String, Value>> {
        self.get_value(key).and_then(|v| v.as_table())
    }

    /// Returns the entire configuration as a TOML table.
    #[allow(dead_code)]
    pub fn as_table(&self) -> &toml::map::Map<String, Value> {
        self.value.as_table().unwrap()
    }

    /// Flattens the configuration into a HashMap with dot-separated keys.
    #[allow(dead_code)]
    pub fn get_flattened_values(&self, key_separator: &str) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        if let Some(table) = self.value.as_table() {
            self.flatten_table_recursive("", table, &mut map, key_separator);
        }
        map
    }

    // Recursively flattens a TOML table into a HashMap.
    fn flatten_table_recursive(&self, prefix: &str, table: &toml::map::Map<String, Value>, map: &mut HashMap<String, Value>, key_separator: &str) {
        for (key, value) in table {
            let new_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}{}{}", prefix, key_separator, key)
            };

            if let Some(sub_table) = value.as_table() {
                self.flatten_table_recursive(&new_key, sub_table, map, key_separator);
            } else {
                map.insert(new_key, value.clone());
            }
        }
    }

    // Retrieves a value from the configuration using a dot-separated key.
    fn get_value(&self, key: &str) -> Option<&Value> {
        let mut current = &self.value;
        for part in key.split('.') {
            match current {
                Value::Table(table) => {
                    current = table.get(part)?;
                }
                _ => return None,
            }
        }
        Some(current)
    }
}

/// A trait for converting a `toml::Value` into another type.
#[allow(dead_code)]
pub trait FromValue {
    /// Attempts to convert a `toml::Value` into `Self`.
    fn from_value(value: &Value) -> Option<Self>
    where
        Self: Sized;
}

impl FromValue for String {
    /// Converts a `toml::Value` to a `String`.
    fn from_value(value: &Value) -> Option<Self> {
        value.as_str().map(|s| s.to_string())
    }
}

impl FromValue for i64 {
    /// Converts a `toml::Value` to an `i64`.
    fn from_value(value: &Value) -> Option<Self> {
        value.as_integer()
    }
}

impl FromValue for f64 {
    /// Converts a `toml::Value` to an `f64`.
    fn from_value(value: &Value) -> Option<Self> {
        value.as_float()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use serial_test::serial;

    // Helper function to create a temporary config file
    fn create_temp_config(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        file.write_all(content.as_bytes()).expect("Failed to write to temp file");
        file
    }

    #[test]
    fn test_config_new_valid() -> Result<(), Box<dyn std::error::Error>> {
        let toml_content = r#"
            [contract]
            hourly_rate = 100.0
            payment_days = 30
        "#;
        let temp_file = create_temp_config(toml_content);
        let config = Config::new(Some(temp_file.path().to_str().unwrap()), None)?;
        assert_eq!(config.get_f64("contract.hourly_rate"), Some(100.0));
        assert_eq!(config.get_i64("contract.payment_days"), Some(30));
        Ok(())
    }

    #[test]
    fn test_config_new_invalid_toml() {
        let toml_content = r#"
            [contract
            hourly_rate = 100.0
        "#; // Malformed TOML
        let temp_file = create_temp_config(toml_content);
        let result = Config::new(Some(temp_file.path().to_str().unwrap()), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_get_string() -> Result<(), Box<dyn std::error::Error>> {
        let toml_content = r#"
            project_name = "My Project"
        "#;
        let temp_file = create_temp_config(toml_content);
        let config = Config::new(Some(temp_file.path().to_str().unwrap()), None)?;
        assert_eq!(config.get_string("project_name"), Some("My Project".to_string()));
        assert_eq!(config.get_string("non_existent"), None);
        Ok(())
    }

    #[test]
    fn test_config_has() -> Result<(), Box<dyn std::error::Error>> {
        let toml_content = r#"
            [section]
            key = "value"
        "#;
        let temp_file = create_temp_config(toml_content);
        let config = Config::new(Some(temp_file.path().to_str().unwrap()), None)?;
        assert!(config.has("section.key"));
        assert!(!config.has("section.non_existent_key"));
        assert!(!config.has("non_existent_section.key"));
        Ok(())
    }

    #[test]
    fn test_config_kind() -> Result<(), Box<dyn std::error::Error>> {
        let toml_content = r#"
            str_key = "hello"
            int_key = 123
            float_key = 1.23
            bool_key = true
            date_key = 2023-01-01T12:00:00Z
            arr_key = [1, 2, 3]
            [table_key]
            sub_key = "value"
        "#;
        let temp_file = create_temp_config(toml_content);
        let config = Config::new(Some(temp_file.path().to_str().unwrap()), None)?;

        assert_eq!(config.kind("str_key"), Some("string"));
        assert_eq!(config.kind("int_key"), Some("integer"));
        assert_eq!(config.kind("float_key"), Some("float"));
        assert_eq!(config.kind("bool_key"), Some("boolean"));
        assert_eq!(config.kind("date_key"), Some("datetime"));
        assert_eq!(config.kind("arr_key"), Some("array"));
        assert_eq!(config.kind("table_key"), Some("table"));
        assert_eq!(config.kind("non_existent"), None);
        Ok(())
    }

    #[test]
    fn test_config_get_f64() -> Result<(), Box<dyn std::error::Error>> {
        let toml_content = r#"
            float_val = 123.45
            int_val = 678
        "#;
        let temp_file = create_temp_config(toml_content);
        let config = Config::new(Some(temp_file.path().to_str().unwrap()), None)?;

        assert_eq!(config.get_f64("float_val"), Some(123.45));
        assert_eq!(config.get_f64("int_val"), Some(678.0));
        assert_eq!(config.get_f64("non_existent"), None);
        Ok(())
    }

    #[test]
    fn test_config_get_i64() -> Result<(), Box<dyn std::error::Error>> {
        let toml_content = r#"
            int_val = 123
            float_val = 456.78 # Should return None for float
        "#;
        let temp_file = create_temp_config(toml_content);
        let config = Config::new(Some(temp_file.path().to_str().unwrap()), None)?;

        assert_eq!(config.get_i64("int_val"), Some(123));
        assert_eq!(config.get_i64("float_val"), None);
        assert_eq!(config.get_i64("non_existent"), None);
        Ok(())
    }

    #[test]
    fn test_config_get_table() -> Result<(), Box<dyn std::error::Error>> {
        let toml_content = r#"
            [section]
            key = "value"
            [another_section]
            num = 123
        "#;
        let temp_file = create_temp_config(toml_content);
        let config = Config::new(Some(temp_file.path().to_str().unwrap()), None)?;

        let table = config.get_table("section").unwrap();
        assert_eq!(table.get("key").unwrap().as_str(), Some("value"));

        assert!(config.get_table("non_existent").is_none());
        assert!(config.get_table("section.key").is_none()); // Not a table
        Ok(())
    }

    #[test]
    fn test_config_as_table() -> Result<(), Box<dyn std::error::Error>> {
        let toml_content = r#"
            key1 = "value1"
            [section]
            key2 = "value2"
        "#;
        let temp_file = create_temp_config(toml_content);
        let config = Config::new(Some(temp_file.path().to_str().unwrap()), None)?;

        let table = config.as_table();
        assert_eq!(table.get("key1").unwrap().as_str(), Some("value1"));
        assert!(table.get("section").is_some());
        Ok(())
    }

    #[test]
    fn test_config_get_flattened_values() -> Result<(), Box<dyn std::error::Error>> {
        let toml_content = r#"
            key1 = "value1"
            [section1]
            key2 = "value2"
            [section1.subsection]
            key3 = "value3"
            [section2]
            key4 = 123
        "#;
        let temp_file = create_temp_config(toml_content);
        let config = Config::new(Some(temp_file.path().to_str().unwrap()), None)?;

        let flattened = config.get_flattened_values(".");
        assert_eq!(flattened.get("key1").unwrap().as_str(), Some("value1"));
        assert_eq!(flattened.get("section1.key2").unwrap().as_str(), Some("value2"));
        assert_eq!(flattened.get("section1.subsection.key3").unwrap().as_str(), Some("value3"));
        assert_eq!(flattened.get("section2.key4").unwrap().as_integer(), Some(123));
        assert_eq!(flattened.len(), 4);

        let flattened_underscore = config.get_flattened_values("_");
        assert_eq!(flattened_underscore.get("section1_key2").unwrap().as_str(), Some("value2"));
        Ok(())
    }

    #[test]
    fn test_config_find_config_path_specified_file() -> Result<(), Box<dyn std::error::Error>> {
        let temp_file = create_temp_config("key = \"value\"");
        let path = temp_file.path().to_str().unwrap();
        let found_path = Config::find_config_path(Some(path), None)?;
        assert_eq!(found_path, PathBuf::from(path).canonicalize()?);
        Ok(())
    }

    #[test]
    fn test_config_find_config_path_specified_file_not_found() {
        let result = Config::find_config_path(Some("/non/existent/path/to/config.toml"), None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn test_config_find_config_path_data_directory() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        env::set_var("temp", temp_dir.path());
        let config_path = temp_dir.path().join("clinvoice.toml");
        std::fs::write(&config_path, "key = \"value\"")?;

        let found_path = Config::find_config_path(None, Some(temp_dir.path().to_str().unwrap()))?;
        assert_eq!(found_path, config_path.canonicalize()?);
        Ok(())
    }

    #[test]
    #[serial(set_current_dir)]
    fn test_config_find_config_path_default_locations() -> Result<(), Box<dyn std::error::Error>> {
        let original_home = env::var("HOME");
        let original_dir = env::current_dir()?;

        let temp_home_dir = tempfile::tempdir()?;
        env::set_var("HOME", temp_home_dir.path());
        tracing::trace!("HOME: {}", temp_home_dir.path().display());

        let temp_current_dir = tempfile::tempdir()?;
        env::set_current_dir(&temp_current_dir)?;
        tracing::trace!("PWD: {}", temp_current_dir.path().display());

        let config_path = temp_current_dir.path().join("clinvoice.toml");
        std::fs::write(&config_path, "key = \"value\"")?;

        let found_path = Config::find_config_path(None, None)?;
        assert_eq!(found_path, config_path.canonicalize()?);

        if let Ok(home) = original_home {
            env::set_var("HOME", home);
        }
        env::set_current_dir(&original_dir)?;
        Ok(())
    }

    #[test]
    #[serial(set_current_dir)]
    fn test_config_find_config_path_no_config_found_isolated() -> Result<(), Box<dyn std::error::Error>> {
        let original_home = env::var("HOME");
        let original_dir = env::current_dir()?;

        let temp_home_dir = tempfile::tempdir()?;
        env::set_var("HOME", temp_home_dir.path());
        tracing::trace!("HOME: {}", temp_home_dir.path().display());

        let temp_current_dir = tempfile::tempdir()?;
        env::set_current_dir(&temp_current_dir)?;
        tracing::trace!("PWD: {}", temp_current_dir.path().display());

        let result = Config::find_config_path(None, None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);

        if let Ok(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
        env::set_current_dir(&original_dir)?;
        Ok(())
    }
}
