use std::sync::Arc;
use tokio::sync::Mutex;
use event_center::EventPublisher;
use event_center::exchange_event::{ExchangeEvent, ExchangeKlineUpdateEvent, ExchangeKlineSeriesUpdateEvent};
use types::market::{Kline, Exchange, KlineSeries, MT5Server};
use types::position::PositionNumber;
use utils::{get_utc8_timestamp_millis, generate_batch_id};
use crate::metatrader5::mt5_types::Mt5KlineInterval;
use types::order::OriginalOrder;
use crate::metatrader5::mt5_types::{Mt5Order, Mt5OrderState, Mt5Position};
use types::order::Order;
use types::position::{OriginalPosition, Position};
use types::transaction::{OriginalTransaction, Transaction};
use crate::metatrader5::mt5_types::Mt5Deal;
use types::account::OriginalAccountInfo;
use types::account::mt5_account::OriginalMt5AccountInfo;
use chrono::{Utc, TimeZone};

#[derive(Debug)]
pub struct Mt5DataProcessor {
    server: MT5Server,
    event_publisher: Arc<Mutex<EventPublisher>>,

}


impl Mt5DataProcessor {
    pub fn new(event_publisher: Arc<Mutex<EventPublisher>>, server: MT5Server) -> Self {
        Self {
            event_publisher,
            server,
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
            let exchange_kline_update_event_config = ExchangeKlineUpdateEvent {
                exchange: Exchange::Metatrader5(self.server.clone()),
                symbol: symbol.to_string(),
                interval: interval.clone().into(),
                kline: kline,
                event_timestamp: get_utc8_timestamp_millis(),
            };
            let event = ExchangeEvent::ExchangeKlineUpdate(exchange_kline_update_event_config).into();
            self.event_publisher.lock().await.publish(event).await.unwrap();

            
        }
    }

    pub async fn process_stream(&self, raw_stream: serde_json::Value) {
        // tracing::debug!("处理流数据: {:?}", raw_stream);
        // 如果data_type为kline，则处理k线数据，如果没有data_type，则跳过
        if let Some(data_type) = raw_stream.get("type") {
            match data_type.as_str() {
                Some("kline") => {
                    self.process_stream_kline(raw_stream).await;
                }
                _ => {}
            }
        }
    }

    pub async fn process_kline_series(&self, symbol: &str, interval: Mt5KlineInterval, raw_data: serde_json::Value) -> Vec<Kline> {
        // let data = raw_data["data"].as_array().expect("转换为array失败");
        let klines: Vec<Kline> = raw_data
            .as_array()
            .expect("转换为array失败")
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

        // let exchange_klineseries_update = ExchangeKlineSeriesUpdateEvent {
        //     exchange: Exchange::Metatrader5(self.server.clone()),
        //     event_timestamp: get_utc8_timestamp_millis(),
        //     symbol: symbol.to_string(),
        //     interval: interval.clone().into(),
        //     kline_series: klines.clone(),
        // };
        // let exchange_klineseries_update_event = ExchangeEvent::ExchangeKlineSeriesUpdate(exchange_klineseries_update);
        // self.event_publisher.lock().await.publish(exchange_klineseries_update_event.into()).await.unwrap();
        klines
    }

    // 处理订单信息
    pub async fn process_order(&self, order_info: serde_json::Value) -> Result<Box<dyn OriginalOrder>, String> {
        let mut order_data = order_info["data"][0].clone();
        order_data["server"] = self.server.clone().into();
        tracing::debug!("订单信息: {:?}", order_data);
        // 取出order_data  array 的第一个值
        
        let order = serde_json::from_value::<Mt5Order>(order_data)
            .map_err(|e| format!("解析订单数据失败: {}", e))?;
        tracing::info!("订单信息: {:?}", order);
        Ok(Box::new(order))
    }

    pub async fn update_order(&self, new_order_info: serde_json::Value, old_order: Order) -> Result<Order, String> {
        tracing::debug!("订单信息: {:?}", new_order_info);
        // 订单数据
        // 取列表元素第一个值
        let order_data = new_order_info["data"][0].clone();
        let new_order_status = order_data["state"].as_str()
            .expect("解析order_status失败").parse::<Mt5OrderState>().expect("转换为mt5_order_status失败");

        let order = Order {
            order_id: old_order.order_id,
            strategy_id: old_order.strategy_id,
            node_id: old_order.node_id,
            exchange_order_id: old_order.exchange_order_id,
            account_id: old_order.account_id,
            exchange: old_order.exchange,
            symbol: old_order.symbol,
            order_side: old_order.order_side,
            order_type: old_order.order_type,
            order_status: new_order_status.into(),
            quantity: old_order.quantity,
            open_price: old_order.open_price,
            tp: old_order.tp,
            sl: old_order.sl,
            extra_info: old_order.extra_info,
            created_time: old_order.created_time,
            updated_time: old_order.updated_time,
        };
        Ok(order)
    }

    pub async fn process_position(&self, mut position_json: serde_json::Value) -> Result<Box<dyn OriginalPosition>, String> {
        position_json["server"] = self.server.clone().into();

        tracing::debug!("仓位信息 :{:?}", position_json);
        let position = serde_json::from_value::<Mt5Position>(position_json)
            .map_err(|e| format!("解析仓位数据失败: {}", e))?;
        tracing::info!("仓位信息: {:?}", position);

        Ok(Box::new(position))
    }

    pub async fn process_latest_position(&self, mut new_position_json: serde_json::Value, old_position: &Position) -> Result<Position, String> {
        // tracing::debug!("最新仓位信息: {:?}", new_position_json);
        // 仓位数据
        new_position_json["server"] = self.server.clone().into();
        let new_mt_position = serde_json::from_value::<Mt5Position>(new_position_json)
            .map_err(|e| format!("解析仓位数据失败: {}", e))?;
        let new_position = Position {
            position_id: old_position.position_id,
            strategy_id: old_position.strategy_id.clone(),
            node_id: old_position.node_id.clone(),
            account_id: old_position.account_id,
            exchange: old_position.exchange.clone(),
            exchange_position_id: old_position.exchange_position_id.clone(),
            symbol: old_position.symbol.clone(),
            position_side: old_position.position_side.clone(),
            position_state: old_position.position_state.clone(),
            quantity: old_position.quantity,
            open_price: old_position.open_price,
            current_price: Some(new_mt_position.current_price),
            tp: old_position.tp,
            sl: old_position.sl,
            unrealized_profit: Some(new_mt_position.profit),
            extra_info: old_position.extra_info.clone(),
            create_time: Utc.timestamp_millis_opt(new_mt_position.time_msc).unwrap(), // 毫秒转换为时间
            update_time: Utc.timestamp_millis_opt(new_mt_position.time_update_msc).unwrap(), // 毫秒转换为时间
        };
        return Ok(new_position);
        
        
    }

    pub async fn process_deal(&self, deal_info: serde_json::Value) -> Result<Box<dyn OriginalTransaction>, String> {
        let mut deal_data = deal_info["data"][0].clone();
        deal_data["server"] = self.server.clone().into();
        tracing::debug!("成交信息 :{:?}", deal_data);
        let deal = serde_json::from_value::<Mt5Deal>(deal_data)
            .map_err(|e| format!("解析成交数据失败: {}", e))?;
        Ok(Box::new(deal))
    }

    pub async fn process_position_number(&self, position_number_info: serde_json::Value) -> Result<PositionNumber, String> {
        let position_number_data = position_number_info["data"].clone();
        tracing::debug!("仓位数量信息 :{:?}", position_number_data);
        let position_number = PositionNumber {
            exchange: Exchange::Metatrader5(self.server.clone()),
            symbol: position_number_data["symbol"].as_str().expect("解析symbol失败").to_string(),
            position_side: None,
            position_number: position_number_data["position_number"].as_i64().expect("解析position_number失败") as i32
        };
        Ok(position_number)

    }

    pub async fn process_account_info(&self, account_id: i32, account_info: serde_json::Value) -> Result<Box<dyn OriginalAccountInfo>, String> {
        let mut account_info_data = account_info["data"].clone();
        // 把account_id 添加到account_info_data中
        account_info_data["account_id"] = account_id.into();
        let account_info = serde_json::from_value::<OriginalMt5AccountInfo>(account_info_data)
            .map_err(|e| format!("解析账户信息失败: {}", e)).expect("解析账户信息失败");
        Ok(Box::new(account_info))
    }
}




