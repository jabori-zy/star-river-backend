use serde::{Serialize, Deserialize};

#[derive(Clone,Debug, Serialize, Deserialize)]
pub enum AccountConfig {
    MetaTrader5(MetaTrader5AccountConfig),
    Binance(BinanceAccountConfig),
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaTrader5AccountConfig {
    pub account_id: i64,
    pub password: String,
    pub server: String,
    pub terminal_path: String,
}




#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BinanceAccountConfig {
    pub api_key: String,
    pub api_secret: String,
}




