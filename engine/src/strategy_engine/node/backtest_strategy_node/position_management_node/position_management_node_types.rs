use serde::{Deserialize, Serialize};
use star_river_core::strategy::SelectedAccount;
use strum::{Display, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString)]
pub enum PositionOperation {
    #[serde(rename = "update")]
    #[strum(serialize = "update")]
    Update,
    #[serde(rename = "close_all")]
    #[strum(serialize = "close_all")]
    CloseAll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionOperationConfig {
    #[serde(rename = "positionOperationId")]
    pub position_operation_id: i32, // 配置ID

    #[serde(rename = "inputHandleId")]
    pub input_handle_id: String, // 输入句柄ID

    #[serde(rename = "symbol")]
    pub symbol: Option<String>,

    #[serde(rename = "positionOperation")]
    pub position_operation: PositionOperation, // 操作类型

    #[serde(rename = "positionOperationName")]
    pub position_operation_name: String, // 操作名称
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNodeBacktestConfig {
    #[serde(rename = "selectedAccount")]
    pub selected_account: SelectedAccount,

    #[serde(rename = "positionOperations")]
    pub position_operations: Vec<PositionOperationConfig>,
}
