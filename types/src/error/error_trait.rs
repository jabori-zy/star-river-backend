use std::error::Error;
use super::ErrorCode;

/// Trait that all errors in the Star River Backend system should implement
/// This ensures consistent error handling patterns across all components
pub trait StarRiverErrorTrait: Error + Send + Sync + 'static {
    /// Returns the error prefix for this error type (e.g., "MT5_HTTP_CLIENT", "DATA_PROCESSOR")
    fn get_prefix(&self) -> &'static str;
    
    /// Returns a string error code in format "PREFIX_NNNN" (e.g., "MT5_HTTP_CLIENT_1001")
    fn error_code(&self) -> ErrorCode;
    
    /// Extract structured context information from the error
    /// Returns key-value pairs for logging, monitoring, and debugging
    fn context(&self) -> Vec<(&'static str, String)> {
        vec![]
    }
}

/// Helper trait for adding context to error results
pub trait ErrorContext<T, E>
where
    E: StarRiverErrorTrait,
{
    /// Add contextual information to an error result
    fn with_context<F>(self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> String;
    
    /// Add operation context to an error result
    fn with_operation_context(self, operation: &str) -> Result<T, E>;
    
    /// Add resource context to an error result (symbol, order_id, etc.)
    fn with_resource_context(self, resource_type: &str, resource_id: &str) -> Result<T, E>;
}