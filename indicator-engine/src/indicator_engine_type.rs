use key::{IndicatorKey, KlineKey};
use star_river_core::{exchange::Exchange, kline::KlineInterval};
use ta_lib::IndicatorConfig;

// 指标的订阅键
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IndicatorSubKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator_config: IndicatorConfig,
}

impl From<IndicatorSubKey> for IndicatorKey {
    fn from(indicator_sub_key: IndicatorSubKey) -> Self {
        let kline_key = KlineKey::new(
            indicator_sub_key.exchange,
            indicator_sub_key.symbol,
            indicator_sub_key.interval,
            None,
            None,
        );
        IndicatorKey::new(kline_key, indicator_sub_key.indicator_config)
    }
}
