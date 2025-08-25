use std::sync::Arc;
use tokio::sync::Mutex;
use event_center::EventPublisher;
use event_center::exchange_event::{ExchangeEvent, ExchangeKlineUpdateEvent};
use types::market::{Kline, Exchange, MT5Server};
use types::position::PositionNumber;
use utils::get_utc8_timestamp_millis;
use crate::metatrader5::mt5_types::Mt5KlineInterval;
use types::order::OriginalOrder;
use crate::metatrader5::mt5_types::{Mt5Order, Mt5OrderState, Mt5Position};
use types::order::Order;
use types::position::{OriginalPosition, Position};
use types::transaction::OriginalTransaction;
use crate::metatrader5::mt5_types::Mt5Deal;
use types::account::OriginalAccountInfo;
use types::account::mt5_account::OriginalMt5AccountInfo;
use chrono::{Utc, TimeZone};
use crate::data_processor_error::DataProcessorError;
use types::market::Symbol;

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

    async fn process_stream_kline(&self, raw_stream: serde_json::Value) -> Result<(), DataProcessorError> {
        let kline_data = raw_stream.get("data")
            .ok_or_else(|| DataProcessorError::missing_field("data", Some("stream kline data".to_string())))?;

        // Extract and validate symbol
        let symbol = kline_data["symbol"].as_str()
            .ok_or_else(|| DataProcessorError::missing_field("symbol", Some("kline data".to_string())))?;

        // Extract and validate interval
        let interval_str = kline_data["interval"].as_str()
            .ok_or_else(|| DataProcessorError::missing_field("interval", Some("kline data".to_string())))?;
        let interval = interval_str.parse::<Mt5KlineInterval>()
            .map_err(|e| DataProcessorError::type_conversion(
                "interval",
                format!("Failed to parse interval '{}': {}", interval_str, e),
                Some(interval_str.to_string())
            ))?;

        // Extract and validate timestamp
        let timestamp = kline_data["timestamp"].as_i64()
            .ok_or_else(|| DataProcessorError::missing_field("timestamp", Some("kline data".to_string())))?;

        // Validate timestamp range (should be positive and reasonable)
        if timestamp <= 0 {
            return Err(DataProcessorError::data_validation(
                "Timestamp must be positive",
                Some("timestamp".to_string()),
                Some(timestamp.to_string())
            ));
        }

        // Extract and validate price data
        let open = kline_data["open"].as_f64()
            .ok_or_else(|| DataProcessorError::missing_field("open", Some("kline data".to_string())))?;
        let high = kline_data["high"].as_f64()
            .ok_or_else(|| DataProcessorError::missing_field("high", Some("kline data".to_string())))?;
        let low = kline_data["low"].as_f64()
            .ok_or_else(|| DataProcessorError::missing_field("low", Some("kline data".to_string())))?;
        let close = kline_data["close"].as_f64()
            .ok_or_else(|| DataProcessorError::missing_field("close", Some("kline data".to_string())))?;
        let volume = kline_data["volume"].as_f64()
            .ok_or_else(|| DataProcessorError::missing_field("volume", Some("kline data".to_string())))?;

        // Validate price data relationships
        if high < low {
            return Err(DataProcessorError::data_validation(
                format!("High price ({}) cannot be less than low price ({})", high, low),
                Some("price_validation".to_string()),
                Some(format!("high: {}, low: {}", high, low))
            ));
        }

        if open < 0.0 || high < 0.0 || low < 0.0 || close < 0.0 {
            return Err(DataProcessorError::data_validation(
                "Prices cannot be negative",
                Some("price_validation".to_string()),
                Some(format!("open: {}, high: {}, low: {}, close: {}", open, high, low, close))
            ));
        }

        if volume < 0.0 {
            return Err(DataProcessorError::data_validation(
                "Volume cannot be negative",
                Some("volume".to_string()),
                Some(volume.to_string())
            ));
        }

        let kline = Kline {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        };

        let exchange_kline_update_event = ExchangeKlineUpdateEvent {
            exchange: Exchange::Metatrader5(self.server.clone()),
            symbol: symbol.to_string(),
            interval: interval.clone().into(),
            kline,
            event_timestamp: get_utc8_timestamp_millis(),
        };

        let event = ExchangeEvent::ExchangeKlineUpdate(exchange_kline_update_event).into();
        
        self.event_publisher.lock().await.publish(event).await
            .map_err(|e| DataProcessorError::stream_processing(
                format!("Failed to publish kline update event: {}", e),
                Some("kline".to_string())
            ))?;

        Ok(())
    }

    pub async fn process_stream(&self, raw_stream: serde_json::Value) -> Result<(), DataProcessorError> {
        // tracing::debug!("处理流数据: {:?}", raw_stream);
        // 如果data_type为kline，则处理k线数据，如果没有data_type，则跳过
        if let Some(data_type) = raw_stream.get("type") {
            match data_type.as_str() {
                Some("kline") => {
                    self.process_stream_kline(raw_stream).await?
                }
                Some(unknown_type) => {
                    tracing::warn!("Unknown stream data type: {}", unknown_type);
                }
                None => {
                    return Err(DataProcessorError::invalid_field_type(
                        "type",
                        "string",
                        "non-string value",
                        Some("stream data".to_string())
                    ));
                }
            }
        }
        Ok(())
    }

    pub async fn process_symbol_list(&self, symbols: serde_json::Value) -> Result<Vec<Symbol>, DataProcessorError> {
        let symbols = symbols.as_array()
            .ok_or_else(|| DataProcessorError::array_parsing(
                "non-array",
                "symbol list"
            ))?;
        let mut symbol_list = Vec::new();
        for (_, symbol) in symbols.iter().enumerate() {
            let symbol_name = symbol.get("name")
                .ok_or_else(|| DataProcessorError::missing_field("name", Some("symbol".to_string())))?
                .as_str()
                .ok_or_else(|| DataProcessorError::invalid_field_type(
                    "name",
                    "string",
                    "non-string",
                    Some("symbol".to_string())
                ))?;
            let symbol = Symbol::new(symbol_name, None, None, Exchange::Metatrader5(self.server.clone()));
            symbol_list.push(symbol);
        };
        // println!("symbol_list: {:?}", symbol_list);
        Ok(symbol_list)
    }

    pub async fn process_kline_series(&self, symbol: &str, interval: Mt5KlineInterval, raw_data: serde_json::Value) -> Result<Vec<Kline>, DataProcessorError> {
        let data_array = raw_data.as_array()
            .ok_or_else(|| DataProcessorError::array_parsing(
                "non-array",
                "kline series data"
            ))?;

        let mut klines = Vec::with_capacity(data_array.len());

        for (index, k) in data_array.iter().enumerate() {
            let arr = k.as_array()
                .ok_or_else(|| DataProcessorError::kline_data_parsing(
                    format!("Kline data at index {} is not an array: {:?}", index, k),
                    Some(symbol.to_string()),
                    Some(interval.to_string())
                ))?;

            if arr.len() != 6 {
                return Err(DataProcessorError::invalid_kline_array_format(
                    arr.len(),
                    format!("{:?}", arr)
                ));
            }

            // Extract and validate each field
            let timestamp = arr[0].as_i64()
                .ok_or_else(|| DataProcessorError::kline_data_parsing(
                    format!("Invalid timestamp at index {}: {:?}", index, arr[0]),
                    Some(symbol.to_string()),
                    Some(interval.to_string())
                ))?;

            let open = arr[1].as_f64()
                .ok_or_else(|| DataProcessorError::kline_data_parsing(
                    format!("Invalid open price at index {}: {:?}", index, arr[1]),
                    Some(symbol.to_string()),
                    Some(interval.to_string())
                ))?;

            let high = arr[2].as_f64()
                .ok_or_else(|| DataProcessorError::kline_data_parsing(
                    format!("Invalid high price at index {}: {:?}", index, arr[2]),
                    Some(symbol.to_string()),
                    Some(interval.to_string())
                ))?;

            let low = arr[3].as_f64()
                .ok_or_else(|| DataProcessorError::kline_data_parsing(
                    format!("Invalid low price at index {}: {:?}", index, arr[3]),
                    Some(symbol.to_string()),
                    Some(interval.to_string())
                ))?;

            let close = arr[4].as_f64()
                .ok_or_else(|| DataProcessorError::kline_data_parsing(
                    format!("Invalid close price at index {}: {:?}", index, arr[4]),
                    Some(symbol.to_string()),
                    Some(interval.to_string())
                ))?;

            let volume = arr[5].as_f64()
                .ok_or_else(|| DataProcessorError::kline_data_parsing(
                    format!("Invalid volume at index {}: {:?}", index, arr[5]),
                    Some(symbol.to_string()),
                    Some(interval.to_string())
                ))?;

            // Validate data integrity
            if timestamp <= 0 {
                return Err(DataProcessorError::data_validation(
                    format!("Invalid timestamp {} at index {}", timestamp, index),
                    Some("timestamp".to_string()),
                    Some(timestamp.to_string())
                ));
            }

            if high < low {
                return Err(DataProcessorError::data_validation(
                    format!("High price ({}) less than low price ({}) at index {}", high, low, index),
                    Some("price_validation".to_string()),
                    Some(format!("high: {}, low: {}", high, low))
                ));
            }

            if open < 0.0 || high < 0.0 || low < 0.0 || close < 0.0 {
                return Err(DataProcessorError::data_validation(
                    format!("Negative price values at index {}", index),
                    Some("price_validation".to_string()),
                    Some(format!("open: {}, high: {}, low: {}, close: {}", open, high, low, close))
                ));
            }

            if volume < 0.0 {
                return Err(DataProcessorError::data_validation(
                    format!("Negative volume {} at index {}", volume, index),
                    Some("volume".to_string()),
                    Some(volume.to_string())
                ));
            }

            klines.push(Kline {
                timestamp,
                open,
                high,
                low,
                close,
                volume,
            });
        }

        // Optionally publish kline series update event
        // let exchange_klineseries_update = ExchangeKlineSeriesUpdateEvent {
        //     exchange: Exchange::Metatrader5(self.server.clone()),
        //     event_timestamp: get_utc8_timestamp_millis(),
        //     symbol: symbol.to_string(),
        //     interval: interval.clone().into(),
        //     kline_series: klines.clone(),
        // };
        // let exchange_klineseries_update_event = ExchangeEvent::ExchangeKlineSeriesUpdate(exchange_klineseries_update);
        // self.event_publisher.lock().await.publish(exchange_klineseries_update_event.into()).await
        //     .map_err(|e| DataProcessorError::stream_processing(
        //         format!("Failed to publish kline series update event: {}", e),
        //         Some("kline_series".to_string())
        //     ))?;

        Ok(klines)
    }

    // 处理订单信息
    pub async fn process_order(&self, order_info: serde_json::Value) -> Result<Box<dyn OriginalOrder>, DataProcessorError> {
        let data_array = order_info.get("data")
            .ok_or_else(|| DataProcessorError::missing_field("data", Some("order info".to_string())))?
            .as_array()
            .ok_or_else(|| DataProcessorError::array_parsing(
                "non-array",
                "order data field"
            ))?;

        if data_array.is_empty() {
            return Err(DataProcessorError::order_data_parsing(
                "Order data array is empty",
                None
            ));
        }

        let mut order_data = data_array[0].clone();
        order_data["server"] = self.server.clone().into();
        tracing::debug!("订单信息: {:?}", order_data);
        
        let order = serde_json::from_value::<Mt5Order>(order_data)
            .map_err(|e| DataProcessorError::order_data_parsing(
                format!("Failed to deserialize order data: {}", e),
                None
            ))?;
        
        tracing::info!("订单信息: {:?}", order);
        Ok(Box::new(order))
    }

    pub async fn update_order(&self, new_order_info: serde_json::Value, old_order: Order) -> Result<Order, DataProcessorError> {
        tracing::debug!("订单信息: {:?}", new_order_info);
        
        let data_array = new_order_info.get("data")
            .ok_or_else(|| DataProcessorError::missing_field("data", Some("order update info".to_string())))?
            .as_array()
            .ok_or_else(|| DataProcessorError::array_parsing(
                "non-array",
                "order update data field"
            ))?;

        if data_array.is_empty() {
            return Err(DataProcessorError::order_data_parsing(
                "Order update data array is empty",
                Some(old_order.order_id.into())
            ));
        }

        let order_data = &data_array[0];
        let state_str = order_data.get("state")
            .ok_or_else(|| DataProcessorError::missing_field("state", Some("order update data".to_string())))?
            .as_str()
            .ok_or_else(|| DataProcessorError::invalid_field_type(
                "state",
                "string",
                "non-string",
                Some("order update data".to_string())
            ))?;

        let new_order_status = state_str.parse::<Mt5OrderState>()
            .map_err(|e| DataProcessorError::enum_parsing(
                "state",
                state_str,
                vec!["ORDER_STATE_STARTED".to_string(), "ORDER_STATE_PLACED".to_string(), 
                     "ORDER_STATE_CANCELED".to_string(), "ORDER_STATE_PARTIAL".to_string(),
                     "ORDER_STATE_FILLED".to_string(), "ORDER_STATE_REJECTED".to_string(),
                     "ORDER_STATE_EXPIRED".to_string(), "ORDER_STATE_REQUEST_ADD".to_string(),
                     "ORDER_STATE_REQUEST_MODIFY".to_string(), "ORDER_STATE_REQUEST_CANCEL".to_string()]
            ))?;

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

    pub async fn process_position(&self, mut position_json: serde_json::Value) -> Result<Box<dyn OriginalPosition>, DataProcessorError> {
        position_json["server"] = self.server.clone().into();

        tracing::debug!("仓位信息 :{:?}", position_json);
        let position = serde_json::from_value::<Mt5Position>(position_json)
            .map_err(|e| DataProcessorError::position_data_parsing(
                format!("Failed to deserialize position data: {}", e),
                None
            ))?;
        tracing::info!("仓位信息: {:?}", position);

        Ok(Box::new(position))
    }

    pub async fn process_latest_position(&self, mut new_position_json: serde_json::Value, old_position: &Position) -> Result<Position, DataProcessorError> {
        // tracing::debug!("最新仓位信息: {:?}", new_position_json);
        // 仓位数据
        new_position_json["server"] = self.server.clone().into();
        let new_mt_position = serde_json::from_value::<Mt5Position>(new_position_json)
            .map_err(|e| DataProcessorError::position_data_parsing(
                format!("Failed to deserialize position data: {}", e),
                Some(old_position.position_id.into())
            ))?;

        // Validate timestamp conversion
        let create_time = Utc.timestamp_millis_opt(new_mt_position.time_msc)
            .single()
            .ok_or_else(|| DataProcessorError::timestamp_conversion(
                format!("Invalid create timestamp: {}", new_mt_position.time_msc),
                Some(new_mt_position.time_msc)
            ))?;

        let update_time = Utc.timestamp_millis_opt(new_mt_position.time_update_msc)
            .single()
            .ok_or_else(|| DataProcessorError::timestamp_conversion(
                format!("Invalid update timestamp: {}", new_mt_position.time_update_msc),
                Some(new_mt_position.time_update_msc)
            ))?;

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
            create_time,
            update_time,
        };
        
        Ok(new_position)
    }

    pub async fn process_deal(&self, deal_info: serde_json::Value) -> Result<Box<dyn OriginalTransaction>, DataProcessorError> {
        let data_array = deal_info.get("data")
            .ok_or_else(|| DataProcessorError::missing_field("data", Some("deal info".to_string())))?
            .as_array()
            .ok_or_else(|| DataProcessorError::array_parsing(
                "non-array",
                "deal data field"
            ))?;

        if data_array.is_empty() {
            return Err(DataProcessorError::deal_data_parsing(
                "Deal data array is empty",
                None
            ));
        }

        let mut deal_data = data_array[0].clone();
        deal_data["server"] = self.server.clone().into();
        tracing::debug!("成交信息 :{:?}", deal_data);
        
        let deal = serde_json::from_value::<Mt5Deal>(deal_data)
            .map_err(|e| DataProcessorError::deal_data_parsing(
                format!("Failed to deserialize deal data: {}", e),
                None
            ))?;
            
        Ok(Box::new(deal))
    }

    pub async fn process_position_number(&self, position_number_info: serde_json::Value) -> Result<PositionNumber, DataProcessorError> {
        let position_number_data = position_number_info.get("data")
            .ok_or_else(|| DataProcessorError::missing_field("data", Some("position number info".to_string())))?;
            
        tracing::debug!("仓位数量信息 :{:?}", position_number_data);
        
        let symbol = position_number_data.get("symbol")
            .ok_or_else(|| DataProcessorError::missing_field("symbol", Some("position number data".to_string())))?
            .as_str()
            .ok_or_else(|| DataProcessorError::invalid_field_type(
                "symbol",
                "string",
                "non-string",
                Some("position number data".to_string())
            ))?;

        let position_number_value = position_number_data.get("position_number")
            .ok_or_else(|| DataProcessorError::missing_field("position_number", Some("position number data".to_string())))?
            .as_i64()
            .ok_or_else(|| DataProcessorError::invalid_field_type(
                "position_number",
                "integer",
                "non-integer",
                Some("position number data".to_string())
            ))?;

        // Validate position number range
        if position_number_value < i32::MIN as i64 || position_number_value > i32::MAX as i64 {
            return Err(DataProcessorError::data_validation(
                format!("Position number {} out of i32 range", position_number_value),
                Some("position_number".to_string()),
                Some(position_number_value.to_string())
            ));
        }

        let position_number = PositionNumber {
            exchange: Exchange::Metatrader5(self.server.clone()),
            symbol: symbol.to_string(),
            position_side: None,
            position_number: position_number_value as i32
        };
        
        Ok(position_number)
    }

    pub async fn process_account_info(&self, account_id: i32, mut account_info: serde_json::Value) -> Result<Box<dyn OriginalAccountInfo>, DataProcessorError> {
        // 把account_id 添加到account_info_data中
        account_info["account_id"] = account_id.into();
        
        let account_info = serde_json::from_value::<OriginalMt5AccountInfo>(account_info)
            .map_err(|e| DataProcessorError::account_info_parsing(
                format!("Failed to deserialize account info: {}", e),
                Some(account_id)
            ))?;
            
        Ok(Box::new(account_info))
    }
}




