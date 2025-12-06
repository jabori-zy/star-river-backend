use serde::{Deserialize, Serialize};

/// Case branch trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseBranchTrigger {
    pub from_node_type: String,
    pub from_handle_id: String,
    pub from_node_id: String,
    pub from_node_name: String,
    pub case_id: i32,
}

/// Else branch trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElseBranchTrigger {
    pub from_node_type: String,
    pub from_handle_id: String,
    pub from_node_id: String,
    pub from_node_name: String,
}

/// Condition trigger (Case or Else branch)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "triggerType", rename_all = "lowercase")]
pub enum ConditionTrigger {
    Case(CaseBranchTrigger),
    Else(ElseBranchTrigger),
}

impl ConditionTrigger {
    /// Check if Case branch
    pub fn is_case(&self) -> bool {
        matches!(self, ConditionTrigger::Case(_))
    }

    /// Check if Else branch
    pub fn is_else(&self) -> bool {
        matches!(self, ConditionTrigger::Else(_))
    }

    /// Get source node ID
    pub fn from_node_id(&self) -> &str {
        match self {
            ConditionTrigger::Case(trigger) => &trigger.from_node_id,
            ConditionTrigger::Else(trigger) => &trigger.from_node_id,
        }
    }

    /// Get source node name
    pub fn from_node_name(&self) -> &str {
        match self {
            ConditionTrigger::Case(trigger) => &trigger.from_node_name,
            ConditionTrigger::Else(trigger) => &trigger.from_node_name,
        }
    }
}
