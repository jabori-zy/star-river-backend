use serde::{Serialize, Deserialize};
use types::strategy::SelectedAccount;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionOperationType {
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "close_all")]
    CloseAll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationConfig {
    #[serde(rename = "selectedAccount")]
    selected_account: SelectedAccount,
    symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionOperationConfig {
    #[serde(rename = "configId")]
    config_id: i32, // 配置ID
    #[serde(rename = "operationType")]
    operation_type: PositionOperationType, // 操作类型
    #[serde(rename = "operationName")]
    operation_name: String, // 操作名称
    #[serde(rename = "operationConfig")]
    operation_config: OperationConfig,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNodeLiveConfig {
    operations: Vec<PositionOperationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNodeSimulateConfig {
    operations: Vec<PositionOperationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNodeBacktestConfig {
    operations: Vec<PositionOperationConfig>,
}
