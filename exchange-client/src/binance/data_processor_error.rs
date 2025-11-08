use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, generate_error_code_chain};
use snafu::{Backtrace, Snafu};
use exchange_core::error::DataProcessorError;


#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BinanceDataProcessorError {
    #[snafu(transparent)]
    DataProcessorError { source: DataProcessorError, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for DataProcessorError
impl StarRiverErrorTrait for BinanceDataProcessorError {
    fn get_prefix(&self) -> &'static str {
        "BINANCE_DATA_PROCESSOR"
    }

    fn error_code(&self) -> ErrorCode {
        // let prefix = self.get_prefix();
        match self {
            BinanceDataProcessorError::DataProcessorError { source, .. } => source.error_code(),
        }
    }


    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            BinanceDataProcessorError::DataProcessorError { source, .. } => generate_error_code_chain(source),
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                BinanceDataProcessorError::DataProcessorError { source, .. } => source.error_message(language),
            },
        }
    }
}
