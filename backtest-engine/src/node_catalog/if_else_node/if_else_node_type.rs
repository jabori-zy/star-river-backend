use serde::{Deserialize, Serialize};
use strategy_core::node_infra::if_else_node::Case;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfElseNodeBacktestConfig {
    pub cases: Vec<Case>,
}
