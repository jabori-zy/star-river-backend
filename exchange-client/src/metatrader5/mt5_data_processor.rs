use std::sync::Arc;
use tokio::sync::Mutex;
use event_center::EventPublisher;
use event_center::exchange_event::{ExchangeEvent, ExchangeKlineSeriesUpdateEventInfo, ExchangeKlineUpdateEventInfo};
use types::market::{Kline, Exchange, KlineSeries};
use types::position::PositionNumber;
use utils::{get_utc8_timestamp_millis, generate_batch_id};
use crate::metatrader5::Mt5KlineInterval;
use types::order::{Order, OrderType, OrderSide, OrderStatus};

#[derive(Debug)]
pub struct Mt5DataProcessor {
    event_publisher: Arc<Mutex<EventPublisher>>,

}


impl Mt5DataProcessor {
    pub fn new(event_publisher: Arc<Mutex<EventPublisher>>) -> Self {
        Self {
            event_publisher,
        }
    }

    async fn process_stream_kline(&self, raw_stream: serde_json::Value) {
        if let Some(kline_data) = raw_stream.get("data") {
            let symbol = kline_data["symbol"].as_str().expect("解析symbol失败");
            let interval = kline_data["interval"].as_str().expect("解析interval失败").parse::<Mt5KlineInterval>().expect("解析interval失败");
            let timestamp = kline_data["timestamp"].as_i64().expect("解析timestamp失败");
            let open = kline_data["open"].as_f64().expect("解析open失败");
            let high = kline_data["high"].as_f64().expect("解析high失败");
            let low = kline_data["low"].as_f64().expect("解析low失败");
            let close = kline_data["close"].as_f64().expect("解析close失败");
            let volume = kline_data["volume"].as_f64().expect("解析volume失败");

            let kline = Kline {
                timestamp: timestamp,
                open: open,
                high: high,
                low: low,
                close: close,
                volume: volume,
            };
            let exchange_kline_update_event_config = ExchangeKlineUpdateEventInfo {
                exchange: Exchange::Metatrader5,
                symbol: symbol.to_string(),
                interval: interval.clone().into(),
                kline: kline,
                event_timestamp: get_utc8_timestamp_millis(),
                batch_id: generate_batch_id(),
            };
            let event = ExchangeEvent::ExchangeKlineUpdate(exchange_kline_update_event_config).into();
            let _ = self.event_publisher.lock().await.publish(event);

            
        }
    }

    pub async fn process_stream(&self, raw_stream: serde_json::Value) {
        // 如果data_type为kline，则处理k线数据，如果没有data_type，则跳过
        if let Some(data_type) = raw_stream.get("data_type") {
            match data_type.as_str() {
                Some("kline") => {
                    self.process_stream_kline(raw_stream).await;
                }
                _ => {}
            }
        }
    }

    pub async fn process_kline_series(&self, symbol: &str, interval: Mt5KlineInterval, raw_data: Vec<serde_json::Value>) {
        let klines = raw_data
            .iter()
            .map(|k| {
                if let Some(arr) = k.as_array() {
                    Kline {
                        timestamp: arr[0].as_i64().unwrap_or(0),
                        open: arr[1].as_f64().unwrap_or(0.0),
                        high: arr[2].as_f64().unwrap_or(0.0),
                        low: arr[3].as_f64().unwrap_or(0.0),
                        close: arr[4].as_f64().unwrap_or(0.0),
                        volume: arr[5].as_f64().unwrap_or(0.0),
                    }
                } else {
                    tracing::error!("K线数据格式错误: {:?}", k);
                    Kline {
                        timestamp: 0,
                        open: 0.0,
                        high: 0.0,
                        low: 0.0,
                        close: 0.0,
                        volume: 0.0,
                    }
                }
            })
            .collect::<Vec<Kline>>();
        let kline_series = KlineSeries {
            exchange: Exchange::Metatrader5,
            symbol: symbol.to_string(),
            interval: interval.clone().into(),
            series: klines,
        };

        let exchange_klineseries_update_event_config = ExchangeKlineSeriesUpdateEventInfo {
            exchange: Exchange::Metatrader5,
            event_timestamp: get_utc8_timestamp_millis(),
            symbol: symbol.to_string(),
            interval: interval.clone().into(),
            kline_series,
            batch_id: generate_batch_id(),
        };
        let exchange_klineseries_update_event = ExchangeEvent::ExchangeKlineSeriesUpdate(exchange_klineseries_update_event_config).into();
        let _ = self.event_publisher.lock().await.publish(exchange_klineseries_update_event);
    }

    pub async fn process_order(&self, order_info: serde_json::Value) -> Result<Order, String> {
        // tracing::info!("处理订单信息: {:?}", order_info);
        let order_data = order_info["data"].clone();
        tracing::debug!("订单信息: {:?}", order_data);
        let order = Order {
            strategy_id: order_info["strategy_id"].as_i64()
                .expect("解析strategy_id失败"),
            node_id: order_info["node_id"].as_str()
                .expect("解析node_id失败")
                .to_string(),
            order_id: order_data["order_id"].as_i64()
                .expect("解析order_id失败"),
            exchange: Exchange::Metatrader5,
            symbol: order_data["symbol"].as_str()
                .expect("解析symbol失败")
                .to_string(),
            order_type: order_data["order_type"].as_str()
                .expect("解析order_type失败")
                .parse::<OrderType>()
                .expect("解析order_type失败"),
            order_side: order_data["order_side"].as_str()
                .expect("解析order_side失败")
                .parse::<OrderSide>()
                .expect("解析order_side失败"),
            quantity: order_data["volume"].as_f64()
                .expect("解析volume失败"),
            price: order_data.get("price").and_then(|p| p.as_f64()).unwrap_or(0.0),
            tp: order_data.get("tp").and_then(|tp| tp.as_f64()),
            sl: order_data.get("sl").and_then(|sl| sl.as_f64()),
            order_status: OrderStatus::Created,
        };
        tracing::info!("订单信息: {:?}", order);
        Ok(order)
    }

    pub async fn process_position_number(&self, position_number_info: serde_json::Value) -> Result<PositionNumber, String> {
        let position_number_data = position_number_info["data"].clone();
        tracing::debug!("仓位数量信息 :{:?}", position_number_data);
        let position_number = PositionNumber {
            exchange: Exchange::Metatrader5,
            symbol: position_number_data["symbol"].as_str().expect("解析symbol失败").to_string(),
            position_side: None,
            position_number: position_number_data["position_number"].as_i64().expect("解析position_number失败") as i32
        };
        Ok(position_number)

    }
}




