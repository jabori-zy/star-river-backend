use serde::{Deserialize, Serialize};
use types::market::KlineInterval;
use types::strategy::{BacktestDataSource, SelectedAccount, DataSourceExchange};
use std::str::FromStr;
use types::strategy::TimeRange;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveDataNodeLiveConfig {
    #[serde(rename = "selectedLiveAccount")]
    pub selected_live_account: SelectedAccount,
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_kline_interval")]
    pub interval: KlineInterval,
    // pub frequency: u32,
}

fn deserialize_kline_interval<'de, D>(deserializer: D) -> Result<KlineInterval, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // 将字符串反序列化为String
    let s = String::deserialize(deserializer)?;
    
    // 使用as_str()方法获取&str，然后传递给from_str
    match KlineInterval::from_str(s.as_str()) {
        Ok(interval) => Ok(interval),
        Err(e) => Err(serde::de::Error::custom(format!("无法解析KlineInterval: {}", e)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineNodeBacktestConfig {
    #[serde(rename = "dataSource")]
    pub data_source: BacktestDataSource,
    #[serde(rename = "fileConfig")]
    pub file_config: Option<FileConfig>,
    #[serde(rename = "exchangeConfig")]
    pub exchange_config: Option<ExchangeConfig>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    #[serde(rename = "filePath")]
    pub file_path: String,
}
 


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    #[serde(rename = "selectedDataSource")]
    pub selected_data_source: DataSourceExchange,
    pub symbol: String,
    pub interval: KlineInterval,
    #[serde(rename = "timeRange")]
    pub time_range: TimeRange,
}
