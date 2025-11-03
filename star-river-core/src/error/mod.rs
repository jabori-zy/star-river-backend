pub mod datetime_error;
// pub mod engine_error;
pub mod error_trait;
// pub mod exchange_client_error;
pub mod indicator_error;
pub mod star_river_error;
// pub mod virtual_trading_system_error;
// pub mod node_error;
// pub mod strategy_error;

pub use axum::http::StatusCode;

pub type ErrorCode = String;

pub use error_trait::{StarRiverErrorTrait, ErrorLanguage};

pub use reqwest::Error as ReqwestError;

// pub use exchange_client_error::*;




pub fn generate_error_code_chain(source: &dyn StarRiverErrorTrait) -> Vec<ErrorCode> {
    let mut chain = source.error_code_chain();
    chain.push(source.error_code());
    chain
}
