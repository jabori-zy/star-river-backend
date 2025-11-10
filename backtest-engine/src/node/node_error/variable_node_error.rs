use star_river_core::error::{ErrorCode, StarRiverErrorTrait, ErrorLanguage, StatusCode};
use snafu::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum VariableNodeError {
    #[snafu(display("the symbol of system variable is null: {sys_var_name}"))]
    SysVariableSymbolIsNull {
        sys_var_name: String,
        backtrace: Backtrace,
    }
}

impl StarRiverErrorTrait for VariableNodeError {
    fn get_prefix(&self) -> &'static str {
        "VARIABLE_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            VariableNodeError::SysVariableSymbolIsNull { .. } => 1001, //系统变量交易对为空
        };

        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            VariableNodeError::SysVariableSymbolIsNull { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        // All GetVariableNodeError variants either have no source or
        // have external sources (serde_json::Error) that don't implement our trait
        vec![self.error_code()]
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                VariableNodeError::SysVariableSymbolIsNull { sys_var_name, .. } => {
                    format!("系统变量 [{}] 的交易对为空", sys_var_name)
                }
            },
        }
    }
}
