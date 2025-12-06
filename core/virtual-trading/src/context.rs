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
use snafu::{OptionExt, ResultExt};
use star_river_core::{custom_type::*, exchange::Exchange, kline::Kline};
use tokio::sync::{Mutex, broadcast, mpsc, watch};
use tokio_util::sync::CancellationToken;

// External utils, not from current crate
use crate::{
    command::VtsCommand,
    event::{VtsEvent, VtsEventReceiver, VtsEventSender},
};
use crate::{
    error::{EventSendFailedSnafu, KlineKeyNotFoundSnafu, VtsError},
    types::{
        VirtualOrder, VirtualPosition, VirtualTransaction,
        id_generator::{ORDER_ID_COUNTER, POSITION_ID_COUNTER, TRANSACTION_ID_COUNTER},
    },
};

/// Virtual Trading System
///
#[derive(Debug)]
pub struct VtsContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    strategy_time_watch_rx: watch::Receiver<DateTime<Utc>>,

    event_sender: broadcast::Sender<VtsEvent>,
    command_transceiver: (mpsc::Sender<VtsCommand>, Arc<Mutex<mpsc::Receiver<VtsCommand>>>),
    cancel_token: CancellationToken,
    kline_node_event_receiver: Vec<broadcast::Receiver<E>>,
    pub leverage: Leverage, // Leverage

    pub kline_price: HashMap<(Exchange, String), Kline>, // Kline cache key for getting all kline cache data, cache key -> (latest close price, latest timestamp), only get kline data from min_interval_symbols

    // Fund related
    pub initial_balance: Balance,   // Initial balance
    pub balance: Balance,           // Account balance (account balance = initial balance + realized pnl)
    pub available_balance: Balance, // Available balance (available balance = equity - used margin - frozen margin)
    pub equity: Equity,             // Equity (equity = account balance + unrealized pnl)

    // PnL related
    pub realized_pnl: Pnl,   // Realized profit/loss
    pub unrealized_pnl: Pnl, // Unrealized profit/loss

    // Margin related
    pub used_margin: Margin,       // Used margin
    pub frozen_margin: Margin,     // Frozen margin (margin occupied by pending orders)
    pub margin_ratio: MarginRatio, // Margin ratio

    // Fee related
    pub fee_rate: FeeRate, // Fee rate

    // Position related
    pub current_positions: Vec<VirtualPosition>, // Current positions
    pub history_positions: Vec<VirtualPosition>, // History positions
    pub unfilled_orders: Vec<VirtualOrder>,      // All orders (unfilled orders)
    pub history_orders: Vec<VirtualOrder>,       // History orders (filled orders)
    pub transactions: Vec<VirtualTransaction>,   // Transaction history
}

// Virtual Trading System getter methods
impl<E> VtsContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    pub fn new(strategy_time_watch_rx: watch::Receiver<DateTime<Utc>>) -> Self {
        let (tx, _) = broadcast::channel::<VtsEvent>(100);
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
            event_sender: tx,
            command_transceiver: (command_tx, Arc::new(Mutex::new(command_rx))),
            cancel_token: CancellationToken::new(),
        }
    }

    pub fn current_datetime(&self) -> DateTime<Utc> {
        *self.strategy_time_watch_rx.borrow()
    }

    pub fn vts_event_receiver(&self) -> VtsEventReceiver {
        self.event_sender.subscribe()
    }

    pub fn vts_event_publisher(&self) -> VtsEventSender {
        self.event_sender.clone()
    }

    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    pub fn kline_node_event_receiver(&self) -> &Vec<broadcast::Receiver<E>> {
        &self.kline_node_event_receiver
    }

    pub fn find_kline_price(&self, exchange: &Exchange, symbol: &String) -> Result<&Kline, VtsError> {
        self.kline_price
            .get(&(exchange.clone(), symbol.clone()))
            .context(KlineKeyNotFoundSnafu {
                exchange: exchange.to_string(),
                symbol: symbol,
            })
    }

    pub fn set_kline_price(&mut self, kline_price: HashMap<(Exchange, String), Kline>) {
        self.kline_price = kline_price;
    }

    pub fn handle_kline_update(&mut self, exchange: Exchange, symbol: String, kline: Kline) {
        // if kline_key not in hashmap key, skip
        if !self.kline_price.contains_key(&(exchange.clone(), symbol.clone())) {
            return;
        }
        self.kline_price
            .entry((exchange.clone(), symbol.clone()))
            .and_modify(|e| *e = kline.clone());
        self.update_system(&exchange, &symbol, &kline).unwrap();
    }

    pub fn add_kline_node_event_receiver(&mut self, kline_node_event_receiver: broadcast::Receiver<E>) {
        self.kline_node_event_receiver.push(kline_node_event_receiver);
    }

    // Set kline cache index and update all data
    pub fn update_system(&mut self, exchange: &Exchange, symbol: &String, kline: &Kline) -> Result<(), VtsError> {
        self.check_unfilled_orders(exchange, symbol, kline)?;
        // After price update, update positions
        self.update_current_positions(exchange, symbol, kline)?;

        // Update balance related data in correct order
        // 1. Update realized pnl
        self.update_realized_pnl();
        // 2. Update unrealized pnl
        self.update_unrealized_pnl();
        // 3. Update used margin
        self.update_used_margin();
        // 4. Update frozen margin
        self.update_frozen_margin();
        // 5. Update account balance
        self.update_balance();
        // 6. Update equity
        self.update_equity();
        // 7. Update available balance
        self.update_available_balance();
        // 8. Update margin ratio
        self.update_margin_ratio();

        // Send event
        self.send_event(VtsEvent::UpdateFinished)?;
        Ok(())
    }

    pub fn send_event(&self, event: VtsEvent) -> Result<usize, VtsError> {
        self.event_sender.send(event).context(EventSendFailedSnafu {})
    }

    // Set initial balance
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

    // Reset system
    // Clear all positions and orders
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
