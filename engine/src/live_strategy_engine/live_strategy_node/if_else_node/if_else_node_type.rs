use super::condition::Case;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfElseNodeLiveConfig {
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfElseNodeBacktestConfig {
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfElseNodeSimulateConfig {
    pub cases: Vec<Case>,
}
