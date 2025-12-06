use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct Symbol {
    pub name: String,
    pub base: Option<String>,
    pub quote: Option<String>,
    pub exchange: String,
    #[schema(value_type = f32, example = "2024-01-01T12:00:00Z")]
    pub point: ordered_float::OrderedFloat<f32>,
}

impl Symbol {
    /// Create a new Symbol from base/quote and exchange
    pub fn new(name: &str, base: Option<&str>, quote: Option<&str>, exchange: String, point: f32) -> Self {
        Self {
            name: name.to_string(),
            base: base.map(|s| s.to_string()),
            quote: quote.map(|s| s.to_string()),
            exchange,
            point: ordered_float::OrderedFloat::from(point),
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

    pub fn point(&self) -> f32 {
        self.point.into_inner()
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
