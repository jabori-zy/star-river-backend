pub mod command_handler;
pub mod order_handler;
pub mod position_handler;
pub mod statistics_handler;
pub mod transaction_handler;

use std::{collections::HashMap, fmt::Debug, sync::Arc};

use key::{KeyTrait, KlineKey};
use snafu::{OptionExt, ResultExt};
use star_river_core::{custom_type::*, exchange::Exchange, kline::Kline, order::OrderType};
use tokio::sync::{Mutex, broadcast, mpsc};
use tokio_util::sync::CancellationToken;

// 外部的utils，不是当前crate的utils
use crate::{
    command::VtsCommand,
    event::{VirtualTradingSystemEventReceiver, VirtualTradingSystemEventSender, VtsEvent},
};
use crate::{
    error::{EventSendFailedSnafu, KlineKeyNotFoundSnafu, OrderNotFoundSnafu, VirtualTradingSystemError},
    types::{VirtualOrder, VirtualPosition, VirtualTransaction},
};

/// 虚拟交易系统
///
#[derive(Debug)]
pub struct VirtualTradingSystemContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    // current_datetime: DateTime<Utc>,       // 时间戳 (不是现实中的时间戳，而是回测时，播放到的k线的时间戳)
    kline_price: HashMap<KlineKey, Kline>, // k线缓存key，用于获取所有的k线缓存数据 缓存key -> (最新收盘价, 最新时间戳) 只获取min_interval_symbols中的k线缓存数据
    // kline_update_status: HashMap<KlineKey, bool>, // k线缓存key，用于记录k线是否已经更新
    // is_all_kline_updated: bool, // 是否所有k线都已更新
    event_transceiver: (broadcast::Sender<VtsEvent>, broadcast::Receiver<VtsEvent>),
    command_transceiver: (mpsc::Sender<VtsCommand>, Arc<Mutex<mpsc::Receiver<VtsCommand>>>),
    cancel_token: CancellationToken,
    kline_node_event_receiver: Vec<broadcast::Receiver<E>>,
    pub leverage: Leverage, // 杠杆

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
impl<E> VirtualTradingSystemContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel::<VtsEvent>(100);
        let (command_tx, command_rx) = mpsc::channel::<VtsCommand>(100);
        Self {
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
            orders: vec![],
            transactions: vec![],
            event_transceiver: (tx, rx),
            command_transceiver: (command_tx, Arc::new(Mutex::new(command_rx))),
            cancel_token: CancellationToken::new(),
        }
    }

    // pub fn current_datetime(&self) -> DateTime<Utc> {
    //     self.current_datetime
    // }

    pub fn get_vts_event_receiver(&self) -> VirtualTradingSystemEventReceiver {
        self.event_transceiver.1.resubscribe()
    }

    pub fn get_vts_event_publisher(&self) -> VirtualTradingSystemEventSender {
        self.event_transceiver.0.clone()
    }

    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    pub fn kline_node_event_receiver(&self) -> &Vec<broadcast::Receiver<E>> {
        &self.kline_node_event_receiver
    }

    pub fn get_kline_price(&self, kline_key: &KlineKey) -> Result<&Kline, VirtualTradingSystemError> {
        self.kline_price.get(kline_key).context(KlineKeyNotFoundSnafu {
            exchange: kline_key.get_exchange().to_string(),
            symbol: kline_key.get_symbol(),
        })
    }

    pub fn set_kline_price(&mut self, kline_price: HashMap<KlineKey, Kline>) {
        self.kline_price = kline_price;

        // self.kline_update_status = self.kline_price
        //     .keys()
        //     .map(|kline_key| (kline_key.clone(), false))
        //     .collect();
        // self.is_all_kline_updated = false;
    }

    pub fn kline_price(&self) -> &HashMap<KlineKey, Kline> {
        &self.kline_price
    }

    pub fn handle_kline_update(&mut self, kline_key: KlineKey, kline: Kline) {
        // if kline_key not in hashmap key, skip
        if !self.kline_price.contains_key(&kline_key) {
            return;
        }
        self.kline_price.entry(kline_key.clone()).and_modify(|e| *e = kline.clone());
        self.update_system(&kline_key, &kline).unwrap();
        // switch kline_update_status to true
        // self.kline_update_status.entry(kline_key).and_modify(|e| *e = true);
    }

    pub fn add_kline_node_event_receiver(&mut self, kline_node_event_receiver: broadcast::Receiver<E>) {
        self.kline_node_event_receiver.push(kline_node_event_receiver);
    }

    // pub fn current_datetime(&self) -> DateTime<Utc> {
    //     self.current_datetime
    // }

    pub fn balance(&self) -> Balance {
        self.balance
    }

    pub fn equity(&self) -> Equity {
        self.equity
    }

    pub fn available_balance(&self) -> Balance {
        self.available_balance
    }

    pub fn unrealized_pnl(&self) -> Pnl {
        self.unrealized_pnl
    }

    pub fn realized_pnl(&self) -> Pnl {
        self.realized_pnl
    }

    // pub fn get_play_index(&self) -> PlayIndex {
    //     *self.play_index_watch_rx.borrow()
    // }

    // 设置k线缓存索引, 并更新所有数据
    pub fn update_system(&mut self, kline_key: &KlineKey, kline: &Kline) -> Result<(), VirtualTradingSystemError> {
        // 当k线索引更新后，更新k线缓存key的最新收盘价
        // self.update_kline_price().await;
        // 更新时间戳
        // self.update_timestamp();

        self.check_unfilled_orders(&kline_key, &kline)?;
        // 价格更新后，更新仓位
        self.update_position(&kline_key, &kline)?;

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

    pub fn send_event(&self, event: VtsEvent) -> Result<usize, VirtualTradingSystemError> {
        self.event_transceiver.0.send(event).context(EventSendFailedSnafu {})
    }

    // async fn update_kline_price(&mut self) {
    //     let keys: Vec<KlineKey> = self.kline_price.keys().cloned().collect();

    //     let mut timestamp_list = vec![];
    //     for kline_key in keys {
    //         let kline = self.get_close_price(kline_key.clone()).await;
    //         if let Ok(kline) = kline {
    //             timestamp_list.push(kline.datetime);
    //             self.kline_price.entry(kline_key).and_modify(|e| *e = kline);
    //         }
    //     }

    //     // 检查完成后，需要检查所有k线的时间戳是否相同
    //     if timestamp_list.len() > 0 {
    //         let first_timestamp = timestamp_list[0];
    //         for timestamp in timestamp_list {
    //             if timestamp != first_timestamp {
    //                 tracing::warn!("k线时间戳不一致");
    //             }
    //         }
    //     }
    // }

    // pub fn update_datetime(&mut self, datetime: DateTime<Utc>) {
    //     // 获取所有k线的时间戳
    //     // let mut timestamp_list = vec![];
    //     // self.kline_price.iter().for_each(|(_, kline)| {
    //     //     timestamp_list.push(kline.datetime);
    //     // });

    //     // let min_timestamp = timestamp_list.iter().min();
    //     // if let Some(min_timestamp) = min_timestamp {
    //     //     self.current_datetime = *min_timestamp;
    //     // }
    //     // if self.current_datetime == datetime {
    //     //     return;
    //     // }
    //     // self.current_datetime = datetime;
    // }

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

    // 设置杠杆
    pub fn leverage(&mut self, leverage: Leverage) {
        self.leverage = leverage;
    }

    // 设置手续费率
    pub fn fee_rate(&mut self, fee_rate: FeeRate) {
        self.fee_rate = fee_rate;
    }

    // 获取初始资金
    pub fn initial_balance(&self) -> Balance {
        self.initial_balance
    }

    // 获取当前资金
    pub fn current_balance(&self) -> Balance {
        self.available_balance
    }

    // 获取保证金
    pub fn margin(&self) -> Margin {
        self.used_margin
    }

    // 获取所有订单
    pub fn get_orders(&self) -> &Vec<VirtualOrder> {
        &self.orders
    }

    pub fn get_order_by_id(&self, order_id: &OrderId) -> Result<&VirtualOrder, VirtualTradingSystemError> {
        self.orders
            .iter()
            .find(|order| &order.order_id == order_id)
            .context(OrderNotFoundSnafu { order_id: *order_id })
    }

    pub fn get_order_by_id_mut(&mut self, order_id: &OrderId) -> Result<&mut VirtualOrder, VirtualTradingSystemError> {
        self.orders
            .iter_mut()
            .find(|order| &order.order_id == order_id)
            .context(OrderNotFoundSnafu { order_id: *order_id })
    }

    pub fn get_take_profit_order(&self, position_id: PositionId) -> Option<&VirtualOrder> {
        self.orders
            .iter()
            .find(|order| order.position_id == Some(position_id) && order.order_type == OrderType::TakeProfitMarket)
    }

    pub fn get_stop_loss_order(&self, position_id: PositionId) -> Option<&VirtualOrder> {
        self.orders
            .iter()
            .find(|order| order.position_id == Some(position_id) && order.order_type == OrderType::StopMarket)
    }

    // 从缓存引擎获取k线数据
    // async fn get_close_price(&self, kline_key: KlineKey) -> Result<Kline, String> {
    //     let (resp_tx, resp_rx) = oneshot::channel();
    //     let payload = GetKlineDataCmdPayload::new(kline_key, Some(self.get_play_index()), Some(1));
    //     let cmd: BacktestStrategyCommand = GetKlineDataCommand::new("virtual_trading_system".to_string(), resp_tx, Some(payload)).into();
    //     self.get_strategy_command_sender().send(cmd.into()).await.unwrap();

    //     // 等待响应
    //     let response = resp_rx.await.unwrap();
    //     if response.is_success() {
    //         if response.kline_series.is_empty() {
    //             return Err("get cache data response is empty".to_string());
    //         }
    //         let kline = response.kline_series[0].clone();
    //         return Ok(kline);
    //     }
    //     Err("get history kline cache failed".to_string())
    // }

    // 根据交易所和symbol获取k线缓存key
    fn get_kline_key(&self, exchange: &Exchange, symbol: &String) -> Result<&KlineKey, VirtualTradingSystemError> {
        // tracing::debug!("kline_price keys: {:?}", self.kline_price.keys());
        self.kline_price()
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
        self.orders.clear();
        self.transactions.clear();
        self.available_balance = self.initial_balance;
        self.used_margin = 0.0;
    }

    // pub async fn listen_play_index(virtual_trading_system: Arc<Mutex<Self>>) -> Result<(), String> {
    //     let mut play_index_watch_rx = {
    //         let vts_guard = virtual_trading_system.lock().await;
    //         vts_guard.play_index_watch_rx.clone()
    //     };

    //     // 监听播放索引变化
    //     tokio::spawn(async move {
    //         loop {
    //             // 监听播放索引变化
    //             match play_index_watch_rx.changed().await {
    //                 Ok(_) => {
    //                     // 更新虚拟交易系统的播放索引
    //                     let mut vts_guard = virtual_trading_system.lock().await;
    //                     vts_guard.update_system().await;
    //                 }
    //                 Err(e) => {
    //                     tracing::error!("VirtualTradingSystem 监听播放索引错误: {}", e);
    //                 }
    //             }
    //         }
    //     });
    //     Ok(())
    // }
}
