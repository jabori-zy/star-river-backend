use types::market::{Exchange, KlineInterval};
use types::indicator::IndicatorConfig;
use types::cache::cache_key::IndicatorCacheKey;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IndicatorSubKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator_config: IndicatorConfig,
}

impl From<IndicatorSubKey> for IndicatorCacheKey {
    fn from(indicator_sub_key: IndicatorSubKey) -> Self {
        IndicatorCacheKey::new(indicator_sub_key.exchange, indicator_sub_key.symbol, indicator_sub_key.interval, indicator_sub_key.indicator_config)
    }
}
