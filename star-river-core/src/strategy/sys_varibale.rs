use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString)]
pub enum SysVariable {
    #[serde(rename = "position_number")]
    #[strum(serialize = "position_number")]
    PositionNumber, //持仓数量
    #[serde(rename = "filled_order_number")]
    #[strum(serialize = "filled_order_number")]
    FilledOrderNumber, // 已成交订单数量
}

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
