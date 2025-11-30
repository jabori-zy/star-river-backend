use serde::{Deserialize, Serialize};

/// Case 分支触发器
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseBranchTrigger {
    pub from_node_type: String,
    pub from_handle_id: String,
    pub from_node_id: String,
    pub from_node_name: String,
    pub case_id: i32,
}

/// Else 分支触发器
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElseBranchTrigger {
    pub from_node_type: String,
    pub from_handle_id: String,
    pub from_node_id: String,
    pub from_node_name: String,
}

/// 条件触发器（Case 或 Else 分支）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "triggerType", rename_all = "lowercase")]
pub enum ConditionTrigger {
    Case(CaseBranchTrigger),
    Else(ElseBranchTrigger),
}

impl ConditionTrigger {
    /// 判断是否为 Case 分支
    pub fn is_case(&self) -> bool {
        matches!(self, ConditionTrigger::Case(_))
    }

    /// 判断是否为 Else 分支
    pub fn is_else(&self) -> bool {
        matches!(self, ConditionTrigger::Else(_))
    }

    /// 获取来源节点 ID
    pub fn from_node_id(&self) -> &str {
        match self {
            ConditionTrigger::Case(trigger) => &trigger.from_node_id,
            ConditionTrigger::Else(trigger) => &trigger.from_node_id,
        }
    }

    /// 获取来源节点名称
    pub fn from_node_name(&self) -> &str {
        match self {
            ConditionTrigger::Case(trigger) => &trigger.from_node_name,
            ConditionTrigger::Else(trigger) => &trigger.from_node_name,
        }
    }
}
