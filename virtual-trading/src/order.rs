use super::VirtualTradingSystem;
use types::order::{OrderSide, OrderType, OrderStatus};
use chrono::Utc;
use types::order::virtual_order::VirtualOrder;
use types::custom_type::*;
use types::market::Exchange;
use types::cache::cache_key::BacktestKlineCacheKey;

impl VirtualTradingSystem {
    // 生成订单ID, 从0开始
    fn generate_order_id(&self) -> i32 {
        self.unfilled_orders.len() as i32
    }
    // 创建订单
    pub fn create_order(&mut self,
        strategy_id: i32,
        node_id: String,
        symbol: String,
        exchange: Exchange,
        price: f64,
        order_side: OrderSide,
        order_type: OrderType,
        quantity: f64,
        tp: Option<f64>,
        sl: Option<f64>,
    ) -> Result<OrderId, String> {
        let order_id = self.generate_order_id();
        tracing::debug!("order_id: {:?}", order_id);
        let kline_cache_key = self.get_kline_cache_key(&exchange, &symbol);
        if let Some(kline_cache_key) = kline_cache_key {
            // 根据订单类型判断是否需要立即成交
            match &order_type {
                // 市价单
                OrderType::Market => {
                    // 市价单立即成交
                    // 获取当前价格
                    let current_price = self.kline_cache_data.get(&kline_cache_key).unwrap();
                    // 市价单忽略创建订单时的价格，而是使用最新的价格
                    let market_order = VirtualOrder::new(order_id, strategy_id, node_id, exchange.clone(), symbol.clone(), order_side, order_type, quantity, current_price.clone(), tp, sl);
                    tracing::debug!("创建市价订单: {:?}", market_order);
                    // 创建完成后，直接成交订单
                    let position_id = self.execute_order(&market_order, current_price.clone());
                    
                    // 将订单加入到所有订单中
                    self.orders.push(market_order);

                    
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
        Ok(order_id)
    }
    
}