use serde::{Deserialize, Serialize};

use super::{
    trigger::TriggerConfig,
    variable_operation::{UpdateVarValueOperation, VariableOperation},
};
use crate::variable::custom_variable::VariableValue;

// ==================== Base Configuration ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VarType {
    System,
    Custom,
}

/// Base variable configuration (excludes varType and varOperation, handled by outer enum)
#[derive(Debug, Clone, Deserialize)]
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

// ==================== Get Config Module ====================
pub mod get {
    use std::ops::Deref;

    use super::*;

    /// Get system variable configuration
    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetSystemVariableConfig {
        #[serde(flatten)]
        pub base: BaseVariableConfig,
        // varOperation handled by outer enum's #[serde(tag = "varOperation")]
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
        pub fn symbol(&self) -> &Option<String> {
            &self.symbol
        }

        pub fn var_name(&self) -> &str {
            &self.var_name
        }

        pub fn config_id(&self) -> i32 {
            self.config_id
        }

        pub fn output_handle_id(&self) -> &String {
            &self.output_handle_id
        }

        pub fn var_display_name(&self) -> &String {
            &self.var_display_name
        }
    }

    /// Get custom variable configuration
    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetCustomVariableConfig {
        #[serde(flatten)]
        pub base: BaseVariableConfig,
        // varOperation handled by outer enum's #[serde(tag = "varOperation")]
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

        pub fn output_handle_id(&self) -> &String {
            &self.output_handle_id
        }
    }

    /// Get variable configuration (system or custom variable)
    #[derive(Debug, Clone, Deserialize)]
    #[serde(tag = "varType", rename_all = "lowercase")]
    pub enum GetVariableConfig {
        System(GetSystemVariableConfig),
        Custom(GetCustomVariableConfig),
    }

    impl GetVariableConfig {
        /// Check if system variable configuration
        pub fn is_system(&self) -> bool {
            matches!(self, GetVariableConfig::System(_))
        }

        /// Check if custom variable configuration
        pub fn is_custom(&self) -> bool {
            !self.is_system()
        }

        /// Get variable value
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

        pub fn output_handle_id(&self) -> &String {
            match self {
                GetVariableConfig::System(config) => config.output_handle_id(),
                GetVariableConfig::Custom(config) => config.output_handle_id(),
            }
        }
    }
}

// ==================== Update Config Module ====================
pub mod update {
    use std::ops::Deref;

    use super::*;

    /// Update variable configuration
    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UpdateVariableConfig {
        #[serde(flatten)]
        pub base: BaseVariableConfig,
        // pub var_type: VarType, // "system" | "custom"
        // varOperation handled by outer enum's #[serde(tag = "varOperation")]
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

        /// Get update operation type
        pub fn update_var_value_operation(&self) -> &UpdateVarValueOperation {
            &self.update_var_value_operation
        }

        /// Get update operation value
        pub fn update_operation_value(&self) -> Option<&VariableValue> {
            self.update_operation_value.as_ref()
        }

        pub fn trigger_config(&self) -> &TriggerConfig {
            &self.trigger_config
        }

        pub fn output_handle_id(&self) -> &String {
            &self.output_handle_id
        }
    }
}

// ==================== Reset Config Module ====================
pub mod reset {
    use std::ops::Deref;

    use super::*;

    /// Reset variable configuration
    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ResetVariableConfig {
        #[serde(flatten)]
        pub base: BaseVariableConfig,
        // pub var_type: VarType, // "system" | "custom"
        // varOperation handled by outer enum's #[serde(tag = "varOperation")]
        pub var_initial_value: VariableValue,
    }

    impl Deref for ResetVariableConfig {
        type Target = BaseVariableConfig;

        fn deref(&self) -> &Self::Target {
            &self.base
        }
    }

    impl ResetVariableConfig {
        pub fn var_name(&self) -> &str {
            &self.var_name
        }

        pub fn config_id(&self) -> i32 {
            self.config_id
        }

        /// Get initial value
        pub fn var_initial_value(&self) -> &VariableValue {
            &self.var_initial_value
        }

        pub fn trigger_config(&self) -> &TriggerConfig {
            &self.trigger_config
        }

        pub fn output_handle_id(&self) -> &String {
            &self.output_handle_id
        }
    }
}

// ==================== Unified Configuration Type ====================

// Re-export submodule types
pub use get::GetVariableConfig;
pub use reset::ResetVariableConfig;
pub use update::UpdateVariableConfig;

/// Variable configuration (Get, Update, or Reset)
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "varOperation", rename_all = "lowercase")]
pub enum VariableConfig {
    Get(GetVariableConfig),
    Update(UpdateVariableConfig),
    Reset(ResetVariableConfig),
}

impl VariableConfig {
    pub fn confing_id(&self) -> i32 {
        match self {
            VariableConfig::Get(config) => config.config_id(),
            VariableConfig::Update(config) => config.config_id(),
            VariableConfig::Reset(config) => config.config_id(),
        }
    }

    /// Check if Get variable configuration
    pub fn is_get(&self) -> bool {
        matches!(self, VariableConfig::Get(_))
    }

    /// Check if Update variable configuration
    pub fn is_update(&self) -> bool {
        matches!(self, VariableConfig::Update(_))
    }

    /// Check if Reset variable configuration
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

    /// Get variable operation type
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

    pub fn output_handle_id(&self) -> &String {
        match self {
            VariableConfig::Get(config) => config.output_handle_id(),
            VariableConfig::Update(config) => config.output_handle_id(),
            VariableConfig::Reset(config) => config.output_handle_id(),
        }
    }
}
