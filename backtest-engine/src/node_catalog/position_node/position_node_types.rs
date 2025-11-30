use serde::{Deserialize, Serialize};
use strategy_core::{node_infra::condition_trigger::ConditionTrigger, strategy::SelectedAccount};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString)]
pub enum PositionOperation {
    CloseAllPosition,
    ClosePosition,
    PartiallyClosePosition,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionOperationConfig {
    pub config_id: i32, // 配置ID

    pub input_handle_id: String, // 输入句柄ID

    pub symbol: Option<String>,

    pub position_operation: PositionOperation, // 操作类型

    pub operation_name: String, // 操作名称

    pub trigger_config: ConditionTrigger,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionNodeBacktestConfig {
    pub selected_account: SelectedAccount,
    pub position_operations: Vec<PositionOperationConfig>,
}
