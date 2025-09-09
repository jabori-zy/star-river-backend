use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

use super::Exchange;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct Symbol {
    pub name: String,
    pub base: Option<String>,
    pub quote: Option<String>,
    pub exchange: Exchange,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SymbolError {
    #[error("Invalid symbol format")]
    InvalidFormat,

    #[error("Unknown symbol pair: {0}")]
    UnknownPair(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

impl SymbolError {
    pub fn unknown_pair<S: Into<String>>(pair: S) -> Self {
        Self::UnknownPair(pair.into())
    }

    pub fn parse_error<S: Into<String>>(message: S) -> Self {
        Self::ParseError(message.into())
    }
}

impl Symbol {
    /// Create a new Symbol from base/quote and exchange
    pub fn new(name: &str, base: Option<&str>, quote: Option<&str>, exchange: Exchange) -> Self {
        Self {
            name: name.to_string(),
            base: base.map(|s| s.to_string()),
            quote: quote.map(|s| s.to_string()),
            exchange,
        }
    }

    /// Convert to specific exchange format

    /// Get base currency
    pub fn base(&self) -> Option<&str> {
        self.base.as_deref()
    }

    /// Get quote currency
    pub fn quote(&self) -> Option<&str> {
        self.quote.as_deref()
    }

    // /// Check if this is a BTC pair
    // pub fn is_btc_pair(&self) -> bool {
    //     self.base == "BTC" || self.quote == "BTC"
    // }

    // /// Check if this is a USDT pair
    // pub fn is_usdt_pair(&self) -> bool {
    //     self.quote == "USDT"
    // }
}
