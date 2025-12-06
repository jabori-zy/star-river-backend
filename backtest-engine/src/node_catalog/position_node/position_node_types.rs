use serde::{Deserialize, Serialize};
use snafu::OptionExt;
use star_river_core::custom_type::NodeName;
use strategy_core::{node_infra::condition_trigger::ConditionTrigger, strategy::SelectedAccount};
use strum::{Display, EnumString};

use crate::node::node_error::{PositionNodeError, position_node_error::OperationConfigNotFoundSnafu};

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString)]
pub enum PositionOperation {
    CloseAllPositions,
    ClosePosition,
    PartiallyClosePosition,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionOperationConfig {
    pub config_id: i32, // Configuration ID

    pub input_handle_id: String, // Input handle ID

    pub symbol: Option<String>,

    pub position_operation: PositionOperation, // Operation type

    pub operation_name: String, // Operation name

    pub trigger_config: ConditionTrigger,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionNodeBacktestConfig {
    #[serde(skip)]
    pub node_name: NodeName,
    pub selected_account: SelectedAccount,
    pub position_operations: Vec<PositionOperationConfig>,
}

impl PositionNodeBacktestConfig {
    pub fn find_position_operation_config(&self, config_id: i32) -> Result<&PositionOperationConfig, PositionNodeError> {
        self.position_operations
            .iter()
            .find(|config| config.config_id == config_id)
            .context(OperationConfigNotFoundSnafu {
                node_name: self.node_name.clone(),
                config_id,
            })
    }
}
