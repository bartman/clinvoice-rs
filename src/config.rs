use std::path::{Path, PathBuf};
use toml::Value;
use std::fs;
use std::env;

#[allow(dead_code)]
pub struct Config {
    value: Value,
}

impl Config {
    pub fn new(config_file: Option<&str>, data_directory: Option<&str>) -> Result<Self, std::io::Error> {
        let config_path = Self::find_config_path(config_file, data_directory)?;
        let content = fs::read_to_string(&config_path)?;
        let value: Value = toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Config { value })
    }

    fn find_config_path(config_file: Option<&str>, data_directory: Option<&str>) -> Result<PathBuf, std::io::Error> {
        if let Some(path) = config_file {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            } else {
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Specified config file does not exist"));
            }
        }

        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Some(dir) = data_directory {
            candidates.push(Path::new(dir).join("clinvoice.toml"));
        }
        candidates.push(PathBuf::from("./clinvoice.toml"));
        if let Ok(home) = env::var("HOME") {
            candidates.push(Path::new(&home).join(".config").join("clinvoice").join("clinvoice.toml"));
        }

        for candidate in candidates {
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No config file found in searched locations"))
    }

    #[allow(dead_code)]
    pub fn has(&self, key: &str) -> bool {
        self.get_value(key).is_some()
    }

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

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.get_value(key)
    }

    #[allow(dead_code)]
    pub fn get_with_default<T>(&self, key: &str, default: T) -> T
    where
        T: FromValue,
    {
        self.get_value(key)
            .and_then(|v| T::from_value(v))
            .unwrap_or(default)
    }

    #[allow(dead_code)]
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

#[allow(dead_code)]
pub trait FromValue {
    fn from_value(value: &Value) -> Option<Self>
    where
        Self: Sized;
}

impl FromValue for String {
    fn from_value(value: &Value) -> Option<Self> {
        value.as_str().map(|s| s.to_string())
    }
}

impl FromValue for i64 {
    fn from_value(value: &Value) -> Option<Self> {
        value.as_integer()
    }
}

impl FromValue for f64 {
    fn from_value(value: &Value) -> Option<Self> {
        value.as_float()
    }
}