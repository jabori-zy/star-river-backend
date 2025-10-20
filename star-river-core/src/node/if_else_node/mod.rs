mod condition;
mod tests;

pub use condition::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfElseNodeBacktestConfig {
    pub cases: Vec<Case>,
}