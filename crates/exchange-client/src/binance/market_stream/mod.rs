pub mod kline;

use crate::binance::BinanceKlineInterval;
use crate::binance::market_stream::kline::KlineStream;




pub fn klines(symbol: &str, interval: BinanceKlineInterval) -> KlineStream {
    KlineStream::new(symbol, interval)
}
