use super::condition::Case;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfElseNodeBacktestConfig {
    pub cases: Vec<Case>,
}
