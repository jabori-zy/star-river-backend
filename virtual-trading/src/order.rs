use super::VirtualTradingSystem;
use types::order::{FuturesOrderSide, OrderType, OrderStatus, TpslType};
use chrono::{DateTime, Utc};
use types::order::virtual_order::VirtualOrder;
use types::custom_type::*;
use types::market::Exchange;
use types::cache::key::KlineKey;
use types::virtual_trading_system::event::VirtualTradingSystemEvent;
use types::position::virtual_position::VirtualPosition;
use types::position::PositionSide;

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
                        None,
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
                    tracing::debug!("市价订单创建成功: {:?}", market_order);
                    
                    let order_create_event = VirtualTradingSystemEvent::FuturesOrderCreated(market_order.clone());
                    let _ = self.event_publisher.send(order_create_event);
                    
                    // 插入订单
                    self.orders.push(market_order.clone());
                    
                    // 创建完成后，直接成交订单
                    self.execute_order(&market_order, *current_price, *current_timestamp).unwrap();
                    
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

    pub fn update_order_status(&mut self, order_id: OrderId, order_status: OrderStatus, timestamp: i64) -> Result<VirtualOrder, String> {
        if let Some(order) = self.orders.iter_mut().find(|o| o.order_id == order_id) {
            // 更新订单状态
            order.order_status = order_status;
            // 更新订单更新时间
            order.update_time = DateTime::from_timestamp_millis(timestamp).unwrap();
            Ok(order.clone())
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

    // 获取未成交订单
    pub fn get_unfilled_orders(&self) -> Vec<VirtualOrder> {
        self.orders
            .iter()
            .filter(|order| order.order_status == OrderStatus::Created || order.order_status == OrderStatus::Placed)
            .cloned()
            .collect::<Vec<VirtualOrder>>()
    }


    // 检查未成交订单（包括挂单，止盈止损订单），如果未成交，则立即成交
    pub fn check_unfilled_orders(&mut self) {
        // 获取未成交订单
        let unfilled_orders = self.get_unfilled_orders();
        for order in unfilled_orders {
            
            if let Some(kline_cache_key) = self.get_kline_key(&order.exchange, &order.symbol) {
                if let Some((current_price, _)) = self.kline_price.get(&kline_cache_key) {
                    match order.order_type {
                        OrderType::Limit => {
                            match order.order_side {
                                FuturesOrderSide::OpenLong => {
                                    // 限价开多：当前价格 <= 订单价格时执行
                                    if *current_price <= order.open_price {
                                        self.execute_order(&order, *current_price, self.timestamp).unwrap();
                                    }
                                }
                                FuturesOrderSide::OpenShort => {
                                    // 限价开空：当前价格 >= 订单价格时执行
                                    if *current_price >= order.open_price {
                                        self.execute_order(&order, *current_price, self.timestamp).unwrap();
                                    }
                                }
                                _ => {}
                            }
                        }
                        OrderType::StopMarket => {
                            match order.order_side {
                                FuturesOrderSide::CloseLong => {
                                    // 平多止损：当前价格 <= 止损价格时执行
                                    if *current_price <= order.open_price {
                                        self.execute_sl_order(&order);
                                    }
                                    
                                }
                                FuturesOrderSide::CloseShort => {
                                    // 平空止损：当前价格 >= 止损价格时执行
                                    if *current_price >= order.open_price {
                                        self.execute_sl_order(&order);
                                    }
                                    
                                }
                                _ => {}
                            }
                        }
                        OrderType::TakeProfitMarket => {
                            match order.order_side {
                                FuturesOrderSide::CloseLong => {
                                    // 平多止盈：当前价格 >= 止盈价格时执行
                                    if *current_price >= order.open_price {
                                        self.execute_tp_order(&order);
                                    }
                                    
                                }
                                FuturesOrderSide::CloseShort => {
                                    // 平空止盈：当前价格 <= 止盈价格时执行
                                    if *current_price <= order.open_price {
                                        self.execute_tp_order(&order);
                                    }
                                    
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }



    pub fn create_take_profit_order(&mut self, position: &VirtualPosition) -> Option<VirtualOrder> {
        
        // 生成止盈订单
        if let Some(tp) = position.tp {
            
            let order_id = self.generate_order_id();

            let order_side = match position.position_side {
                PositionSide::Long => FuturesOrderSide::CloseLong,
                PositionSide::Short => FuturesOrderSide::CloseShort,
            };

            let tp_order = VirtualOrder::new(
                Some(position.position_id),
                order_id, 
                position.strategy_id, 
                position.node_id.clone(), 
                position.order_config_id, 
                position.exchange.clone(), 
                position.symbol.clone(),
                order_side,
                OrderType::TakeProfitMarket, // 虚拟交易系统，只使用市价止盈
                position.quantity, // 全部止盈
                tp, // 止盈订单的开仓价格，就是主订单中的止盈价格
                None,
                None,
                None,
                None,
                self.timestamp
            );
            return Some(tp_order);
        }
        None
    }


    pub fn create_stop_loss_order(&mut self, position: &VirtualPosition) -> Option<VirtualOrder> {
        
        // 生成止盈止损订单
        if let Some(sl) = position.sl {
            
            let order_id = self.generate_order_id();

            let order_side = match position.position_side {
                PositionSide::Long => FuturesOrderSide::CloseLong,
                PositionSide::Short => FuturesOrderSide::CloseShort,
            };

            let sl_order = VirtualOrder::new(
                Some(position.position_id),
                order_id, 
                position.strategy_id, 
                position.node_id.clone(), 
                position.order_config_id, 
                position.exchange.clone(), 
                position.symbol.clone(),
                order_side,
                OrderType::StopMarket, // 虚拟交易系统，只使用市价止损
                position.quantity, // 全部止盈
                sl, // 止损订单的开仓价格，就是主订单中的止损价格
                None,
                None,
                None,
                None,
                self.timestamp
            );
            return Some(sl_order);
                
            
        }
        None
    }
    
}