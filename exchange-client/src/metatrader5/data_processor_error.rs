use exchange_core::error::DataProcessorError;
use snafu::{Backtrace, Snafu};
use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, generate_error_code_chain};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Mt5DataProcessorError {
    #[snafu(transparent)]
    DataProcessorError { source: DataProcessorError, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for DataProcessorError
impl StarRiverErrorTrait for Mt5DataProcessorError {
    fn get_prefix(&self) -> &'static str {
        "MT5_DATA_PROCESSOR"
    }

    fn error_code(&self) -> ErrorCode {
        // let prefix = self.get_prefix();
        match self {
            Mt5DataProcessorError::DataProcessorError { source, .. } => source.error_code(),
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            Mt5DataProcessorError::DataProcessorError { source, .. } => generate_error_code_chain(source, self.error_code()),
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                Mt5DataProcessorError::DataProcessorError { source, .. } => source.error_message(language),
            },
        }
    }
}
