use std::collections::HashMap;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    data: HashMap<String, Value>,
}

impl Metadata {
    /// Create from HashMap
    pub fn from_map(data: HashMap<String, Value>) -> Self {
        Self { data }
    }

    /// Create from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let data: HashMap<String, Value> = serde_json::from_str(json)?;
        Ok(Self { data })
    }

    /// Get and deserialize to specified type
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.data.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get string
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.data.get(key)?.as_str()
    }

    /// Get integer
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.data.get(key)?.as_i64()
    }

    /// Get float
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.data.get(key)?.as_f64()
    }

    /// Get boolean
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key)?.as_bool()
    }

    /// Check if contains key
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
}
