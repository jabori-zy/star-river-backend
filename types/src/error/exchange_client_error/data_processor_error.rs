use thiserror::Error;
use crate::error::ErrorCode;

#[derive(Error, Debug)]
pub enum DataProcessorError {
    #[error("JSON parsing failed: {0}")]
    JsonParsing(#[from] serde_json::Error),

    #[error("Stream data processing failed: {message}")]
    StreamProcessing {
        message: String,
        data_type: Option<String>,
    },

    #[error("Required field '{field}' missing in JSON data")]
    MissingField {
        field: String,
        context: Option<String>,
    },

    #[error("Invalid data type for field '{field}': expected {expected}, got {actual}")]
    InvalidFieldType {
        field: String,
        expected: String,
        actual: String,
        context: Option<String>,
    },

    #[error("Failed to parse kline data: {message}")]
    KlineDataParsing {
        message: String,
        symbol: Option<String>,
        interval: Option<String>,
    },

    #[error("Failed to parse order data: {message}")]
    OrderDataParsing {
        message: String,
        order_id: Option<i64>,
    },

    #[error("Failed to parse position data: {message}")]
    PositionDataParsing {
        message: String,
        position_id: Option<i64>,
    },

    #[error("Failed to parse deal data: {message}")]
    DealDataParsing {
        message: String,
        deal_id: Option<i64>,
    },

    #[error("Failed to parse account info: {message}")]
    AccountInfoParsing {
        message: String,
        account_id: Option<i32>,
    },

    #[error("Array data parsing failed: expected array format, got {actual_type}")]
    ArrayParsing {
        actual_type: String,
        context: String,
    },

    #[error("Invalid kline array format: expected 6 elements [timestamp, open, high, low, close, volume], got {length}")]
    InvalidKlineArrayFormat {
        length: usize,
        data: String,
    },

    #[error("Type conversion failed for field '{field}': {message}")]
    TypeConversion {
        field: String,
        message: String,
        value: Option<String>,
    },

    #[error("Data validation failed: {message}")]
    DataValidation {
        message: String,
        field: Option<String>,
        value: Option<String>,
    },

    #[error("Enum parsing failed for field '{field}': unknown variant '{variant}'")]
    EnumParsing {
        field: String,
        variant: String,
        valid_variants: Vec<String>,
    },

    #[error("Stream data format error: {message}")]
    StreamDataFormat {
        message: String,
        expected_format: Option<String>,
        actual_data: Option<String>,
    },

    #[error("Timestamp conversion failed: {message}")]
    TimestampConversion {
        message: String,
        timestamp: Option<i64>,
    },

    #[error("Data processing internal error: {0}")]
    Internal(String),
}

impl DataProcessorError {
    /// Returns the error prefix for data processor errors
    pub fn get_prefix(&self) -> &'static str {
        "DATA_PROCESSOR"
    }
    
    /// Returns a string error code for data processor errors (format: DATA_PROCESSOR_NNNN)
    pub fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // JSON and serialization errors (1001-1002)
            DataProcessorError::JsonParsing(_) => 1001,
            DataProcessorError::StreamProcessing { .. } => 1002,
            
            // Field and data structure errors (1003-1006)
            DataProcessorError::MissingField { .. } => 1003,
            DataProcessorError::InvalidFieldType { .. } => 1004,
            DataProcessorError::ArrayParsing { .. } => 1005,
            DataProcessorError::InvalidKlineArrayFormat { .. } => 1006,
            
            // Type conversion and validation errors (1007-1010)
            DataProcessorError::TypeConversion { .. } => 1007,
            DataProcessorError::DataValidation { .. } => 1008,
            DataProcessorError::EnumParsing { .. } => 1009,
            DataProcessorError::TimestampConversion { .. } => 1010,
            
            // Specific data parsing errors (1011-1016)
            DataProcessorError::KlineDataParsing { .. } => 1011,
            DataProcessorError::OrderDataParsing { .. } => 1012,
            DataProcessorError::PositionDataParsing { .. } => 1013,
            DataProcessorError::DealDataParsing { .. } => 1014,
            DataProcessorError::AccountInfoParsing { .. } => 1015,
            DataProcessorError::StreamDataFormat { .. } => 1016,
            
            // Internal errors (1017)
            DataProcessorError::Internal(_) => 1017,
        };
        format!("{}_{:04}", prefix, code)
    }

    pub fn stream_processing(message: impl Into<String>, data_type: Option<String>) -> Self {
        Self::StreamProcessing {
            message: message.into(),
            data_type,
        }
    }

    pub fn missing_field(field: impl Into<String>, context: Option<String>) -> Self {
        Self::MissingField {
            field: field.into(),
            context,
        }
    }

    pub fn invalid_field_type(
        field: impl Into<String>, 
        expected: impl Into<String>, 
        actual: impl Into<String>,
        context: Option<String>
    ) -> Self {
        Self::InvalidFieldType {
            field: field.into(),
            expected: expected.into(),
            actual: actual.into(),
            context,
        }
    }

    pub fn kline_data_parsing(
        message: impl Into<String>, 
        symbol: Option<String>, 
        interval: Option<String>
    ) -> Self {
        let message = message.into();
        let enhanced_message = match (&symbol, &interval) {
            (Some(sym), Some(int)) => format!("{} (symbol: {}, interval: {})", message, sym, int),
            (Some(sym), None) => format!("{} (symbol: {})", message, sym),
            (None, Some(int)) => format!("{} (interval: {})", message, int),
            _ => message,
        };
        
        Self::KlineDataParsing {
            message: enhanced_message,
            symbol,
            interval,
        }
    }

    pub fn order_data_parsing(message: impl Into<String>, order_id: Option<i64>) -> Self {
        Self::OrderDataParsing {
            message: message.into(),
            order_id,
        }
    }

    pub fn position_data_parsing(message: impl Into<String>, position_id: Option<i64>) -> Self {
        Self::PositionDataParsing {
            message: message.into(),
            position_id,
        }
    }

    pub fn deal_data_parsing(message: impl Into<String>, deal_id: Option<i64>) -> Self {
        Self::DealDataParsing {
            message: message.into(),
            deal_id,
        }
    }

    pub fn account_info_parsing(message: impl Into<String>, account_id: Option<i32>) -> Self {
        Self::AccountInfoParsing {
            message: message.into(),
            account_id,
        }
    }

    pub fn array_parsing(actual_type: impl Into<String>, context: impl Into<String>) -> Self {
        Self::ArrayParsing {
            actual_type: actual_type.into(),
            context: context.into(),
        }
    }

    pub fn invalid_kline_array_format(length: usize, data: impl Into<String>) -> Self {
        Self::InvalidKlineArrayFormat {
            length,
            data: data.into(),
        }
    }

    pub fn type_conversion(
        field: impl Into<String>, 
        message: impl Into<String>,
        value: Option<String>
    ) -> Self {
        Self::TypeConversion {
            field: field.into(),
            message: message.into(),
            value,
        }
    }

    pub fn data_validation(
        message: impl Into<String>,
        field: Option<String>,
        value: Option<String>
    ) -> Self {
        Self::DataValidation {
            message: message.into(),
            field,
            value,
        }
    }

    pub fn enum_parsing(
        field: impl Into<String>, 
        variant: impl Into<String>,
        valid_variants: Vec<String>
    ) -> Self {
        Self::EnumParsing {
            field: field.into(),
            variant: variant.into(),
            valid_variants,
        }
    }

    pub fn stream_data_format(
        message: impl Into<String>,
        expected_format: Option<String>,
        actual_data: Option<String>
    ) -> Self {
        Self::StreamDataFormat {
            message: message.into(),
            expected_format,
            actual_data,
        }
    }

    pub fn timestamp_conversion(message: impl Into<String>, timestamp: Option<i64>) -> Self {
        Self::TimestampConversion {
            message: message.into(),
            timestamp,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

// Conversion from common error types
impl From<String> for DataProcessorError {
    fn from(err: String) -> Self {
        Self::Internal(err)
    }
}

impl From<&str> for DataProcessorError {
    fn from(err: &str) -> Self {
        Self::Internal(err.to_string())
    }
}

// Helper trait for adding context to errors
pub trait DataProcessorErrorContext<T> {
    fn with_context<F>(self, f: F) -> Result<T, DataProcessorError>
    where
        F: FnOnce() -> String;

    fn with_field_context(self, field: &str) -> Result<T, DataProcessorError>;
}

impl<T, E> DataProcessorErrorContext<T> for Result<T, E>
where
    E: Into<DataProcessorError>,
{
    fn with_context<F>(self, f: F) -> Result<T, DataProcessorError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            let context = f();
            DataProcessorError::Internal(format!("{}: {}", context, base_error))
        })
    }

    fn with_field_context(self, field: &str) -> Result<T, DataProcessorError> {
        self.map_err(|e| {
            let base_error = e.into();
            DataProcessorError::Internal(format!("Field '{}': {}", field, base_error))
        })
    }
}

// Implement the StarRiverErrorTrait for DataProcessorError
impl crate::error::error_trait::StarRiverErrorTrait for DataProcessorError {
    fn get_prefix(&self) -> &'static str {
        self.get_prefix()
    }
    
    fn error_code(&self) -> ErrorCode {
        self.error_code()
    }
    
    fn category(&self) -> &'static str {
        "data_processor"
    }
    
    fn is_retriable(&self) -> bool {
        matches!(self,
            DataProcessorError::StreamProcessing { .. } |
            DataProcessorError::StreamDataFormat { .. }
        )
    }
    
    fn is_client_error(&self) -> bool {
        matches!(self,
            DataProcessorError::MissingField { .. } |
            DataProcessorError::InvalidFieldType { .. } |
            DataProcessorError::DataValidation { .. } |
            DataProcessorError::EnumParsing { .. } |
            DataProcessorError::InvalidKlineArrayFormat { .. }
        )
    }
    
    fn message(&self) -> &str {
        match self {
            DataProcessorError::JsonParsing(_) => "JSON parsing failed",
            DataProcessorError::StreamProcessing { message, .. } |
            DataProcessorError::KlineDataParsing { message, .. } |
            DataProcessorError::OrderDataParsing { message, .. } |
            DataProcessorError::PositionDataParsing { message, .. } |
            DataProcessorError::DealDataParsing { message, .. } |
            DataProcessorError::AccountInfoParsing { message, .. } |
            DataProcessorError::TypeConversion { message, .. } |
            DataProcessorError::DataValidation { message, .. } |
            DataProcessorError::StreamDataFormat { message, .. } |
            DataProcessorError::TimestampConversion { message, .. } |
            DataProcessorError::Internal(message) => message,
            DataProcessorError::MissingField { field, .. } => {
                // Can't return reference to temporary string, use static message
                "Required field missing"
            },
            DataProcessorError::InvalidFieldType { field, .. } => {
                "Invalid field type"
            },
            DataProcessorError::ArrayParsing { .. } => {
                "Array parsing failed"
            },
            DataProcessorError::InvalidKlineArrayFormat { .. } => {
                "Invalid kline array format"
            },
            DataProcessorError::EnumParsing { .. } => {
                "Enum parsing failed"
            },
        }
    }
    
    fn context(&self) -> Vec<(&'static str, String)> {
        match self {
            DataProcessorError::StreamProcessing { data_type: Some(dt), .. } => {
                vec![("data_type", dt.clone())]
            },
            DataProcessorError::MissingField { field, context } => {
                let mut ctx = vec![("field", field.clone())];
                if let Some(c) = context {
                    ctx.push(("context", c.clone()));
                }
                ctx
            },
            DataProcessorError::InvalidFieldType { field, expected, actual, context } => {
                let mut ctx = vec![
                    ("field", field.clone()),
                    ("expected", expected.clone()),
                    ("actual", actual.clone())
                ];
                if let Some(c) = context {
                    ctx.push(("context", c.clone()));
                }
                ctx
            },
            DataProcessorError::KlineDataParsing { symbol, interval, .. } => {
                let mut ctx = vec![];
                if let Some(sym) = symbol {
                    ctx.push(("symbol", sym.clone()));
                }
                if let Some(int) = interval {
                    ctx.push(("interval", int.clone()));
                }
                ctx
            },
            DataProcessorError::OrderDataParsing { order_id, .. } => {
                if let Some(id) = order_id {
                    vec![("order_id", id.to_string())]
                } else {
                    vec![]
                }
            },
            DataProcessorError::PositionDataParsing { position_id, .. } => {
                if let Some(id) = position_id {
                    vec![("position_id", id.to_string())]
                } else {
                    vec![]
                }
            },
            DataProcessorError::DealDataParsing { deal_id, .. } => {
                if let Some(id) = deal_id {
                    vec![("deal_id", id.to_string())]
                } else {
                    vec![]
                }
            },
            DataProcessorError::AccountInfoParsing { account_id, .. } => {
                if let Some(id) = account_id {
                    vec![("account_id", id.to_string())]
                } else {
                    vec![]
                }
            },
            DataProcessorError::EnumParsing { field, variant, .. } => {
                vec![("field", field.clone()), ("variant", variant.clone())]
            },
            DataProcessorError::TimestampConversion { timestamp, .. } => {
                if let Some(ts) = timestamp {
                    vec![("timestamp", ts.to_string())]
                } else {
                    vec![]
                }
            },
            _ => vec![],
        }
    }
}

// Implement ErrorContext trait for DataProcessorError
impl<T> crate::error::error_trait::ErrorContext<T, DataProcessorError> for Result<T, DataProcessorError> {
    fn with_context<F>(self, f: F) -> Result<T, DataProcessorError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let context = f();
            DataProcessorError::Internal(format!("{}: {}", context, e))
        })
    }
    
    fn with_operation_context(self, operation: &str) -> Result<T, DataProcessorError> {
        self.map_err(|e| {
            DataProcessorError::Internal(format!("Data Processing Operation '{}': {}", operation, e))
        })
    }
    
    fn with_resource_context(self, resource_type: &str, resource_id: &str) -> Result<T, DataProcessorError> {
        self.map_err(|e| {
            DataProcessorError::Internal(format!("Data Processing {} '{}': {}", resource_type, resource_id, e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = DataProcessorError::missing_field("symbol", Some("kline data".to_string()));
        assert!(err.to_string().contains("symbol"));
        assert!(err.to_string().contains("missing"));
    }

    #[test]
    fn test_kline_parsing_error() {
        let err = DataProcessorError::kline_data_parsing(
            "Invalid format", 
            Some("EURUSD".to_string()), 
            Some("M1".to_string())
        );
        assert!(err.to_string().contains("EURUSD"));
        assert!(err.to_string().contains("Invalid format"));
    }

    #[test]
    fn test_array_format_error() {
        let err = DataProcessorError::invalid_kline_array_format(3, "[1, 2, 3]".to_string());
        assert!(err.to_string().contains("expected 6 elements"));
        assert!(err.to_string().contains("got 3"));
    }

    #[test]
    fn test_enum_parsing_error() {
        let err = DataProcessorError::enum_parsing(
            "order_state",
            "INVALID_STATE",
            vec!["PENDING".to_string(), "FILLED".to_string()]
        );
        assert!(err.to_string().contains("order_state"));
        assert!(err.to_string().contains("INVALID_STATE"));
    }

    #[test]
    fn test_error_context() {
        let result: Result<i32, serde_json::Error> = Err(serde_json::Error::io(
            std::io::Error::new(std::io::ErrorKind::Other, "test error")
        ));
        
        let err = result.with_context(|| "Processing order data".to_string()).unwrap_err();
        assert!(err.to_string().contains("Processing order data"));
    }
}