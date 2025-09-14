use tokio::sync::{mpsc, oneshot};
use crate::system::DateTimeUtc;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use entity::strategy_sys_variable::Model as StrategySysVariableModel;
use crate::system::system_config::SystemConfigManager;

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
    pub create_time: DateTimeUtc,
    pub update_time: DateTimeUtc,
}

impl StrategySysVariable {
    pub fn get_position_number(&self) -> i32 {
        self.position_number
    }
}

impl From<StrategySysVariableModel> for StrategySysVariable {
    fn from(model: StrategySysVariableModel) -> Self {
        Self {
            id: model.id,
            strategy_id: model.strategy_id,
            position_number: model.position_number,
            create_time: model.create_time,
            update_time: model.update_time,
        }
    }

    
}
