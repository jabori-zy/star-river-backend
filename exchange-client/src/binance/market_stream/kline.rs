use crate::binance::websocket::Stream;
use crate::binance::BinanceKlineInterval;


pub struct KlineStream {
    symbol: String,
    interval: BinanceKlineInterval,
}

impl KlineStream {
    pub fn new(symbol: &str, interval: BinanceKlineInterval) -> Self {
        Self { symbol: symbol.to_string().to_lowercase(), interval }
    }
    
}

impl From<KlineStream> for Stream {
    fn from(kline_stream: KlineStream) -> Self {
        Stream::new(&format!("{}@kline_{}", kline_stream.symbol, kline_stream.interval))
    }
}

