pub mod command_handler;
pub mod order_handler;
pub mod position_handler;
pub mod statistics_handler;
pub mod transaction_handler;

use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, atomic::Ordering},
};

use chrono::{DateTime, Utc};
use key::{KeyTrait, KlineKey};
use snafu::{OptionExt, ResultExt};
use star_river_core::{custom_type::*, exchange::Exchange, kline::Kline, order::OrderType};
use tokio::sync::{Mutex, broadcast, mpsc, watch};
use tokio_util::sync::CancellationToken;

// 外部的utils，不是当前crate的utils
use crate::{
    command::VtsCommand,
    event::{VirtualTradingSystemEventReceiver, VirtualTradingSystemEventSender, VtsEvent},
};
use crate::{
    error::{EventSendFailedSnafu, KlineKeyNotFoundSnafu, OrderNotFoundSnafu, VtsError},
    types::{
        VirtualOrder, VirtualPosition, VirtualTransaction,
        id_generator::{ORDER_ID_COUNTER, POSITION_ID_COUNTER, TRANSACTION_ID_COUNTER},
    },
};

/// 虚拟交易系统
///
#[derive(Debug)]
pub struct VirtualTradingSystemContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    strategy_time_watch_rx: watch::Receiver<DateTime<Utc>>,

    event_transceiver: (broadcast::Sender<VtsEvent>, broadcast::Receiver<VtsEvent>),
    command_transceiver: (mpsc::Sender<VtsCommand>, Arc<Mutex<mpsc::Receiver<VtsCommand>>>),
    cancel_token: CancellationToken,
    kline_node_event_receiver: Vec<broadcast::Receiver<E>>,
    pub leverage: Leverage, // 杠杆

    pub kline_price: HashMap<KlineKey, Kline>, // k线缓存key，用于获取所有的k线缓存数据 缓存key -> (最新收盘价, 最新时间戳) 只获取min_interval_symbols中的k线缓存数据

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
    pub unfilled_orders: Vec<VirtualOrder>,      // 所有订单(未成交订单)
    pub history_orders: Vec<VirtualOrder>,       // 历史订单(已成交订单)
    pub transactions: Vec<VirtualTransaction>,   // 交易历史
}

// 虚拟交易系统get方法
impl<E> VirtualTradingSystemContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    pub fn new(strategy_time_watch_rx: watch::Receiver<DateTime<Utc>>) -> Self {
        let (tx, rx) = broadcast::channel::<VtsEvent>(100);
        let (command_tx, command_rx) = mpsc::channel::<VtsCommand>(100);
        Self {
            strategy_time_watch_rx,
            kline_price: HashMap::new(),
            kline_node_event_receiver: vec![],
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
            unfilled_orders: vec![],
            history_orders: vec![],
            transactions: vec![],
            event_transceiver: (tx, rx),
            command_transceiver: (command_tx, Arc::new(Mutex::new(command_rx))),
            cancel_token: CancellationToken::new(),
        }
    }

    pub fn current_datetime(&self) -> DateTime<Utc> {
        *self.strategy_time_watch_rx.borrow()
    }

    pub fn vts_event_receiver(&self) -> VirtualTradingSystemEventReceiver {
        self.event_transceiver.1.resubscribe()
    }

    pub fn vts_event_publisher(&self) -> VirtualTradingSystemEventSender {
        self.event_transceiver.0.clone()
    }

    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    pub fn kline_node_event_receiver(&self) -> &Vec<broadcast::Receiver<E>> {
        &self.kline_node_event_receiver
    }

    pub fn get_kline_price(&self, kline_key: &KlineKey) -> Result<&Kline, VtsError> {
        self.kline_price.get(kline_key).context(KlineKeyNotFoundSnafu {
            exchange: kline_key.exchange().to_string(),
            symbol: kline_key.symbol(),
        })
    }

    pub fn set_kline_price(&mut self, kline_price: HashMap<KlineKey, Kline>) {
        self.kline_price = kline_price;
    }

    pub fn handle_kline_update(&mut self, kline_key: KlineKey, kline: Kline) {
        // if kline_key not in hashmap key, skip
        if !self.kline_price.contains_key(&kline_key) {
            return;
        }
        self.kline_price.entry(kline_key.clone()).and_modify(|e| *e = kline.clone());
        self.update_system(&kline_key, &kline).unwrap();
    }

    pub fn add_kline_node_event_receiver(&mut self, kline_node_event_receiver: broadcast::Receiver<E>) {
        self.kline_node_event_receiver.push(kline_node_event_receiver);
    }

    // 设置k线缓存索引, 并更新所有数据
    pub fn update_system(&mut self, kline_key: &KlineKey, kline: &Kline) -> Result<(), VtsError> {
        self.check_unfilled_orders(&kline_key, &kline)?;
        // 价格更新后，更新仓位
        self.update_current_positions(&kline_key, &kline)?;

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
        self.send_event(VtsEvent::UpdateFinished)?;
        Ok(())
    }

    pub fn send_event(&self, event: VtsEvent) -> Result<usize, VtsError> {
        self.event_transceiver.0.send(event).context(EventSendFailedSnafu {})
    }

    // 设置初始资金
    pub fn set_initial_balance(&mut self, initial_balance: Balance) {
        self.initial_balance = initial_balance;
        self.available_balance = initial_balance;
    }

    pub fn set_leverage(&mut self, leverage: Leverage) {
        self.leverage = leverage;
    }

    pub fn set_fee_rate(&mut self, fee_rate: FeeRate) {
        self.fee_rate = fee_rate;
    }

    // 根据交易所和symbol获取k线缓存key
    fn get_kline_key(&self, exchange: &Exchange, symbol: &String) -> Result<&KlineKey, VtsError> {
        // tracing::debug!("kline_price keys: {:?}", self.kline_price.keys());
        self.kline_price
            .keys()
            .find(|kline_key| &kline_key.exchange == exchange && &kline_key.symbol == symbol)
            .context(KlineKeyNotFoundSnafu {
                exchange: exchange.to_string(),
                symbol: symbol.clone(),
            })
    }

    // 重置系统
    // 清空所有仓位和订单
    pub fn reset(&mut self) {
        self.current_positions.clear();
        self.history_positions.clear();
        self.unfilled_orders.clear();
        self.history_orders.clear();
        self.transactions.clear();
        self.available_balance = self.initial_balance;
        self.used_margin = 0.0;
        ORDER_ID_COUNTER.store(0, Ordering::SeqCst);
        POSITION_ID_COUNTER.store(0, Ordering::SeqCst);
        TRANSACTION_ID_COUNTER.store(0, Ordering::SeqCst);
    }
}
