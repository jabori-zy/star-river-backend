use serde::{Deserialize, Serialize};
use strum::Display;

/**
 * 更新变量值的操作类型
 */
#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(rename_all = "lowercase")]
pub enum UpdateVarValueOperation {
    #[serde(rename = "set")]
    #[strum(serialize = "set")]
    Set, // 设置变量值
    #[serde(rename = "add")]
    #[strum(serialize = "add")]
    Add, // 增加变量值
    #[serde(rename = "subtract")]
    #[strum(serialize = "subtract")]
    Subtract, // 减少变量值
    #[serde(rename = "multiply")]
    #[strum(serialize = "multiply")]
    Multiply, // 乘法
    #[serde(rename = "divide")]
    #[strum(serialize = "divide")]
    Divide, // 除法
    #[serde(rename = "max")]
    #[strum(serialize = "max")]
    Max, // 最大值
    #[serde(rename = "min")]
    #[strum(serialize = "min")]
    Min, // 最小值
    #[serde(rename = "toggle")]
    #[strum(serialize = "toggle")]
    Toggle, // 切换变量值
    #[serde(rename = "append")]
    #[strum(serialize = "append")]
    Append, // 添加变量值
    #[serde(rename = "remove")]
    #[strum(serialize = "remove")]
    Remove, // 删除变量值
    #[serde(rename = "clear")]
    #[strum(serialize = "clear")]
    Clear, // 清空变量值
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VariableOperation {
    Get,    // 获取变量值
    Update, // 更新变量值
    Reset,  // 重置变量值
}
