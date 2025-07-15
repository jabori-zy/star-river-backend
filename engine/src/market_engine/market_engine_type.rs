use types::market::{Exchange, KlineInterval};
use types::cache::key::KlineKey;



#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KlineSubKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
}


impl From<KlineSubKey> for KlineKey {
    fn from(kline_sub_key: KlineSubKey) -> Self {
        KlineKey::new(kline_sub_key.exchange, kline_sub_key.symbol, kline_sub_key.interval)
    }
}


