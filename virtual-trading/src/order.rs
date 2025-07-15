use super::VirtualTradingSystem;
use types::order::{FuturesOrderSide, OrderType, OrderStatus};
use chrono::{DateTime, Utc};
use types::order::virtual_order::VirtualOrder;
use types::custom_type::*;
use types::market::Exchange;
use types::cache::cache_key::BacktestKlineCacheKey;
use types::virtual_trading_system::event::VirtualTradingSystemEvent;

impl VirtualTradingSystem {
    // 生成订单ID, 从0开始
    fn generate_order_id(&self) -> i32 {
        self.orders.len() as i32
    }
    // 创建订单
    pub fn create_order(&mut self,
        strategy_id: i32,
        node_id: String,
        order_config_id: i32,
        symbol: String,
        exchange: Exchange,
        price: f64,
        order_side: FuturesOrderSide,
        order_type: OrderType,
        quantity: f64,
        tp: Option<f64>,
        sl: Option<f64>,
    ) -> Result<(), String> {
        let order_id = self.generate_order_id();
        let kline_cache_key = self.get_kline_cache_key(&exchange, &symbol);
        if let Some(kline_cache_key) = kline_cache_key {
            // 根据订单类型判断是否需要立即成交
            match &order_type {
                // 市价单
                OrderType::Market => {
                    // 市价单立即成交
                    // 获取当前价格
                    let (current_price, current_timestamp) = self.kline_price.get(&kline_cache_key).unwrap();
                    // 市价单忽略创建订单时的价格，而是使用最新的价格
                    let market_order = VirtualOrder::new(
                        order_id, 
                        strategy_id, 
                        node_id, 
                        order_config_id, 
                        exchange.clone(), 
                        symbol.clone(), 
                        order_side, 
                        order_type, 
                        quantity, 
                        current_price.clone(), 
                        tp, 
                        sl,
                        *current_timestamp);
                    
                    let order_id = market_order.order_id;
                    let order_create_event = VirtualTradingSystemEvent::FuturesOrderCreated(market_order.clone());
                    let _ = self.event_publisher.send(order_create_event);
                    
                    // 插入订单
                    self.orders.push(market_order.clone());
                    
                    // 创建完成后，直接成交订单
                    self.execute_order(order_id, current_price.clone(), *current_timestamp);
                    
                    

                    
                }
                // 限价单
                OrderType::Limit => {
                    // self.current_orders.push(virtual_order);
                }
                _ => {
                    // self.current_orders.push(virtual_order);
                }
                
            }
        } else {
            // 如果k线缓存key不存在，则不成交
            return Err(format!("k线缓存key不存在: {:?}", kline_cache_key));
        }
        Ok(())
    }

    pub fn update_order_status(&mut self, order_id: OrderId, order_status: OrderStatus, timestamp: i64) -> Result<(), String> {
        if let Some(order) = self.orders.iter_mut().find(|o| o.order_id == order_id) {
            // 更新订单状态
            order.order_status = order_status;
            // 更新订单更新时间
            order.update_time = DateTime::from_timestamp_millis(timestamp).unwrap();
            Ok(())
        } else {
            Err(format!("订单不存在: {:?}", order_id))
        }
    }

    pub fn update_order_position_id(&mut self, order_id: OrderId, position_id: PositionId, timestamp: i64) -> Result<(), String> {
        if let Some(order) = self.orders.iter_mut().find(|o| o.order_id == order_id) {
            order.position_id = Some(position_id);
            order.update_time = DateTime::from_timestamp_millis(timestamp).unwrap();
            Ok(())
        } else {
            Err(format!("订单不存在: {:?}", order_id))
        }
    }
    
}