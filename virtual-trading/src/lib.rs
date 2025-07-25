pub mod order;
pub mod position;
pub mod transaction;

use types::cache::key::BacktestKlineKey;
use types::cache::{Key, CacheValue};
use types::custom_type::*;
use types::position::virtual_position::VirtualPosition;
use types::transaction::virtual_transaction::VirtualTransaction;
use types::order::virtual_order::VirtualOrder;
use types::order::OrderStatus;
use event_center::CommandPublisher;
use tokio::sync::oneshot;
use event_center::command::cache_engine_command::{CacheEngineCommand, GetCacheParams};
use event_center::response::cache_engine_response::CacheEngineResponse;
use utils::get_utc8_timestamp_millis;
use std::collections::HashMap;
use types::market::Exchange;
use types::virtual_trading_system::event::VirtualTradingSystemEventSender;

/// 虚拟交易系统
/// 
#[derive(Debug)]
pub struct VirtualTradingSystem {
    pub kline_price: HashMap<BacktestKlineKey, (f64, i64)>, // k线缓存key，用于获取所有的k线缓存数据 缓存key -> (最新收盘价, 最新时间戳)
    play_index: i32, // k线缓存key的索引
    pub initial_balance: Balance, // 初始资金
    pub leverage: Leverage, // 杠杆
    pub current_balance: Balance, // 当前资金
    pub margin: Margin, // 保证金
    pub current_positions: Vec<VirtualPosition>, // 当前持仓
    pub orders: Vec<VirtualOrder>, // 所有订单(已成交订单 + 未成交订单)
    pub transactions: Vec<VirtualTransaction>, // 交易历史
    pub command_publisher: CommandPublisher, // 命令发布者
    pub event_publisher: VirtualTradingSystemEventSender, // 事件发布者
}


// 虚拟交易系统get方法
impl VirtualTradingSystem {
    pub fn new(command_publisher: CommandPublisher, event_publisher: VirtualTradingSystemEventSender) -> Self {
        Self {
            kline_price: HashMap::new(),
            play_index: 0,
            initial_balance: 0.0, 
            leverage: 0, 
            current_balance: 0.0,
            margin: 0.0, 
            current_positions: vec![],
            orders: vec![],
            transactions: vec![],
            command_publisher,
            event_publisher,
        }
    }

    // 添加k线缓存key，只保留interval最小的那一个
    pub fn add_kline_cache_key(&mut self, kline_key: BacktestKlineKey) {
        // 判断CacheKey是否存在
        if !self.kline_price.contains_key(&kline_key) {
            // 添加前，过滤出exchange, symbol, start_time, end_time相同的kline_cache_key
            let filtered_kline_cache_keys = self.kline_price
            .keys()
            .filter(|key|
                key.exchange == kline_key.exchange
                && key.symbol == kline_key.symbol
                 && key.start_time == kline_key.start_time
                 && key.end_time == kline_key.end_time
                ).collect::<Vec<&BacktestKlineKey>>();
            //比较interval，保留interval最小的那一个
            // 过滤出的列表长度一定为1，因为除了interval不同，其他都相同
            if filtered_kline_cache_keys.len() == 1 {
                // 比较要插入的key的interval和过滤出的key的interval
                // 如果要插入的key的interval小于过滤出的key的interval，则插入
                if kline_key.interval < filtered_kline_cache_keys[0].interval {
                    self.kline_price.insert(kline_key, (0.0, 0));
                } 
                // 如果要插入的key的interval大于过滤出的key的interval，则不插入
                else {
                    tracing::warn!("{}: 要插入的k线缓存key的interval大于过滤出的k线缓存key的interval，不插入", kline_key.symbol);
                }
            } 
            // 如果过滤出的列表长度为0，则直接插入
            else {
                self.kline_price.insert(kline_key, (0.0, 0));
            }
        }
    }

    pub fn get_play_index(&self) -> i32 {
        self.play_index
    }

    // 设置k线缓存索引
    pub async fn set_play_index(&mut self, play_index: i32) {
        self.play_index = play_index;
        // 当k线索引更新后，更新k线缓存key的最新收盘价
        let keys: Vec<BacktestKlineKey> = self.kline_price.keys().cloned().collect();
        for kline_cache_key in keys {
            let (close_price, timestamp) = self.get_close_price(kline_cache_key.clone().into()).await.unwrap();
            self.kline_price.entry(kline_cache_key).and_modify(|e| *e = (close_price, timestamp));
        }
        // 更新仓位
        self.update_position();
        tracing::debug!("持仓: {:?}", self.current_positions);
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
    pub fn get_positions(&self) -> &Vec<VirtualPosition> {
        &self.current_positions
    }

    // 获取当前持仓数量
    pub fn get_position_number(&self) -> u32 {
        self.current_positions.len() as u32
    }

    // 获取某个symbol的持仓数量
    pub fn get_symbol_position_number(&self, symbol: &String, exchange: &Exchange) -> u32 {
        self.current_positions.iter().filter(|position| &position.symbol == symbol && &position.exchange == exchange).count() as u32
    }

    // 获取所有订单
    pub fn get_orders(&self) -> &Vec<VirtualOrder> {
        &self.orders
    }

    // 获取未成交订单
    pub fn get_unfilled_orders(&self) -> Vec<&VirtualOrder> {
        self.orders
            .iter()
            .filter(|order| order.order_status == OrderStatus::Created || order.order_status == OrderStatus::Placed)
            .collect::<Vec<&VirtualOrder>>()
    }

    pub fn get_order(&self, order_id: OrderId) -> Option<&VirtualOrder> {
        self.orders.iter().find(|order| order.order_id == order_id)
    }

    // 获取交易历史
    pub fn get_transactions(&self) -> &Vec<VirtualTransaction> {
        &self.transactions
    }

    // 从缓存引擎获取k线数据
    async fn get_close_price(&self,kline_cache_key: Key) -> Result<(f64, i64), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = GetCacheParams {
            strategy_id: -1,
            node_id: "virtual_trading_system".to_string(),
            key: kline_cache_key.clone(),
            index: Some(self.play_index as u32),
            limit: Some(1),
            sender: "virtual_trading_system".to_string(),
            timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
            };

            let get_cache_command = CacheEngineCommand::GetCache(params);
            self.command_publisher.send(get_cache_command.into()).await.unwrap();
    
            // 等待响应
            let response = resp_rx.await.unwrap();
            if response.code() == 0 {
                if let Ok(CacheEngineResponse::GetCacheData(get_cache_data_response)) = CacheEngineResponse::try_from(response) {
                    let close = get_cache_data_response.cache_data[0].as_kline().unwrap().close;
                    let timestamp = get_cache_data_response.cache_data[0].as_kline().unwrap().timestamp;
                    return Ok((close, timestamp));
                }
            }
            Err("get history kline cache failed".to_string())
        }


    // 根据交易所和symbol获取k线缓存key
    fn get_kline_cache_key(&self, exchange: &Exchange, symbol: &String) -> Option<BacktestKlineKey> {
        tracing::debug!("get_kline_cache_key: {:?}", self.kline_price);
        for kline_cache_key in self.kline_price.keys() {
            if &kline_cache_key.exchange == exchange && &kline_cache_key.symbol == symbol {
                return Some(kline_cache_key.clone());
            }
        }
        None
    }
}









