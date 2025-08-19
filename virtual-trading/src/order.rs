use super::VirtualTradingSystem;
use types::order::{FuturesOrderSide, OrderType, OrderStatus, TpslType};
use chrono::{DateTime, Utc};
use types::order::virtual_order::VirtualOrder;
use types::custom_type::*;
use types::market::Exchange;
use types::cache::key::KlineKey;
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
        tp_type: Option<TpslType>,
        sl_type: Option<TpslType>,
    ) -> Result<(), String> {
        let order_id = self.generate_order_id();
        let kline_cache_key = self.get_kline_key(&exchange, &symbol);
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
                        tp_type,
                        sl_type,
                        *current_timestamp
                    );
                    
                    let order_id = market_order.order_id;
                    let order_create_event = VirtualTradingSystemEvent::FuturesOrderCreated(market_order.clone());
                    let _ = self.event_publisher.send(order_create_event);
                    
                    // 插入订单
                    self.orders.push(market_order.clone());
                    
                    // 创建完成后，直接成交订单
                    self.execute_order(order_id, current_price.clone(), *current_timestamp).unwrap();
                    
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

    pub fn get_virtual_orders(&self) -> Vec<VirtualOrder> {
        self.orders.clone()
    }


    pub fn create_take_profit_order(&mut self, order: &VirtualOrder, position_id: PositionId, timestamp: i64) -> Option<VirtualOrder> {
        
        // 生成止盈止损订单
        if let Some(tp) = order.tp {
            
            let order_id = self.generate_order_id();

            let order_side = match order.order_side {
                FuturesOrderSide::OpenLong => FuturesOrderSide::CloseLong,
                FuturesOrderSide::OpenShort => FuturesOrderSide::CloseShort,
                _ => return None,
            };

            let mut tp_order = VirtualOrder::new(
                order_id, 
                order.strategy_id, 
                order.node_id.clone(), 
                order.order_config_id, 
                order.exchange.clone(), 
                order.symbol.clone(),
                order_side,
                OrderType::TakeProfitMarket, // 虚拟交易系统，只使用市价止盈
                order.quantity, // 全部止盈
                tp, // 止盈订单的开仓价格，就是主订单中的止盈价格
                None,
                None,
                None,
                None,
                timestamp
            );
            tp_order.position_id = Some(position_id);
            return Some(tp_order);
                
            
        }
        None
    }


    pub fn create_stop_loss_order(&mut self, order: &VirtualOrder, position_id: PositionId, timestamp: i64) -> Option<VirtualOrder> {
        
        // 生成止盈止损订单
        if let Some(sl) = order.sl {
            
            let order_id = self.generate_order_id();

            let order_side = match order.order_side {
                FuturesOrderSide::OpenLong => FuturesOrderSide::CloseLong,
                FuturesOrderSide::OpenShort => FuturesOrderSide::CloseShort,
                _ => return None,
            };

            let mut sl_order = VirtualOrder::new(
                order_id, 
                order.strategy_id, 
                order.node_id.clone(), 
                order.order_config_id, 
                order.exchange.clone(), 
                order.symbol.clone(),
                order_side,
                OrderType::StopMarket, // 虚拟交易系统，只使用市价止损
                order.quantity, // 全部止盈
                sl, // 止损订单的开仓价格，就是主订单中的止损价格
                None,
                None,
                None,
                None,
                timestamp
            );
            sl_order.position_id = Some(position_id);
            return Some(sl_order);
                
            
        }
        None
    }
    
}