
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
#[derive(Debug, Clone, Serialize, Deserialize)]


pub struct StrategySysVariable {
    pub id: i32,
    pub strategy_id: i32,
    pub position_number: i32,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl StrategySysVariable {
    pub fn get_position_number(&self) -> i32 {
        self.position_number
    }
}
