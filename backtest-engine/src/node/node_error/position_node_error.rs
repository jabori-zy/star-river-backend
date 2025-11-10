use snafu::{Backtrace, Snafu};
use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, StatusCode};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum PositionNodeError {
    TestError { backtrace: Backtrace },
}

impl StarRiverErrorTrait for PositionNodeError {
    fn get_prefix(&self) -> &'static str {
        "POSITION_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            PositionNodeError::TestError { .. } => 1001,
        };
        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            PositionNodeError::TestError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            PositionNodeError::TestError { .. } => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match *self {
                PositionNodeError::TestError { .. } => "test error".to_string(),
            },
        }
    }
}
