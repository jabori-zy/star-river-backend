pub mod order;
pub mod position;
pub mod statistics;
pub mod transaction;
pub(crate) mod utils;

use event_center::communication::engine::cache_engine::{
    CacheEngineCommand, CacheEngineResponse, GetCacheParams,
};
use star_river_core::cache::key::KlineKey;
use star_river_core::cache::Key;
use star_river_core::custom_type::*;
use star_river_core::order::virtual_order::VirtualOrder;
use star_river_core::order::OrderType;
use star_river_core::position::virtual_position::VirtualPosition;
use star_river_core::transaction::virtual_transaction::VirtualTransaction;
use tokio::sync::oneshot;
// 外部的utils，不是当前crate的utils
use event_center::EventCenterSingleton;
use star_river_core::custom_type::PlayIndex;
use star_river_core::market::Exchange;
use star_river_core::market::Kline;
use star_river_core::virtual_trading_system::event::{
    VirtualTradingSystemEvent, VirtualTradingSystemEventReceiver, VirtualTradingSystemEventSender,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};

/// 虚拟交易系统
///
#[derive(Debug)]
pub struct VirtualTradingSystem {
    current_datetime: DateTime<Utc>, // 时间戳 (不是现实中的时间戳，而是回测时，播放到的k线的时间戳)
    kline_price: HashMap<KlineKey, Kline>, // k线缓存key，用于获取所有的k线缓存数据 缓存key -> (最新收盘价, 最新时间戳)
    // pub command_publisher: CommandPublisher, // 命令发布者
    pub event_publisher: VirtualTradingSystemEventSender, // 事件发布者
    pub event_receiver: VirtualTradingSystemEventReceiver, // 事件接收器
    pub play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>, // 播放索引监听器
    pub leverage: Leverage,                               // 杠杆

    // 资金相关
    pub initial_balance: Balance,   // 初始资金
    pub balance: Balance,           // 账户余额(账户余额 = 初始资金 + 已实现盈亏)
    pub available_balance: Balance, // 可用余额(可用余额 = 净值 - 已用保证金 - 冻结保证金)
    pub equity: Equity,             // 净值(净值 = 账户余额 + 未实现盈亏)

    // 盈亏相关
    pub realized_pnl: Pnl,   // 已实现盈亏
    pub unrealized_pnl: Pnl, // 未实现盈亏

    // 保证金相关
    pub used_margin: Margin,       // 已用保证金
    pub frozen_margin: Margin,     // 冻结保证金（挂单的订单占用的保证金）
    pub margin_ratio: MarginRatio, // 保证金率

    // 手续费相关
    pub fee_rate: FeeRate, // 手续费率

    // 持仓相关
    pub current_positions: Vec<VirtualPosition>, // 当前持仓
    pub history_positions: Vec<VirtualPosition>, // 历史持仓
    pub orders: Vec<VirtualOrder>,               // 所有订单(已成交订单 + 未成交订单)
    pub transactions: Vec<VirtualTransaction>,   // 交易历史
}

// 虚拟交易系统get方法
impl VirtualTradingSystem {
    pub fn new(play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>) -> Self {
        let (virtual_trading_system_event_tx, virtual_trading_system_event_rx) =
            broadcast::channel::<VirtualTradingSystemEvent>(100);
        Self {
            current_datetime: Utc::now(),
            kline_price: HashMap::new(),
            initial_balance: 0.0,
            balance: 0.0,
            available_balance: 0.0,
            equity: 0.0,
            leverage: 0,
            realized_pnl: 0.0,
            unrealized_pnl: 0.0,
            used_margin: 0.0,
            frozen_margin: 0.0,
            margin_ratio: 0.0,
            fee_rate: 0.0,
            current_positions: vec![],
            history_positions: vec![],
            orders: vec![],
            transactions: vec![],
            event_publisher: virtual_trading_system_event_tx,
            event_receiver: virtual_trading_system_event_rx,
            play_index_watch_rx,
        }
    }

    pub fn get_virtual_trading_system_event_receiver(&self) -> VirtualTradingSystemEventReceiver {
        self.event_receiver.resubscribe()
    }

    pub fn get_virtual_trading_system_event_publisher(&self) -> VirtualTradingSystemEventSender {
        self.event_publisher.clone()
    }

    // 添加k线缓存key，只保留interval最小的那一个
    pub fn add_kline_key(&mut self, kline_key: KlineKey) {
        // 判断CacheKey是否存在
        if !self.kline_price.contains_key(&kline_key) {
            // 添加前，过滤出exchange, symbol, start_time, end_time相同的kline_key
            let filtered_kline_keys = self
                .kline_price
                .keys()
                .filter(|key| {
                    key.exchange == kline_key.exchange
                        && key.symbol == kline_key.symbol
                        && key.start_time == kline_key.start_time
                        && key.end_time == kline_key.end_time
                })
                .collect::<Vec<&KlineKey>>();
            //比较interval，保留interval最小的那一个
            // 过滤出的列表长度一定为1，因为除了interval不同，其他都相同
            if filtered_kline_keys.len() == 1 {
                // 比较要插入的key的interval和过滤出的key的interval
                // 如果要插入的key的interval小于过滤出的key的interval，则插入
                if kline_key.interval < filtered_kline_keys[0].interval {
                    self.kline_price.insert(
                        kline_key,
                        Kline {
                            datetime: Utc::now(),
                            open: 0.0,
                            high: 0.0,
                            low: 0.0,
                            close: 0.0,
                            volume: 0.0,
                        },
                    );
                }
                // 如果要插入的key的interval大于过滤出的key的interval，则不插入
                else {
                    tracing::warn!(
                        "{}: 要插入的k线缓存key的interval大于过滤出的k线缓存key的interval，不插入",
                        kline_key.symbol
                    );
                }
            }
            // 如果过滤出的列表长度为0，则直接插入
            else {
                self.kline_price.insert(
                    kline_key,
                    Kline {
                        datetime: Utc::now(),
                        open: 0.0,
                        high: 0.0,
                        low: 0.0,
                        close: 0.0,
                        volume: 0.0,
                    },
                );
            }
        }
    }

    pub fn get_datetime(&self) -> DateTime<Utc> {
        self.current_datetime
    }

    pub fn get_balance(&self) -> Balance {
        self.balance
    }

    pub fn get_equity(&self) -> Equity {
        self.equity
    }

    pub fn get_available_balance(&self) -> Balance {
        self.available_balance
    }

    pub fn get_unrealized_pnl(&self) -> Pnl {
        self.unrealized_pnl
    }

    pub fn get_realized_pnl(&self) -> Pnl {
        self.realized_pnl
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

        // 按正确顺序更新余额相关数据
        // 1. 更新已实现盈亏
        self.update_realized_pnl();
        // 2. 更新未实现盈亏
        self.update_unrealized_pnl();
        // 3. 更新已用保证金
        self.update_used_margin();
        // 4. 更新冻结保证金
        self.update_frozen_margin();
        // 5. 更新账户余额
        self.update_balance();
        // 6. 更新净值
        self.update_equity();
        // 7. 更新可用余额
        self.update_available_balance();
        // 8. 更新保证金率
        self.update_margin_ratio();

        // 发送事件
        self.event_publisher
            .send(VirtualTradingSystemEvent::UpdateFinished)
            .unwrap();
    }

    async fn update_kline_price(&mut self) {
        let keys: Vec<KlineKey> = self.kline_price.keys().cloned().collect();

        let mut timestamp_list = vec![];
        for kline_key in keys {
            let kline = self
                .get_close_price(kline_key.clone().into())
                .await
                .unwrap();
            timestamp_list.push(kline.datetime);
            self.kline_price.entry(kline_key).and_modify(|e| *e = kline);
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
        self.kline_price.iter().for_each(|(_, kline)| {
            timestamp_list.push(kline.datetime);
        });

        let min_timestamp = timestamp_list.iter().min();
        if let Some(min_timestamp) = min_timestamp {
            self.current_datetime = *min_timestamp;
        }
    }

    // 设置初始资金
    pub fn set_initial_balance(&mut self, initial_balance: Balance) {
        self.initial_balance = initial_balance;
        self.available_balance = initial_balance;
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
        self.available_balance
    }

    // 获取保证金
    pub fn get_margin(&self) -> Margin {
        self.used_margin
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
        self.orders.iter().find(|order| {
            order.position_id == Some(position_id)
                && order.order_type == OrderType::TakeProfitMarket
        })
    }

    pub fn get_stop_loss_order(&self, position_id: PositionId) -> Option<&VirtualOrder> {
        self.orders.iter().find(|order| {
            order.position_id == Some(position_id) && order.order_type == OrderType::StopMarket
        })
    }

    // 从缓存引擎获取k线数据
    async fn get_close_price(&self, kline_key: Key) -> Result<Kline, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = GetCacheParams::new(
            -1,
            "virtual_trading_system".to_string(),
            kline_key,
            Some(self.get_play_index() as u32),
            Some(1),
            "virtual_trading_system".to_string(),
            resp_tx,
        );

        let get_cache_command = CacheEngineCommand::GetCache(params);
        // self.command_publisher.send(get_cache_command.into()).await.unwrap();
        EventCenterSingleton::send_command(get_cache_command.into())
            .await
            .unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.success() {
            if let Ok(CacheEngineResponse::GetCacheData(get_cache_data_response)) = CacheEngineResponse::try_from(response)
            {
                if get_cache_data_response.cache_data.is_empty() {
                    return Err("get cache data response is empty".to_string());
                }
                let kline = get_cache_data_response.cache_data[0].as_kline().unwrap();
                return Ok(kline);
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
        self.available_balance = self.initial_balance;
        self.used_margin = 0.0;
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
