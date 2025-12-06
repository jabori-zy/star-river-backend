


#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct KlineKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

impl From<KlineKey> for Key {
    fn from(kline_key: KlineKey) -> Self {
        Key::Kline(kline_key)
    }
}

//TODO: Handle later. With this change KlineKey::get_key_str still emits short keys like "kline|binance|BTCUSDT|1m" whenever no start/end time
// is set, but the new FromStr implementation now hard-requires exactly six pipe-delimited fields.
impl FromStr for KlineKey {
    type Err = StarRiverError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 6 {
            return Err(InvalidKeyFormatSnafu { key_str: s.to_string() }.build());
        }

        let exchange = parts[1].parse::<Exchange>()?;
        let symbol = parts[2].to_string();
        let interval = parts[3].parse::<KlineInterval>().context(ParseKlineIntervalFailedSnafu {
            interval: parts[3].to_string(),
        })?;
        // Use Box::leak to convert string to static reference
        let start_time = Some(parts[4].to_string());
        let end_time = Some(parts[5].to_string());
        Ok(KlineKey::new(exchange, symbol, interval, start_time, end_time))
    }
}

impl KlineKey {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval, start_time: Option<String>, end_time: Option<String>) -> Self {
        Self {
            exchange,
            symbol,
            interval,
            start_time,
            end_time,
        }
    }

    pub fn replace_time_range(&mut self, time_range: TimeRange) {
        self.start_time = Some(time_range.start_date.to_string());
        self.end_time = Some(time_range.end_date.to_string());
    }
}

impl KeyTrait for KlineKey {
    fn get_key_str(&self) -> String {
        match (&self.start_time, &self.end_time) {
            (Some(start_time), Some(end_time)) => {
                format!(
                    "kline|{}|{}|{}|{}|{}",
                    self.exchange.to_string(),
                    self.symbol,
                    self.interval.to_string(),
                    start_time.clone(),
                    end_time.clone()
                )
            }
            _ => {
                format!("kline|{}|{}|{}", self.exchange.to_string(), self.symbol, self.interval.to_string())
            }
        }
    }
    fn get_exchange(&self) -> Exchange {
        self.exchange.clone()
    }
    fn get_symbol(&self) -> String {
        self.symbol.clone()
    }
    fn get_interval(&self) -> KlineInterval {
        self.interval.clone()
    }
    fn get_time_range(&self) -> Option<TimeRange> {
        match (&self.start_time, &self.end_time) {
            (Some(start_time), Some(end_time)) => Some(TimeRange::new(start_time.clone(), end_time.clone())),
            _ => None,
        }
    }
}