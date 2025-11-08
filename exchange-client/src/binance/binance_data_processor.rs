
use super::binance_type::{BinanceKlineRaw, BinanceSymbolRaw};
use chrono::{TimeZone, Utc};
use snafu::{OptionExt, ResultExt};
use star_river_core::exchange::Exchange;
use star_river_core::kline::Kline;
use star_river_core::instrument::Symbol;
use std::str::FromStr;
use strum::Display;
use strum::EnumString;
use crate::binance::binance_type::BinanceKlineInterval;
use exchange_core::exchange_trait::DataProcessor;
use super::data_processor_error::BinanceDataProcessorError;
use exchange_core::error::data_processor_error::*;

#[derive(Debug, Clone, Display, EnumString, Eq, PartialEq, Hash)]
pub enum BinanceStreamEvent {
    #[strum(serialize = "kline")]
    Kline,
    #[strum(serialize = "avgPrice")]
    AvgPrice,
}

#[derive(Clone, Debug)]
pub struct BinanceDataProcessor;

impl DataProcessor for BinanceDataProcessor {}

impl BinanceDataProcessor {
    // 处理k线系列
    pub async fn process_kline_series(&self, raw_data: Vec<serde_json::Value>) -> Result<Vec<Kline>, BinanceDataProcessorError> {
        let klines = raw_data
            .iter()
            .map(|v| {
                let raw: BinanceKlineRaw = serde_json::from_value(v.clone()).context(JsonParseFailedSnafu)?;

                Ok(Kline {
                    datetime: Utc
                        .timestamp_millis_opt(raw.0)
                        .single()
                        .context(TimestampConversionFailedSnafu {
                            message: "Failed to convert timestamp".to_string(),
                            timestamp: Some(raw.0),
                        })?,
                    open: raw.1.parse::<f64>().map_err(|_| {
                        InvalidFieldTypeSnafu {
                            field: "open".to_string(),
                            expected: "f64".to_string(),
                            actual: raw.1.clone(),
                        }.build()
                    })?,
                    high: raw.2.parse::<f64>().map_err(|_| {
                        InvalidFieldTypeSnafu {
                            field: "high".to_string(),
                            expected: "f64".to_string(),
                            actual: raw.2.clone(),
                        }.build()
                    })?,
                    low: raw.3.parse::<f64>().map_err(|_| {
                        InvalidFieldTypeSnafu {
                            field: "low".to_string(),
                            expected: "f64".to_string(),
                            actual: raw.3.clone(),
                        }.build()
                    })?,
                    close: raw.4.parse::<f64>().map_err(|_| {
                        InvalidFieldTypeSnafu {
                            field: "close".to_string(),
                            expected: "f64".to_string(),
                            actual: raw.4.clone(),
                        }.build()
                    })?,
                    volume: raw.5.parse::<f64>().map_err(|_| {
                        InvalidFieldTypeSnafu {
                            field: "volume".to_string(),
                            expected: "f64".to_string(),
                            actual: raw.5.clone(),
                        }.build()
                    })?,
                })
            })
            .collect::<Result<Vec<Kline>, BinanceDataProcessorError>>()?;

        Ok(klines)
    }

    // 处理k线数据并且更新缓存，并且发送事件
    // 发送两个事件
    // 1. k线缓存更新之后，推送整个k线数据到
    // 2. k线缓存大小事件
    // pub async fn process_stream_kline(&self, raw_stream: serde_json::Value, event_publisher: EventPublisher) -> Result<(), String> {
    //     let k = &raw_stream["data"]["k"];
    //     let timestamp = k["t"].as_i64().expect("解析timestamp失败");
    //     let symbol = k["s"].as_str().expect("解析symbol失败");
    //     let open = k["o"].as_str().expect("解析open失败").parse::<f64>().expect("open不是f64");
    //     let high = k["h"].as_str().expect("解析high失败").parse::<f64>().expect("high不是f64");
    //     let low = k["l"].as_str().expect("解析low失败").parse::<f64>().expect("low不是f64");
    //     let close = k["c"].as_str().expect("解析close失败").parse::<f64>().expect("close不是f64");
    //     let volume = k["v"].as_str().expect("解析volume失败").parse::<f64>().expect("volume不是f64");
    //     let interval = k["i"]
    //         .as_str()
    //         .expect("解析interval失败")
    //         .parse::<BinanceKlineInterval>()
    //         .expect("interval不是KlineInterval");
    //     let new_kline = Kline {
    //         datetime: Utc.timestamp_opt(timestamp, 0).single().unwrap(),
    //         open: open,
    //         high: high,
    //         low: low,
    //         close: close,
    //         volume: volume,
    //     };

    //     let exchange_kline_update_event_config = ExchangeKlineUpdateEvent {
    //         exchange: Exchange::Binance,
    //         symbol: symbol.to_string(),
    //         interval: interval.clone().into(),
    //         kline: new_kline,
    //         datetime: Utc::now(),
    //     };

    //     let event = ExchangeEvent::ExchangeKlineUpdate(exchange_kline_update_event_config).into();
    //     // let event_center = self.event_center.lock().await;
    //     // event_center.publish(event).expect("发送k线更新事件失败");
    //     let _ = event_publisher.publish(event);

    //     Ok(())
    // }

    // 处理stream流数据
    // pub async fn process_stream(&self, raw_stream: serde_json::Value, event_publisher: EventPublisher) {
    //     if raw_stream.get("data").is_some() {
    //         // log::info!("process_stream-binance: {:?}", raw_stream);
    //         let event = raw_stream["data"]["e"].as_str().expect("解析stream_event失败");
    //         let stream_event = BinanceStreamEvent::from_str(event).expect("转换为BinanceStreamEvent失败");

    //         match stream_event {
    //             BinanceStreamEvent::Kline => {
    //                 // tracing::debug!("stream事件为: {:?}", stream_event);
    //                 self.process_stream_kline(raw_stream, event_publisher)
    //                     .await
    //                     .expect("处理k线数据失败");
    //             }
    //             _ => {
    //                 tracing::warn!("不支持的事件类型: {:?}", stream_event);
    //             }
    //         }
    //     }
    // }

    pub fn process_symbol_list(&self, exchange_info: serde_json::Value) -> Result<Vec<Symbol>, BinanceDataProcessorError> {
        let symbols = exchange_info
            .get("symbols")
            .context(MissingFieldSnafu {
                field: "symbols".to_string(),
                context: None,
            })?
            .as_array()
            .context(ArrayParseFailedSnafu {
                actual_type: "not array".to_string(),
            })?;

        let symbol_list = symbols
            .iter()
            .map(|symbol| {
                let binance_symbol = serde_json::from_value::<BinanceSymbolRaw>(symbol.clone()).context(JsonParseFailedSnafu)?;
                Ok(Symbol::new(
                    binance_symbol.symbol.as_str(),
                    Some(binance_symbol.base_asset.as_str()),
                    Some(binance_symbol.quote_asset.as_str()),
                    Exchange::Binance,
                    0.001,
                ))
            })
            .collect::<Result<Vec<Symbol>, BinanceDataProcessorError>>()?;

        Ok(symbol_list)
    }

    pub fn process_symbol(&self, symbol_info: serde_json::Value) -> Result<Symbol, BinanceDataProcessorError> {

        let symbol_info = symbol_info
            .get("symbols")
            .context(MissingFieldSnafu {
                field: "symbols".to_string(),
                context: None,
            })?
            .as_array()
            .context(ArrayParseFailedSnafu {
                actual_type: "not array".to_string(),
            })?;

        // determine the list length is 1
        if symbol_info.len() != 1 {
            return Err(DataValidationFailedSnafu {
                field: "symbols".to_string(),
                value: format!("length: {}", symbol_info.len()),
            }.build().into());
        }

        let symbol = Symbol::new(
            symbol_info[0]
                .get("symbol")
                .context(MissingFieldSnafu {
                    field: "symbol".to_string(),
                    context: None,
                })?
                .as_str()
                .context(InvalidFieldTypeSnafu {
                    field: "symbol".to_string(),
                    expected: "string".to_string(),
                    actual: "not string".to_string(),
                })?,
            None,
            None,
            Exchange::Binance,
            0.001,
        );
        Ok(symbol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_process_kline_series() {
        let processor = BinanceDataProcessor;

        let raw_data = vec![json!([
            1499040000000i64,
            "0.01634790",
            "0.80000000",
            "0.01575800",
            "0.01577100",
            "148976.11427815",
            1499644799999i64,
            "2434.19055334",
            308,
            "1756.87402397",
            "28.46694368",
            "17928899.62484339"
        ])];

        let result = processor.process_kline_series(raw_data).await;

        assert!(result.is_ok());
        let klines = result.unwrap();
        assert_eq!(klines.len(), 1);

        let kline = &klines[0];
        assert_eq!(kline.datetime, Utc.timestamp_millis_opt(1499040000000).single().unwrap());
        assert_eq!(kline.open, 0.01634790);
        assert_eq!(kline.high, 0.80000000);
        assert_eq!(kline.low, 0.01575800);
        assert_eq!(kline.close, 0.01577100);
        assert_eq!(kline.volume, 148976.11427815);
    }

    #[tokio::test]
    async fn test_process_kline_series_multiple() {
        let processor = BinanceDataProcessor;

        let raw_data = vec![
            json!([
                1499040000000i64,
                "0.01634790",
                "0.80000000",
                "0.01575800",
                "0.01577100",
                "148976.11427815",
                1499644799999i64,
                "2434.19055334",
                308,
                "1756.87402397",
                "28.46694368",
                "17928899.62484339"
            ]),
            json!([
                1499040060000i64,
                "0.01577100",
                "0.85000000",
                "0.01600000",
                "0.01650000",
                "150000.00000000",
                1499644859999i64,
                "2500.00000000",
                310,
                "1800.00000000",
                "30.00000000",
                "18000000.00000000"
            ]),
        ];

        let result = processor.process_kline_series(raw_data).await;

        assert!(result.is_ok());
        let klines = result.unwrap();
        assert_eq!(klines.len(), 2);

        assert_eq!(klines[0].open, 0.01634790);
        assert_eq!(klines[1].open, 0.01577100);
    }

    #[tokio::test]
    async fn test_process_kline_series_invalid_data() {
        let processor = BinanceDataProcessor;

        let raw_data = vec![json!([
            1499040000000i64,
            "invalid_number",
            "0.80000000",
            "0.01575800",
            "0.01577100",
            "148976.11427815",
            1499644799999i64,
            "2434.19055334",
            308,
            "1756.87402397",
            "28.46694368",
            "17928899.62484339"
        ])];

        let result = processor.process_kline_series(raw_data).await;
        assert!(result.is_err());
    }
}
