use serde::{Deserialize, Serialize};
use super::condition::Case;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfElseNodeBacktestConfig {
    pub cases: Vec<Case>,
}




