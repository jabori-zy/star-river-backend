use std::error::Error;
use std::collections::HashMap;
use super::ErrorCode;

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
    
    /// Extract structured context information from the error
    /// Returns a HashMap with field names as keys and context values as values
    /// for logging, monitoring, and debugging
    fn context(&self) -> HashMap<&'static str, String> {
        HashMap::new()
    }
    
    /// Determines whether the error represents a recoverable condition
    /// Returns true if the operation that caused this error can potentially be retried
    /// Returns false if the error indicates a permanent failure that should not be retried
    fn is_recoverable(&self) -> bool;

}