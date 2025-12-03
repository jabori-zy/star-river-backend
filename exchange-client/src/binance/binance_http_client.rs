use exchange_core::exchange_trait::HttpClient;
use snafu::ResultExt;

use super::error::*;
use crate::binance::{binance_type::BinanceKlineInterval, url::BinanceHttpUrl};

#[derive(Clone, Debug)]

pub struct BinanceHttpClient {
    client: reqwest::Client,
}

impl HttpClient for BinanceHttpClient {}

impl BinanceHttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn ping(&self) -> Result<(), BinanceError> {
        let url = format!("{}{}", BinanceHttpUrl::BaseUrl, BinanceHttpUrl::Ping);
        tracing::debug!("ping url: {:?}", url);

        let result = self
            .client
            .get(&url)
            .send()
            .await
            .context(NetworkSnafu { url: url.clone() })?
            .text()
            .await
            .context(ResponseSnafu { url: url.clone() })?;
        tracing::debug!("ping result: {:?}", result);
        // 如果body为空，则认为连接成功
        if result == "{}" {
            Ok(())
        } else {
            return Err(PingFailedSnafu {}.build());
        }
    }

    // pub async fn get_server_time(&self) -> Result<i64, BinanceError> {
    //     let url = format!("{}{}", BinanceHttpUrl::BaseUrl, BinanceHttpUrl::ServerTime);
    //     let result = self
    //         .client
    //         .get(&url)
    //         .send()
    //         .await
    //         .context(NetworkSnafu { url: url.clone() })?
    //         .text()
    //         .await
    //         .context(ResponseSnafu { url: url.clone() })?;

    //     // 解析JSON字符串
    //     let result: serde_json::Value = serde_json::from_str(&result).context(ParseServerTimeFailedSnafu {})?;

    //     // 提取时间戳
    //     Ok(result.get("serverTime").and_then(|v| v.as_i64()).unwrap())
    // }

    // pub async fn get_ticker_price(&self, symbol: &str) -> Result<serde_json::Value, String> {
    //     // 构建url
    //     let url = format!("{}{}?symbol={}", BinanceHttpUrl::BaseUrl, BinanceHttpUrl::PriceTicker, symbol);

    //     // 获取ticker price
    //     let mut tick_price = self
    //         .client
    //         .get(&url)
    //         .header("X-MBX-TIME-UNIT", "MILLISECOND")
    //         .send()
    //         .await
    //         .expect("获取ticker price失败")
    //         .json::<serde_json::Value>()
    //         .await
    //         .expect("解析ticker price失败");
    //     // 设置时间戳
    //     tick_price["timestamp"] = serde_json::Value::Number(get_utc8_timestamp().into());

    //     Ok(tick_price)
    // }

    pub async fn get_spot_kline(
        &self,
        symbol: &str,
        interval: BinanceKlineInterval,
        limit: Option<u32>,
        start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> Result<Vec<serde_json::Value>, BinanceError> {
        // 如果limit为空，则设置为1000
        let limit = limit.unwrap_or(1000).min(1000);
        // 如果start_time或end_time为空，则不传时间参数

        let url = if let (Some(start_time), Some(end_time)) = (start_time, end_time) {
            format!(
                "{}{}?symbol={}&interval={}&limit={}&startTime={}&endTime={}",
                BinanceHttpUrl::BaseUrl,
                BinanceHttpUrl::SpotKline,
                symbol,
                interval,
                limit,
                start_time,
                end_time
            )
        } else {
            format!(
                "{}{}?symbol={}&interval={}&limit={}",
                BinanceHttpUrl::BaseUrl,
                BinanceHttpUrl::SpotKline,
                symbol,
                interval,
                limit,
            )
        };

        let raw_kline = self
            .client
            .get(&url)
            .send()
            .await
            .context(NetworkSnafu { url: url.clone() })?
            .json::<Vec<serde_json::Value>>()
            .await
            .context(ResponseSnafu { url: url.clone() })?;

        // log::debug!("kline: {:?}", raw_kline);

        Ok(raw_kline)
    }

    pub async fn get_exchange_info(&self) -> Result<serde_json::Value, BinanceError> {
        let url = format!("{}{}", BinanceHttpUrl::BaseUrl, BinanceHttpUrl::ExchangeInfo);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context(NetworkSnafu { url: url.clone() })?
            .json::<serde_json::Value>()
            .await
            .context(ResponseSnafu { url: url.clone() })?;

        Ok(response)
    }

    pub async fn get_spot_symbol_info(&self, symbol: &str) -> Result<serde_json::Value, BinanceError> {
        let url = format!("{}{}?symbol={}", BinanceHttpUrl::BaseUrl, BinanceHttpUrl::ExchangeInfo, symbol);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context(NetworkSnafu { url: url.clone() })?
            .json::<serde_json::Value>()
            .await
            .context(ResponseSnafu { url: url.clone() })?;
        Ok(response)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     fn init_logger() {
//         let _ = tracing_subscriber::fmt()
//             .with_max_level(tracing::Level::DEBUG)
//             .with_test_writer()
//             .try_init();
//     }

//     #[tokio::test]
//     async fn test_ping() {
//         init_logger();
//         let mut client = BinanceHttpClient::new();
//         let result = client.ping().await;
//         assert!(result.is_ok(), "Ping should succeed");
//     }

//     #[tokio::test]
//     async fn test_get_server_time() {
//         init_logger();
//         let client = BinanceHttpClient::new();
//         let result = client.get_server_time().await;
//         assert!(result.is_ok(), "Get server time should succeed");
//         let timestamp = result.unwrap();
//         assert!(timestamp > 0, "Timestamp should be positive");
//     }

//     #[tokio::test]
//     async fn test_get_ticker_price() {
//         init_logger();
//         let client = BinanceHttpClient::new();
//         let result = client.get_ticker_price("BTCUSDT").await;
//         assert!(result.is_ok(), "Get ticker price should succeed");
//         let price = result.unwrap();
//         assert!(price.get("symbol").is_some(), "Response should contain symbol");
//         assert!(price.get("price").is_some(), "Response should contain price");
//     }

//     #[tokio::test]
//     async fn test_get_kline_with_limit() {
//         let client = BinanceHttpClient::new();
//         let result = client.get_kline(
//             "BTCUSDT",
//             BinanceKlineInterval::Minutes1,
//             Some(10),
//             None,
//             None
//         ).await;
//         assert!(result.is_ok(), "Get kline should succeed");
//         let klines = result.unwrap();
//         assert_eq!(klines.len(), 10, "Should return 10 klines");
//     }

//     #[tokio::test]
//     async fn test_get_kline_with_time_range() {
//         let client = BinanceHttpClient::new();
//         let end_time = chrono::Utc::now().timestamp_millis() as u64;
//         let start_time = end_time - 3600000; // 1 hour ago

//         let result = client.get_kline(
//             "ETHUSDT",
//             BinanceKlineInterval::Minutes5,
//             Some(20),
//             Some(start_time),
//             Some(end_time)
//         ).await;
//         assert!(result.is_ok(), "Get kline with time range should succeed");
//         let klines = result.unwrap();
//         assert!(klines.len() > 0, "Should return klines");
//         assert!(klines.len() <= 20, "Should not exceed limit");
//     }

//     #[tokio::test]
//     async fn test_get_kline_different_intervals() {
//         let client = BinanceHttpClient::new();

//         let intervals = vec![
//             BinanceKlineInterval::Minutes1,
//             BinanceKlineInterval::Minutes15,
//             BinanceKlineInterval::Hours1,
//             BinanceKlineInterval::Days1,
//         ];

//         for interval in intervals {
//             let result = client.get_kline(
//                 "BTCUSDT",
//                 interval.clone(),
//                 Some(5),
//                 None,
//                 None
//             ).await;
//             assert!(result.is_ok(), "Get kline for {:?} should succeed", interval);
//         }
//     }

//     #[tokio::test]
//     async fn test_get_exchange_info() {
//         let client = BinanceHttpClient::new();
//         let result = client.get_exchange_info().await;
//         assert!(result.is_ok(), "Get exchange info should succeed");
//         let info = result.unwrap();
//         assert!(info.get("symbols").is_some(), "Response should contain symbols");
//         assert!(info.get("symbols").unwrap().is_array(), "Symbols should be an array");
//     }

//     #[tokio::test]
//     async fn test_get_symbol_info() {
//         let client = BinanceHttpClient::new();
//         let result = client.get_symbol_info("BTCUSDT").await;
//         assert!(result.is_ok(), "Get symbol info should succeed");
//         let info = result.unwrap();
//         assert!(info.get("symbols").is_some(), "Response should contain symbols");
//         println!("symbol info: {:?}", info);
//     }

//     #[tokio::test]
//     async fn test_get_symbol_info_invalid_symbol() {
//         let client = BinanceHttpClient::new();
//         let result = client.get_symbol_info("INVALIDXYZ").await;
//         // Should either succeed with empty result or fail gracefully
//         if let Ok(info) = result {
//             let symbols = info.get("symbols").and_then(|s| s.as_array());
//             if let Some(symbols) = symbols {
//                 assert_eq!(symbols.len(), 0, "Invalid symbol should return empty array");
//             }
//         }
//     }

//     #[tokio::test]
//     async fn test_get_kline_invalid_symbol() {
//         let client = BinanceHttpClient::new();
//         let result = client.get_kline(
//             "INVALIDXYZ123",
//             BinanceKlineInterval::Minutes1,
//             Some(5),
//             None,
//             None
//         ).await;
//         // Should fail or return empty
//         if let Ok(klines) = result {
//             assert_eq!(klines.len(), 0, "Invalid symbol should return empty klines");
//         }
//     }

//     #[tokio::test]
//     async fn test_get_kline_7_days_ago_start_time_no_end_time() {
//         init_logger();
//         let client = BinanceHttpClient::new();

//         // 传入7天前的开始时间，不传入结束时间
//         let start_time = (chrono::Utc::now().timestamp_millis() - 7 * 24 * 3600000) as u64; // 7 days ago

//         let result = client.get_kline(
//             "BTCUSDT",
//             BinanceKlineInterval::Minutes1,
//             None,
//             Some(start_time),
//             None  // 不传入结束时间
//         ).await;

//         // 根据实现逻辑（第109行），如果start_time或end_time任一为None，则不会传递时间参数
//         // 这种情况下会忽略start_time，返回最近的K线数据
//         assert!(result.is_ok(), "Get kline with 7 days ago start time should succeed");
//         let klines = result.unwrap();
//         assert!(klines.len() > 0, "Should return klines");

//         println!("7天前开始时间（无结束时间）返回K线数量: {}", klines.len());
//         println!("第一根k线的时间: {:?}", klines[0]);
//         println!("最后一根k线的时间: {:?}", klines[klines.len() - 1]);
//     }
// }
