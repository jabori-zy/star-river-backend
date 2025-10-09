
use crate::binance::url::BinanceHttpUrl;
use crate::utils::get_utc8_timestamp;
use super::{BinanceKlineInterval};
use super::{
    BinanceError,
    PingFailedSnafu,
    NetworkSnafu,
    ResponseSnafu,
    ParseServerTimeFailedSnafu,
};
use snafu::ResultExt;

#[derive(Clone, Debug)]

pub struct BinanceHttpClient {
    client: reqwest::Client
}

impl BinanceHttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new()
        }
    }

    pub async fn ping(&mut self) -> Result<(), BinanceError> {
        let url = format!("{}{}", BinanceHttpUrl::BaseUrl, BinanceHttpUrl::Ping);

        let result = self.client
            .get(&url)
            .send()
            .await
            .context(NetworkSnafu {
                url: url.clone(),
            })?
            .json::<serde_json::Value>()
            .await
            .context(ResponseSnafu {
                url: url.clone(),
            })?;
        // 如果body为空，则认为连接成功
        if result == "{}" {
            Ok(())
        } else {
            return Err(PingFailedSnafu {}.build());
        }
    }

    pub async fn get_server_time(&self) -> Result<i64, BinanceError> {
        let url = format!("{}{}", BinanceHttpUrl::BaseUrl, BinanceHttpUrl::ServerTime);
        let result = self.client
            .get(&url)
            .send()
            .await
            .context(NetworkSnafu {
                url: url.clone(),
            })?
            .text()
            .await.context(ResponseSnafu {
                url: url.clone(),
            })?;

        // 解析JSON字符串
        let result: serde_json::Value = serde_json::from_str(&result).context(ParseServerTimeFailedSnafu {})?;

        // 提取时间戳
        Ok(result
            .get("serverTime")
            .and_then(|v| v.as_i64())
            .unwrap())
    }

    pub async fn get_ticker_price(&self, symbol: &str) -> Result<serde_json::Value, String> {
        // 构建url
        let url = format!("{}{}?symbol={}", BinanceHttpUrl::BaseUrl, BinanceHttpUrl::PriceTicker, symbol);

        // 获取ticker price
        let mut tick_price = self
            .client
            .get(&url)
            .header("X-MBX-TIME-UNIT", "MILLISECOND")
            .send()
            .await
            .expect("获取ticker price失败")
            .json::<serde_json::Value>()
            .await
            .expect("解析ticker price失败");
        // 设置时间戳
        tick_price["timestamp"] = serde_json::Value::Number(get_utc8_timestamp().into());

        Ok(tick_price)
    }

    pub async fn get_kline(
        &self,
        symbol: &str,
        interval: BinanceKlineInterval,
        limit: Option<u32>,
        start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> Result<Vec<serde_json::Value>, BinanceError> {
        // 如果limit为空，则设置为1000
        let limit = limit.unwrap_or(1000);
        // 如果start_time或end_time为空，则不传时间参数

        let url = if start_time.is_none() || end_time.is_none() {
            format!(
                "{}{}?symbol={}&interval={}&limit={}",
                BinanceHttpUrl::BaseUrl,
                BinanceHttpUrl::Kline,
                symbol,
                interval,
                limit
            )
        } else {
            format!(
                "{}{}?symbol={}&interval={}&limit={}&startTime={}&endTime={}",
                BinanceHttpUrl::BaseUrl,
                BinanceHttpUrl::Kline,
                symbol,
                interval,
                limit,
                start_time.unwrap(),
                end_time.unwrap()
            )
        };

        let raw_kline = self
            .client
            .get(&url)
            .send()
            .await
            .context(NetworkSnafu {
                url: url.clone(),
            })?
            .json::<Vec<serde_json::Value>>()
            .await
            .context(ResponseSnafu {
                url: url.clone(),
            })?;

        // log::debug!("kline: {:?}", raw_kline);

        Ok(raw_kline)
    }

    pub async fn get_exchange_info(&self) -> Result<serde_json::Value, BinanceError> {
        let url = format!("{}{}", BinanceHttpUrl::BaseUrl, BinanceHttpUrl::ExchangeInfo);
        let response = self.client
            .get(&url)
            .send()
            .await
            .context(NetworkSnafu {
                url: url.clone(),
            })?
            .json::<serde_json::Value>()
            .await
            .context(ResponseSnafu {
                url: url.clone(),
            })?;

        Ok(response)
    }
}
