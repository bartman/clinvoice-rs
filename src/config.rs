use std::path::Path;
use toml::Value;
use std::fs;

#[allow(dead_code)]
pub struct Config {
    value: Value,
}

impl Config {
    pub fn new(path: &Path) -> Result<Self, std::io::Error> {
        let content = fs::read_to_string(path)?;
        let value: Value = toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Config { value })
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
