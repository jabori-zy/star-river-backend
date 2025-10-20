use serde::{Deserialize, Serialize};
use crate::strategy::custom_variable::VariableValue;
use super::variable_operation::{VariableOperation, UpdateVarValueOperation};
use super::trigger::TriggerConfig;

// ==================== 基础配置 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VarType {
    System,
    Custom,
}



/// 基础变量配置（不包含 varType 和 varOperation，由外层枚举处理）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseVariableConfig {
    pub config_id: i32,
    pub input_handle_id: String,
    pub output_handle_id: String,
    pub var_name: String,
    pub var_display_name: String,
    // pub var_value_type: VariableValueType,
    pub trigger_config: TriggerConfig,
}

// ==================== Get Config 模块 ====================
pub mod get {
    use std::ops::Deref;

    use super::*;

    /// 获取系统变量配置
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetSystemVariableConfig {
        #[serde(flatten)]
        pub base: BaseVariableConfig,
        // varOperation 由外层枚举的 #[serde(tag = "varOperation")] 处理
        #[serde(skip_serializing_if = "Option::is_none")]
        pub symbol: Option<String>,
        pub var_value: VariableValue,
    }


    impl Deref for GetSystemVariableConfig {
        type Target = BaseVariableConfig;

        fn deref(&self) -> &Self::Target {
            &self.base
        }
    }


    impl GetSystemVariableConfig {
        pub fn var_name(&self) -> &str {
            &self.var_name
        }

        pub fn config_id(&self) -> i32 {
            self.config_id
        }
    }

    /// 获取自定义变量配置
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetCustomVariableConfig {
        #[serde(flatten)]
        pub base: BaseVariableConfig,
        // varOperation 由外层枚举的 #[serde(tag = "varOperation")] 处理
        pub var_value: VariableValue,
    }

    impl Deref for GetCustomVariableConfig {
        type Target = BaseVariableConfig;

        fn deref(&self) -> &Self::Target {
            &self.base
        }
    }

    impl GetCustomVariableConfig {
        pub fn var_name(&self) -> &str {
            &self.var_name
        }

        pub fn config_id(&self) -> i32 {
            self.config_id
        }
    }

    /// 获取变量配置（系统变量或自定义变量）
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "varType", rename_all = "lowercase")]
    pub enum GetVariableConfig {
        System(GetSystemVariableConfig),
        Custom(GetCustomVariableConfig),
    }

    impl GetVariableConfig {
        /// 判断是否为系统变量配置
        pub fn is_system(&self) -> bool {
            matches!(self, GetVariableConfig::System(_))
        }

        /// 判断是否为自定义变量配置
        pub fn is_custom(&self) -> bool {
            !self.is_system()
        }

        /// 获取变量值
        pub fn var_value(&self) -> &VariableValue {
            match self {
                GetVariableConfig::System(config) => &config.var_value,
                GetVariableConfig::Custom(config) => &config.var_value,
            }
        }

        pub fn var_name(&self) -> &str {
            match self {
                GetVariableConfig::System(config) => config.var_name(),
                GetVariableConfig::Custom(config) => config.var_name(),
            }
        }

        pub fn config_id(&self) -> i32 {
            match self {
                GetVariableConfig::System(config) => config.config_id(),
                GetVariableConfig::Custom(config) => config.config_id(),
            }
        }

        pub fn trigger_config(&self) -> &TriggerConfig {
            match self {
                GetVariableConfig::System(config) => &config.trigger_config,
                GetVariableConfig::Custom(config) => &config.trigger_config,
            }
        }
    }
}

// ==================== Update Config 模块 ====================
pub mod update {
    use std::ops::Deref;

    use super::*;

    /// 更新变量配置
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UpdateVariableConfig {
        #[serde(flatten)]
        pub base: BaseVariableConfig,
        // pub var_type: VarType, // "system" | "custom"
        // varOperation 由外层枚举的 #[serde(tag = "varOperation")] 处理
        pub update_var_value_operation: UpdateVarValueOperation,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub update_operation_value: Option<VariableValue>,
    }


    impl Deref for UpdateVariableConfig {
        type Target = BaseVariableConfig;

        fn deref(&self) -> &Self::Target {
            &self.base
        }
    }

    impl UpdateVariableConfig {


        pub fn config_id(&self) -> i32 {
            self.config_id
        }

        pub fn var_name(&self) -> &str {
            &self.var_name
        }

        /// 获取更新操作类型
        pub fn update_operation_type(&self) -> &UpdateVarValueOperation {
            &self.update_var_value_operation
        }

        /// 获取更新操作值
        pub fn update_operation_value(&self) -> Option<&VariableValue> {
            self.update_operation_value.as_ref()
        }

        pub fn trigger_config(&self) -> &TriggerConfig {
            &self.trigger_config
        }
    }
}

// ==================== Reset Config 模块 ====================
pub mod reset {
    use std::ops::Deref;

    use super::*;

    /// 重置变量配置
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ResetVariableConfig {
        #[serde(flatten)]
        pub base: BaseVariableConfig,
        // pub var_type: VarType, // "system" | "custom"
        // varOperation 由外层枚举的 #[serde(tag = "varOperation")] 处理
        pub var_initial_value: VariableValue,
    }

    impl Deref for ResetVariableConfig {
        type Target = BaseVariableConfig;

        fn deref(&self) -> &Self::Target {
            &self.base
        }
    }

    impl ResetVariableConfig {
        pub fn config_id(&self) -> i32 {
            self.config_id
        }

        /// 获取初始值
        pub fn var_initial_value(&self) -> &VariableValue {
            &self.var_initial_value
        }

        pub fn trigger_config(&self) -> &TriggerConfig {
            &self.trigger_config
        }
    }
}

// ==================== 统一配置类型 ====================

// 重新导出子模块类型
pub use get::GetVariableConfig;
pub use update::UpdateVariableConfig;
pub use reset::ResetVariableConfig;

/// 变量配置（Get、Update 或 Reset）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "varOperation", rename_all = "lowercase")]
pub enum VariableConfig {
    Get(GetVariableConfig),
    Update(UpdateVariableConfig),
    Reset(ResetVariableConfig),
}

impl VariableConfig {
    /// 判断是否为获取变量配置
    pub fn is_get(&self) -> bool {
        matches!(self, VariableConfig::Get(_))
    }

    /// 判断是否为更新变量配置
    pub fn is_update(&self) -> bool {
        matches!(self, VariableConfig::Update(_))
    }

    /// 判断是否为重置变量配置
    pub fn is_reset(&self) -> bool {
        matches!(self, VariableConfig::Reset(_))
    }

    pub fn config_id(&self) -> i32 {
        match self {
            VariableConfig::Get(config) => config.config_id(),
            VariableConfig::Update(config) => config.config_id(),
            VariableConfig::Reset(config) => config.config_id(),
        }
    }

    /// 获取变量操作类型
    pub fn var_operation(&self) -> VariableOperation {
        match self {
            VariableConfig::Get(_) => VariableOperation::Get,
            VariableConfig::Update(_) => VariableOperation::Update,
            VariableConfig::Reset(_) => VariableOperation::Reset,
        }
    }

    pub fn trigger_config(&self) -> &TriggerConfig {
        match self {
            VariableConfig::Get(config) => config.trigger_config(),
            VariableConfig::Update(config) => config.trigger_config(),
            VariableConfig::Reset(config) => config.trigger_config(),
        }
    }
}
