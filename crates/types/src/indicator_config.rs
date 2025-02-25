use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SMAConfig {
    pub period: i32,
}

impl Default for SMAConfig {
    fn default() -> Self {
        Self {
            period: 9,
        }
    }
}

impl SMAConfig {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

pub struct EMAConfig {
    pub period: i32,
    pub long_period: i32,
}

impl Default for EMAConfig {
    fn default() -> Self {
        Self { period: 9, long_period: 21 }
    }
}
