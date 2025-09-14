use super::super::NodeEvent;
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::market::Exchange;
use star_river_core::strategy::sys_varibale::SysVariable;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event_type")]
pub enum VariableNodeEvent {
    #[strum(serialize = "sys-variable-updated-event")]
    #[serde(rename = "sys-variable-updated-event")]
    SysVariableUpdated(SysVariableUpdatedEvent), // 系统变量更新

    #[strum(serialize = "position-number-updated-event")]
    #[serde(rename = "position-number-updated-event")]
    PositionNumberUpdated(PositionNumberUpdateEvent), // 仓位数量更新
}

// 类型别名
pub type SysVariableUpdatedEvent = NodeEvent<SysVariableUpdatedPayload>;
pub type PositionNumberUpdateEvent = NodeEvent<PositionNumberUpdatePayload>;

// 载荷类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysVariableUpdatedPayload {
    #[serde(rename = "playIndex")]
    pub play_index: i32,

    #[serde(rename = "variableConfigId")]
    pub variable_config_id: i32, // 变量配置id

    #[serde(rename = "variable")]
    pub variable: SysVariable,

    #[serde(rename = "variableValue")]
    pub variable_value: f64,
}

impl SysVariableUpdatedPayload {
    pub fn new(
        play_index: i32,
        variable_config_id: i32,
        variable: SysVariable,
        variable_value: f64,
    ) -> Self {
        Self {
            play_index,
            variable_config_id,
            variable,
            variable_value,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNumberUpdatePayload {
    #[serde(rename = "exchange")]
    pub exchange: Option<Exchange>,

    #[serde(rename = "symbol")]
    pub symbol: Option<String>,

    #[serde(rename = "positionNumber")]
    pub position_number: u32,

    #[serde(rename = "eventTimestamp")]
    pub event_timestamp: i64,
}

impl PositionNumberUpdatePayload {
    pub fn new(
        exchange: Option<Exchange>,
        symbol: Option<String>,
        position_number: u32,
        event_timestamp: i64,
    ) -> Self {
        Self {
            exchange,
            symbol,
            position_number,
            event_timestamp,
        }
    }
}
