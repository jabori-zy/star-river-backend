pub mod order;
pub mod position;
pub mod transaction;
pub mod statistics;
pub(crate) mod utils;

use types::order::OrderType;
use types::cache::key::KlineKey;
use types::cache::Key;
use types::custom_type::*;
use types::position::virtual_position::VirtualPosition;
use types::transaction::virtual_transaction::VirtualTransaction;
use types::order::virtual_order::VirtualOrder;
use types::order::OrderStatus;
use event_center::CommandPublisher;
use tokio::sync::oneshot;
use event_center::command::cache_engine_command::{CacheEngineCommand, GetCacheParams};
use event_center::response::cache_engine_response::CacheEngineResponse;
// 外部的utils，不是当前crate的utils
use ::utils::get_utc8_timestamp_millis;
use std::collections::HashMap;
use types::market::Exchange;
use types::virtual_trading_system::event::{VirtualTradingSystemEvent, VirtualTradingSystemEventSender};
use types::custom_type::PlayIndex;
use tokio::sync::Mutex;
use std::sync::Arc;

/// 虚拟交易系统
/// 
#[derive(Debug)]
pub struct VirtualTradingSystem {
    timestamp: i64, // 时间戳 (不是现实中的时间戳，而是回测时，播放到的k线的时间戳)
    kline_price: HashMap<KlineKey, (f64, i64)>, // k线缓存key，用于获取所有的k线缓存数据 缓存key -> (最新收盘价, 最新时间戳)
    pub command_publisher: CommandPublisher, // 命令发布者
    pub event_publisher: VirtualTradingSystemEventSender, // 事件发布者
    pub play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>, // 播放索引监听器


    pub initial_balance: Balance, // 初始资金
    pub leverage: Leverage, // 杠杆
    pub current_balance: Balance, // 当前可用资金(当前可用资金 = 当前资金 - 冻结资金)
    pub unrealized_pnl: UnrealizedPnl, // 未实现盈亏
    pub pnl: Pnl, //已实现盈亏

    // 保证金相关
    pub margin: Margin, // 保证金
    pub frozen_margin: FrozenMargin, // 冻结保证金
    pub margin_ratio: MarginRatio, // 保证金率

    // 手续费相关
    pub fee_rate: FeeRate, // 手续费率


    // 持仓相关
    pub current_positions: Vec<VirtualPosition>, // 当前持仓
    pub history_positions: Vec<VirtualPosition>, // 历史持仓
    pub orders: Vec<VirtualOrder>, // 所有订单(已成交订单 + 未成交订单)
    pub transactions: Vec<VirtualTransaction>, // 交易历史
    
}


// 虚拟交易系统get方法
impl VirtualTradingSystem {
    pub fn new(
        command_publisher: CommandPublisher, 
        event_publisher: VirtualTradingSystemEventSender,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Self {
        Self {
            timestamp: 0,
            kline_price: HashMap::new(),
            initial_balance: 0.0,
            leverage: 0, 
            current_balance: 0.0,
            unrealized_pnl: 0.0,
            pnl: 0.0,
            margin: 0.0,
            frozen_margin: 0.0,
            margin_ratio: 0.0,
            fee_rate: 0.0,
            current_positions: vec![],
            history_positions: vec![],
            orders: vec![],
            transactions: vec![],
            command_publisher,
            event_publisher,
            play_index_watch_rx,
        }
    }

    // 添加k线缓存key，只保留interval最小的那一个
    pub fn add_kline_cache_key(&mut self, kline_key: KlineKey) {
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
                ).collect::<Vec<&KlineKey>>();
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

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn get_unrealized_pnl(&self) -> UnrealizedPnl {
        self.unrealized_pnl
    }

    pub fn get_play_index(&self) -> PlayIndex {
        *self.play_index_watch_rx.borrow()
    }

    // 设置k线缓存索引, 并更新所有数据
    pub async fn update_system(&mut self) {
        // 当k线索引更新后，更新k线缓存key的最新收盘价
        self.update_kline_price().await;
        // 更新时间戳
        self.update_timestamp();

        self.check_unfilled_orders();
        // 价格更新后，更新仓位
        self.update_position();
        
        // 更新未实现盈亏
        self.update_unrealized_pnl();
        // 发送事件
        self.event_publisher.send(VirtualTradingSystemEvent::UpdateFinished).unwrap();
        
        
    }


    async fn update_kline_price(&mut self) {
        let keys: Vec<KlineKey> = self.kline_price.keys().cloned().collect();

        let mut timestamp_list = vec![];
        for kline_cache_key in keys {
            let (close_price, timestamp) = self.get_close_price(kline_cache_key.clone().into()).await.unwrap();
            timestamp_list.push(timestamp);
            self.kline_price.entry(kline_cache_key).and_modify(|e| *e = (close_price, timestamp));
        }

        // 检查完成后，需要检查所有k线的时间戳是否相同
        if timestamp_list.len() > 0 {
            let first_timestamp = timestamp_list[0];
            for timestamp in timestamp_list {
                if timestamp != first_timestamp {
                    tracing::warn!("k线时间戳不一致");
                }
            }
        }
    }

    fn update_timestamp(&mut self) {
        // 获取所有k线的时间戳
        let mut timestamp_list = vec![];
        self.kline_price.iter().for_each(|(_, (_, timestamp))| {
            timestamp_list.push(timestamp.clone());
        });
        
        let min_timestamp = timestamp_list.iter().min();
        if let Some(min_timestamp) = min_timestamp {
            self.timestamp = *min_timestamp;
        }
    }

    

    // 设置初始资金
    pub fn set_initial_balance(&mut self, initial_balance: Balance) {
        self.initial_balance = initial_balance;
        self.current_balance = initial_balance;
        tracing::debug!("set_initial_balance: {}", initial_balance);
    }

    // 设置杠杆
    pub fn set_leverage(&mut self, leverage: Leverage) {
        self.leverage = leverage;
    }

    // 设置手续费率
    pub fn set_fee_rate(&mut self, fee_rate: FeeRate) {
        self.fee_rate = fee_rate;
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
    pub fn get_current_positions_ref(&self) -> &Vec<VirtualPosition> {
        &self.current_positions
    }


    // 获取所有订单
    pub fn get_orders(&self) -> &Vec<VirtualOrder> {
        &self.orders
    }

    

    pub fn get_order(&self, order_id: OrderId) -> Option<&VirtualOrder> {
        self.orders.iter().find(|order| order.order_id == order_id)
    }
    
    pub fn get_take_profit_order(&self, position_id: PositionId) -> Option<&VirtualOrder> {
        self.orders.iter().find(|order| order.position_id == Some(position_id) && order.order_type == OrderType::TakeProfitMarket)
    }
    
    pub fn get_stop_loss_order(&self, position_id: PositionId) -> Option<&VirtualOrder> {
        self.orders.iter().find(|order| order.position_id == Some(position_id) && order.order_type == OrderType::StopMarket)
    }

    

    // 从缓存引擎获取k线数据
    async fn get_close_price(&self,kline_cache_key: Key) -> Result<(f64, i64), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = GetCacheParams {
            strategy_id: -1,
            node_id: "virtual_trading_system".to_string(),
            key: kline_cache_key,
            index: Some(self.get_play_index() as u32),
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
    fn get_kline_key(&self, exchange: &Exchange, symbol: &String) -> Option<KlineKey> {
        for kline_key in self.kline_price.keys() {
            if &kline_key.exchange == exchange && &kline_key.symbol == symbol {
                return Some(kline_key.clone());
            }
        }
        None
    }
    
    // 重置系统
    // 清空所有仓位和订单
    pub fn reset(&mut self) {
        self.current_positions.clear();
        self.history_positions.clear();
        self.orders.clear();
        self.transactions.clear();
        self.current_balance = self.initial_balance;
        self.margin = 0.0;
    }



    pub async fn listen_play_index(virtual_trading_system: Arc<Mutex<Self>>) -> Result<(), String> {
        let mut play_index_watch_rx = {
            let vts_guard = virtual_trading_system.lock().await;
            vts_guard.play_index_watch_rx.clone()
        };

        // 监听播放索引变化
        tokio::spawn(async move {
            loop {
                // 监听播放索引变化
                match play_index_watch_rx.changed().await {
                    Ok(_) => {
                        // 更新虚拟交易系统的播放索引
                        let mut vts_guard = virtual_trading_system.lock().await;
                        vts_guard.update_system().await;
                    }
                    Err(e) => {
                        tracing::error!("VirtualTradingSystem 监听播放索引错误: {}", e);
                    }
                }
            }
        });
        Ok(())
    }
}