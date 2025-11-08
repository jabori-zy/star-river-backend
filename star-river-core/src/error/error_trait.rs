use super::ErrorCode;
use std::error::Error;
use axum::http::StatusCode;
use strum::Display;


#[derive(Debug, Clone, Display)]
#[strum(serialize_all = "lowercase")]
pub enum ErrorLanguage {
    English,
    Chinese,
}

/// Trait that all errors in the Star River Backend system should implement
/// This ensures consistent error handling patterns across all components
///
/// Note: With snafu, context is handled natively through error variants
/// No separate ErrorContext trait is needed
pub trait StarRiverErrorTrait: Error + Send + Sync + 'static {
    /// Returns the error prefix for this error type (e.g., "MT5", "DATA_PROCESSOR")
    fn get_prefix(&self) -> &'static str;

    /// Returns a string error code in format "PREFIX_NNNN" (e.g., "MT5_1001")
    fn error_code(&self) -> ErrorCode;

    /// Determines whether the error represents a recoverable condition
    /// Returns true if the operation that caused this error can potentially be retried
    /// Returns false if the error indicates a permanent failure that should not be retried
    fn is_recoverable(&self) -> bool {
        false
    }

    /// Returns localized error message based on the specified language
    /// For English, it returns the Display trait message (from snafu display)
    /// For other languages, it should return the localized version
    fn error_message(&self, language: ErrorLanguage) -> String;



    /// Returns the HTTP status code for this error
    /// Default implementation returns INTERNAL_SERVER_ERROR
    fn http_status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    /// Returns the error code chain from the root cause to this error
    /// For leaf errors (no source), returns [self.error_code()]
    /// For errors with source, returns [root_error_code, ..., parent_error_code, self.error_code()]
    /// Example: A->B->C would return [A_code, B_code, C_code] where A is the root cause
    fn error_code_chain(&self) -> Vec<ErrorCode> {
        // Default implementation for leaf errors (no source)
        vec![self.error_code()]
    }
}
