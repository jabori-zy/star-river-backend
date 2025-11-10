use snafu::{Backtrace, Snafu};
use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, StatusCode};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum StartNodeError {
    Empty { backtrace: Backtrace }, // >= 0
                                    // #[snafu(display("[{node_name}] config {config_name} should be greater than or equal to(>= 0) zero, but got [{config_value}]"))]
                                    // ValueNotGreaterThanOrEqualToZero {
                                    //     node_name: String,
                                    //     config_name: String,
                                    //     config_value: f64,
                                    //     backtrace: Backtrace,
                                    // },

                                    // // > 0
                                    // #[snafu(display(
                                    //     "[{node_name}] config [{config_name}] should be greater than(> 0) zero, but got [{config_value}]"
                                    // ))]
                                    // ValueNotGreaterThanZero {
                                    //     node_name: String,
                                    //     config_name: String,
                                    //     config_value: f64,
                                    //     backtrace: Backtrace,
                                    // },
}

// Implement the StarRiverErrorTrait for Mt5Error
impl StarRiverErrorTrait for StartNodeError {
    fn get_prefix(&self) -> &'static str {
        "START_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            StartNodeError::Empty { .. } => 1001, // empty
        };

        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            StartNodeError::Empty { .. } => StatusCode::BAD_REQUEST, // 400 Bad Request
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => {
                // 直接使用 Display trait 中定义的英文消息
                self.to_string()
            }
            ErrorLanguage::Chinese => match self {
                StartNodeError::Empty { .. } => {
                    format!("开始节点是空的")
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        vec![self.error_code()]
    }
}
