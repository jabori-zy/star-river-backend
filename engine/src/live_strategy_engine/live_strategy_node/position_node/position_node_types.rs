use serde::{Deserialize, Serialize};
use star_river_core::strategy::SelectedAccount;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionOperationType {
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "close_all")]
    CloseAll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationConfig {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionOperationConfig {
    #[serde(rename = "configId")]
    config_id: i32, // 配置ID
    #[serde(rename = "operationType")]
    operation_type: PositionOperationType, // 操作类型
    #[serde(rename = "operationName")]
    operation_name: String, // 操作名称
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNodeLiveConfig {
    #[serde(rename = "selectedLiveAccount")]
    pub selected_live_account: SelectedAccount,
    pub symbol: String,
    pub operations: Vec<PositionOperationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNodeSimulateConfig {
    #[serde(rename = "selectedSimulateAccount")]
    pub selected_simulate_account: SelectedAccount,
    pub symbol: String,
    pub operations: Vec<PositionOperationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNodeBacktestConfig {
    #[serde(rename = "selectedAccount")]
    pub selected_account: SelectedAccount,
    pub symbol: String,
    pub operations: Vec<PositionOperationConfig>,
}
