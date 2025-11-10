use star_river_core::error::{ErrorCode, StarRiverErrorTrait, ErrorLanguage, StatusCode};
use snafu::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum IfElseNodeError {
    #[snafu(display("[{node_name}] evaluate result serialization failed. reason: {source}"))]
    EvaluateResultSerializationFailed { 
        node_name: String,
        source: serde_json::Error,
        backtrace: Backtrace,
    },
}

impl StarRiverErrorTrait for IfElseNodeError {
    fn get_prefix(&self) -> &'static str {
        "IF_ELSE_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            IfElseNodeError::EvaluateResultSerializationFailed { .. } => 1001, // evaluate result serialization failed
        };

        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            IfElseNodeError::EvaluateResultSerializationFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            IfElseNodeError::EvaluateResultSerializationFailed { .. } => vec![self.error_code()],
        }
        
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                IfElseNodeError::EvaluateResultSerializationFailed { node_name, source, .. } => {
                    format!("[{node_name}] 条件结果序列化失败，原因: {source}")
                }
            },
        }
    }
}
