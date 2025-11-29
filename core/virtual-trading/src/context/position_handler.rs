// External crate imports
use key::KlineKey;
use snafu::OptionExt;
// Current crate imports
use star_river_core::{
    custom_type::*,
    exchange::Exchange,
    kline::Kline,
    order::{FuturesOrderSide, OrderStatus},
    position::PositionSide,
};

// Local module imports
use super::VirtualTradingSystemContext;
use crate::{
    error::{MarginNotEnoughSnafu, PositionNotFoundForSymbolSnafu, PositionNotFoundSnafu, VtsError},
    event::VtsEvent,
    types::{VirtualOrder, VirtualPosition, VirtualTransaction},
    utils::Formula,
};

impl<E> VirtualTradingSystemContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    pub fn find_position(&self, position_id: PositionId) -> Result<&VirtualPosition, VtsError> {
        self.current_positions
            .iter()
            .find(|p| p.position_id == position_id)
            .context(PositionNotFoundSnafu { position_id: position_id })
    }

    pub fn find_position_for(&self, symbol: &String, exchange: &Exchange) -> Result<&VirtualPosition, VtsError> {
        self.current_positions
            .iter()
            .find(|p| &p.symbol == symbol && &p.exchange == exchange)
            .context(PositionNotFoundForSymbolSnafu {
                symbol: symbol.clone(),
                exchange: exchange.to_string(),
            })
    }

    pub fn find_position_mut(&mut self, position_id: PositionId) -> Result<&mut VirtualPosition, VtsError> {
        self.current_positions
            .iter_mut()
            .find(|p| p.position_id == position_id)
            .context(PositionNotFoundSnafu { position_id: position_id })
    }

    // 将仓位从当前持仓列表中移除
    pub fn remove_open_position(&mut self, position_id: PositionId) {
        self.current_positions.retain(|p| p.position_id != position_id);
    }

    pub fn current_positions_count(&self) -> usize {
        self.current_positions.len()
    }

    pub fn current_positions_count_of_symbol(&self, symbol: &String, exchange: &Exchange) -> usize {
        self.current_positions
            .iter()
            .filter(|p| &p.symbol == symbol && &p.exchange == exchange)
            .count()
    }

    pub fn history_positions_count(&self) -> usize {
        self.history_positions.len()
    }

    pub fn history_positions_count_of_symbol(&self, symbol: &String, exchange: &Exchange) -> usize {
        self.history_positions
            .iter()
            .filter(|p| &p.symbol == symbol && &p.exchange == exchange)
            .count()
    }
}

impl<E> VirtualTradingSystemContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    pub fn create_position(&mut self, order: &VirtualOrder, current_price: f64) -> Result<VirtualPosition, VtsError> {
        // 判断保证金是否充足
        let margin = Formula::calculate_margin(self.leverage, current_price, order.quantity);
        if margin > self.available_balance {
            return Err(MarginNotEnoughSnafu {
                need_margin: margin,
                available_balance: self.available_balance,
            }
            .build());
        }

        let position_side = match order.order_side {
            FuturesOrderSide::Long => PositionSide::Long,
            FuturesOrderSide::Short => PositionSide::Short,
        };
        let force_price = Formula::calculate_force_price(&position_side, self.leverage, current_price, order.quantity);
        let margin_ratio = Formula::calculate_margin_ratio(self.available_balance, self.leverage, current_price, order.quantity);
        let virtual_position = VirtualPosition::new(
            position_side,
            order.strategy_id,
            order.exchange.clone(),
            order.symbol.clone(),
            order.quantity,
            current_price,
            force_price,
            margin,
            margin_ratio,
            self.leverage,
            self.current_datetime(),
        );
        tracing::debug!("create position successfully: {:#?}", virtual_position);
        self.current_positions.push(virtual_position.clone());
        Ok(virtual_position)
    }

    /// Execute an open order, return position id
    /// Generate position and transaction details
    pub fn execute_order(&mut self, order: &VirtualOrder, current_price: f64) -> Result<PositionId, VtsError> {
        tracing::info!("execute open order: {:#?}, execute price: {:?}", order, current_price);

        let execute_datetime = self.current_datetime();

        // Check if same exchange and symbol position already exists
        let existing_position_id = self
            .current_positions
            .iter()
            .find(|p| p.exchange == order.exchange && p.symbol == order.symbol)
            .map(|p| p.position_id);

        // Get or create position
        let (position, is_new_position) = if let Some(position_id) = existing_position_id {
            let updated_position = {
                let position = self.find_position_mut(position_id)?;
                position.update_with_new_order(order, current_price, execute_datetime)?;
                position.clone()
            };
            (updated_position, false)
        } else {
            let new_position = self.create_position(order, current_price)?;
            (new_position, true)
        };

        let position_id = position.position_id;

        // Send position event based on whether it's new or updated
        if is_new_position {
            self.update_order_position_id(order.order_id, position_id)?;
            self.send_event(VtsEvent::PositionCreated(position.clone()))?;
        } else {
            self.send_event(VtsEvent::PositionUpdated(position.clone()))?;
        }

        // Create take profit order
        if let Some(tp_order) = self.create_tp_order(order, &position) {
            self.unfilled_orders.push(tp_order.clone());
            self.send_event(VtsEvent::TakeProfitOrderCreated(tp_order))?;
        }

        // Create stop loss order
        if let Some(sl_order) = self.create_sl_order(order, &position) {
            self.unfilled_orders.push(sl_order.clone());
            self.send_event(VtsEvent::StopLossOrderCreated(sl_order))?;
        }

        // Generate transaction
        let virtual_transaction = VirtualTransaction::new(
            order.order_id,
            position.position_id,
            order.strategy_id,
            order.node_id.clone(),
            order.node_name.clone(),
            order.order_config_id,
            order.exchange.clone(),
            order.symbol.clone(),
            order.order_side.clone().into(),
            order.quantity,
            order.open_price,
            None,
            execute_datetime,
        );
        self.transactions.push(virtual_transaction.clone());
        self.send_event(VtsEvent::TransactionCreated(virtual_transaction))?;

        // Update order status and send filled event
        let filled_order = self.update_order_status(order.order_id, OrderStatus::Filled)?;
        self.send_event(VtsEvent::FuturesOrderFilled(filled_order))?;

        Ok(position_id)
    }

    // 更新仓位
    pub fn update_current_positions(&mut self, kline_key: &KlineKey, kline: &Kline) -> Result<(), VtsError> {
        let position_ids: Vec<PositionId> = self
            .current_positions
            .iter()
            .filter(|p| p.exchange == kline_key.exchange && p.symbol == kline_key.symbol)
            .map(|p| p.position_id)
            .collect();

        tracing::debug!("update position: position_ids: {:?}", position_ids);

        if position_ids.is_empty() {
            return Ok(());
        }

        for position_id in position_ids {
            let leverage = self.leverage;
            let available_balance = self.available_balance;

            let position = self.find_position_mut(position_id)?;
            let current_price = kline.close;
            let current_datetime = kline.datetime;
            let quantity = position.quantity;

            // 计算新的保证金信息
            let margin = Formula::calculate_margin(leverage, current_price, quantity);
            let margin_ratio = Formula::calculate_margin_ratio(available_balance, leverage, current_price, quantity);
            let force_price = Formula::calculate_force_price(&position.position_side, leverage, current_price, quantity);

            // 更新仓位
            position.update(current_price, current_datetime, margin, margin_ratio, force_price);
            let position_updated_event = VtsEvent::PositionUpdated(position.clone());
            self.send_event(position_updated_event)?;
        }
        Ok(())
    }

    /// Execute take profit order
    pub fn execute_tp_order(&mut self, tp_order: &VirtualOrder) -> Result<(), VtsError> {
        tracing::info!(
            "execute tp order: ID: {:?}, side: {:?}, price: {:?}",
            tp_order.order_id,
            tp_order.order_side,
            tp_order.open_price
        );

        let position_id = match tp_order.position_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let execute_datetime = self.current_datetime();

        // Update position and determine if fully closed
        let (is_all_closed, virtual_transaction) = {
            let available_balance = self.available_balance;
            let position = self.find_position_mut(position_id)?;
            position.update_with_tp_order(tp_order, available_balance, execute_datetime)
        };

        // Get position snapshot
        let position = self.find_position(position_id)?.clone();

        // Send position updated event
        self.send_event(VtsEvent::PositionUpdated(position.clone()))?;

        // Send position closed event if fully closed
        if is_all_closed {
            self.send_event(VtsEvent::PositionClosed(position.clone()))?;
        }
        self.transactions.push(virtual_transaction.clone());
        self.send_event(VtsEvent::TransactionCreated(virtual_transaction))?;

        // Update tp order status to filled
        let updated_tp_order = self.update_order_status(tp_order.order_id, OrderStatus::Filled)?;
        self.send_event(VtsEvent::TakeProfitOrderFilled(updated_tp_order))?;

        // If fully closed, cancel all unfilled sl orders and tp orders, then move position to history
        if is_all_closed {
            let sl_order_ids = self.find_sl_order_ids(&position.symbol, &position.exchange);
            for id in sl_order_ids {
                let updated_sl_order = self.update_order_status(id, OrderStatus::Canceled)?;
                self.send_event(VtsEvent::StopLossOrderCanceled(updated_sl_order))?;
            }
            let tp_order_ids = self.find_tp_order_ids(&position.symbol, &position.exchange);
            for id in tp_order_ids {
                let updated_tp_order = self.update_order_status(id, OrderStatus::Canceled)?;
                self.send_event(VtsEvent::TakeProfitOrderCanceled(updated_tp_order))?;
            }
            self.remove_open_position(position_id);
            self.history_positions.push(position);
        }

        Ok(())
    }

    /// Execute stop loss order
    pub fn execute_sl_order(&mut self, sl_order: &VirtualOrder) -> Result<(), VtsError> {
        tracing::info!(
            "execute sl order: ID: {:?}, side: {:?}, price: {:?}, quantity: {:?}",
            sl_order.order_id,
            sl_order.order_side,
            sl_order.open_price,
            sl_order.quantity
        );

        let position_id = match sl_order.position_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let execute_datetime = self.current_datetime();

        // Update position and determine if fully closed
        let (is_all_closed, virtual_transaction) = {
            let available_balance = self.available_balance;
            let position = self.find_position_mut(position_id)?;
            position.update_with_sl_order(sl_order, available_balance, execute_datetime)
        };

        // Get position snapshot
        let position = self.find_position(position_id)?.clone();

        // Send position updated event
        self.send_event(VtsEvent::PositionUpdated(position.clone()))?;

        // Send position closed event if fully closed
        if is_all_closed {
            self.send_event(VtsEvent::PositionClosed(position.clone()))?;
        }

        // send transaction
        self.transactions.push(virtual_transaction.clone());
        self.send_event(VtsEvent::TransactionCreated(virtual_transaction))?;

        // Update sl order status to filled
        let updated_sl_order = self.update_order_status(sl_order.order_id, OrderStatus::Filled)?;
        self.send_event(VtsEvent::StopLossOrderFilled(updated_sl_order))?;

        // If fully closed, cancel tp orders and move position to history
        if is_all_closed {
            let tp_order_ids = self.find_tp_order_ids(&position.symbol, &position.exchange);
            for id in tp_order_ids {
                let updated_tp_order = self.update_order_status(id, OrderStatus::Canceled)?;
                self.send_event(VtsEvent::TakeProfitOrderCanceled(updated_tp_order))?;
            }
            self.remove_open_position(position_id);
            self.history_positions.push(position);
        }

        Ok(())
    }
}
