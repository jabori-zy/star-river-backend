use types::cache::cache_key::BacktestKlineCacheKey;
use types::custom_type::*;
use types::position::virtual_position::VirtualPosition;
use types::transaction::virtual_transaction::VirtualTransaction;
use types::order::virtual_order::VirtualOrder;
use types::order::{OrderSide, OrderType, OrderStatus};
use chrono::Utc;
use std::collections::HashMap;

/// 虚拟交易系统
/// 
#[derive(Debug)]
pub struct VirtualTradingSystem {
    pub kline_cache_keys: Vec<BacktestKlineCacheKey>, // k线缓存key，用于获取所有的k线缓存数据
    kline_cache_index: u32, // k线缓存key的索引
    pub initial_balance: Balance, // 初始资金
    pub leverage: Leverage, // 杠杆
    pub current_balance: Balance, // 当前资金
    pub margin: Margin, // 保证金
    pub current_positions: Vec<VirtualPosition>, // 当前持仓
    pub current_orders: Vec<VirtualOrder>, // 当前挂单
    pub transaction_history: Vec<VirtualTransaction>, // 交易历史
}


// 虚拟交易系统get方法
impl VirtualTradingSystem {
    pub fn new() -> Self {
        Self {
            kline_cache_keys: vec![],
            kline_cache_index: 0,
            initial_balance: 0.0, 
            leverage: 0, 
            current_balance: 0.0,
            margin: 0.0, 
            current_positions: vec![], 
            current_orders: vec![],
            transaction_history: vec![]
        }
    }

    // 添加k线缓存key，只保留interval最小的那一个
    pub fn add_kline_cache_key(&mut self, kline_cache_key: BacktestKlineCacheKey) {
        // 判断CacheKey是否存在
        if !self.kline_cache_keys.contains(&kline_cache_key) {
            // 添加前，过滤出exchange, symbol, start_time, end_time相同的kline_cache_key
            let filtered_kline_cache_keys = self.kline_cache_keys
            .iter()
            .filter(|kline_cache_key| 
                kline_cache_key.exchange == kline_cache_key.exchange 
                && kline_cache_key.symbol == kline_cache_key.symbol
                 && kline_cache_key.start_time == kline_cache_key.start_time 
                 && kline_cache_key.end_time == 
                 kline_cache_key.end_time
                ).collect::<Vec<&BacktestKlineCacheKey>>();
            //比较interval，保留interval最小的那一个
            // 过滤出的列表长度一定为1，因为除了interval不同，其他都相同
            if filtered_kline_cache_keys.len() == 1 {
                // 比较要插入的key的interval和过滤出的key的interval
                // 如果要插入的key的interval小于过滤出的key的interval，则插入
                if kline_cache_key.interval < filtered_kline_cache_keys[0].interval {
                    self.kline_cache_keys.push(kline_cache_key);
                } 
                // 如果要插入的key的interval大于过滤出的key的interval，则不插入
                else {
                    tracing::warn!("{}: 要插入的k线缓存key的interval大于过滤出的k线缓存key的interval，不插入", kline_cache_key.symbol);
                }
            } else {
                self.kline_cache_keys.push(kline_cache_key);
            }
        }
    }

    pub fn get_kline_cache_index(&self) -> u32 {
        self.kline_cache_index
    }

    pub fn set_kline_cache_index(&mut self, kline_cache_index: u32) {
        self.kline_cache_index = kline_cache_index;
        tracing::debug!("virtual trading system kline cache index: {}", self.kline_cache_index);
    }

    // 设置初始资金
    pub fn set_initial_balance(&mut self, initial_balance: Balance) {
        self.initial_balance = initial_balance;
        self.current_balance = initial_balance;
    }

    // 设置杠杆
    pub fn set_leverage(&mut self, leverage: Leverage) {
        self.leverage = leverage;
    }

    // 获取初始资金
    pub fn get_initial_balance(&self) -> Balance {
        self.initial_balance
    }


    // 获取当前资金
    pub fn get_current_balance(&self) -> Balance {
        self.current_balance
    }

    // 获取保证金
    pub fn get_margin(&self) -> Margin {
        self.margin
    }

    // 获取杠杆
    pub fn get_leverage(&self) -> Leverage {
        self.leverage
    }

    // 获取当前持仓
    pub fn get_positions(&self) -> Vec<VirtualPosition> {
        self.current_positions.clone()
    }

    pub fn get_orders(&self) -> Vec<VirtualOrder> {
        self.current_orders.clone()
    }

    pub fn get_order(&self, order_id: OrderId) -> Option<VirtualOrder> {
        self.current_orders.iter().find(|order| order.order_id == order_id).cloned()
    }

    // 获取交易历史
    pub fn get_transaction_history(&self) -> Vec<VirtualTransaction> {
        self.transaction_history.clone()
    }
}



impl VirtualTradingSystem {
    // 生成订单ID, 从0开始
    fn generate_order_id(&self) -> i32 {
        self.current_orders.len() as i32
    }
    // 创建订单
    pub fn create_order(&mut self,
        strategy_id: i32,
        node_id: String,
        symbol: String,
        price: f64,
        order_side: OrderSide,
        order_type: OrderType,
        quantity: f64,
        tp: Option<f64>,
        sl: Option<f64>,
    ) -> OrderId {
        let order_id = self.generate_order_id();
        let order = VirtualOrder {
            order_id,
            strategy_id,
            node_id,
            symbol,
            order_side,
            order_type,
            quantity,
            open_price: price,
            tp,
            sl,
            order_status: OrderStatus::Created,
            created_time: Utc::now(),
            updated_time: Utc::now(),
        };
        self.current_orders.push(order);
        order_id
    }
    
}





