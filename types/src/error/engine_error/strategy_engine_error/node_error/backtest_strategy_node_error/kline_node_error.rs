use snafu::{Snafu, Backtrace};
use std::collections::HashMap;
use crate::error::ErrorCode;
use std::sync::Arc;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum KlineNodeError {

    #[snafu(display("{node_name}({node_id}) register exchange error"))]
    RegisterExchange {
        node_id: String,
        node_name: String,
        #[snafu(source(from(Arc<dyn std::error::Error + Send + Sync + 'static>, Arc::new)))]
        source: Arc<dyn std::error::Error + Send + Sync + 'static>,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for Mt5Error
impl crate::error::error_trait::StarRiverErrorTrait for KlineNodeError {
    fn get_prefix(&self) -> &'static str {
        "MT5"
    }
    
    fn error_code(&self) -> ErrorCode {
            let prefix = self.get_prefix();
            let code = match self {
                // HTTP and JSON errors (1001-1004)
                KlineNodeError::RegisterExchange { .. } => 1001,
            };   

            format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx 
    }

    fn is_recoverable(&self) -> bool {
        matches!(self,
            KlineNodeError::RegisterExchange { .. }
        )
    }
}