use strum::Display;



#[derive(Display, Debug, Clone)]
pub(crate) enum BinanceHttpUrl {
    #[strum(serialize = "https://api.binance.com")]
    BaseUrl,
    #[strum(serialize = "/api/v3/ticker/price")]
    PriceTicker,
    #[strum(serialize = "/api/v3/klines")]
    Kline,
    #[strum(serialize = "/api/v3/time")]
    ServerTime,
    #[strum(serialize = "/api/v3/exchangeInfo")]
    ExchangeInfo,
    #[strum(serialize = "/api/v3/ping")]
    Ping,
}

#[derive(Display, Debug, Clone)]
pub(crate) enum BinanceWsUrl {
    #[strum(serialize = "wss://stream.binance.com:9443/stream")]
    BaseUrl,
}

