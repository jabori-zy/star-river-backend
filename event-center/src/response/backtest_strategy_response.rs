use types::strategy::BacktestStrategyConfig;
use types::custom_type::NodeId;

#[derive(Debug)]
pub enum StrategyResponse {
    GetStartNodeConfig(GetStartNodeConfigResponse),
}


impl StrategyResponse {
    pub fn code(&self) -> i32 {
        match self {
            StrategyResponse::GetStartNodeConfig(get_start_node_config_response) => get_start_node_config_response.code,
        }
    }

    pub fn node_id(&self) -> &str {
        match self {
            StrategyResponse::GetStartNodeConfig(get_start_node_config_response) => &get_start_node_config_response.node_id,
        }
    }

    pub fn message(&self) -> String {
        match self {
            StrategyResponse::GetStartNodeConfig(get_start_node_config_response) => get_start_node_config_response.message.clone(),
        }
    }

    pub fn response_timestamp(&self) -> i64 {
        match self {
            StrategyResponse::GetStartNodeConfig(get_start_node_config_response) => get_start_node_config_response.response_timestamp,
        }
    }
}


#[derive(Debug)]
pub struct GetStartNodeConfigResponse {
    pub code: i32,
    pub message: String,
    pub node_id: NodeId,
    pub backtest_strategy_config: BacktestStrategyConfig,
    pub response_timestamp: i64,
}




