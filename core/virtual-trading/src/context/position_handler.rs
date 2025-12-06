// External crate imports
use snafu::OptionExt;
// Current crate imports
use star_river_core::{
    custom_type::*,
    exchange::Exchange,
    kline::Kline,
    order::{FuturesOrderSide, OrderStatus},
    position::PositionSide,
};
use star_river_core::{order::OrderType, position::PositionState};

// Local module imports
use super::VtsContext;
use crate::{
    error::{MarginNotEnoughSnafu, PositionNotFoundForSymbolSnafu, PositionNotFoundSnafu, VtsError},
    event::VtsEvent,
    types::{VirtualOrder, VirtualPosition, VirtualTransaction},
    utils::Formula,
};

impl<E> VtsContext<E>
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

    // Remove position from current positions list
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

impl<E> VtsContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    pub fn create_position(&mut self, order: &VirtualOrder, current_price: f64) -> Result<(VirtualPosition, VirtualTransaction), VtsError> {
        // Check if margin is sufficient
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
        // tracing::debug!("create position successfully: {:#?}", virtual_position);
        self.current_positions.push(virtual_position.clone());
        let transaction = VirtualTransaction::new(
            order.order_id,
            virtual_position.position_id,
            order.strategy_id,
            order.node_id.clone(),
            order.node_name.clone(),
            order.order_config_id,
            order.exchange.clone(),
            order.symbol.clone(),
            order.order_side.clone().into(),
            order.quantity,
            current_price,
            None,
            self.current_datetime(),
        );
        self.transactions.push(transaction.clone());
        Ok((virtual_position, transaction))
    }

    /// Execute an open order, return position id
    /// Generate position and transaction details
    pub fn execute_order(&mut self, order: &VirtualOrder, current_price: f64) -> Result<PositionId, VtsError> {
        // tracing::info!("execute open order: {:#?}, execute price: {:?}", order, current_price);

        let execute_datetime = self.current_datetime();

        // Check if same exchange and symbol position already exists
        let existing_position_id = self
            .current_positions
            .iter()
            .find(|p| p.exchange == order.exchange && p.symbol == order.symbol)
            .map(|p| p.position_id);

        // Get or create position
        if let Some(position_id) = existing_position_id {
            tracing::debug!("existing position: {:#?}", position_id);
            let available_balance = self.available_balance;
            let (position, transaction) = {
                let position = self.find_position_mut(position_id)?;
                let (position, transaction) = position.update_with_new_order(order, current_price, available_balance, execute_datetime)?;
                (position, transaction)
            };
            self.transactions.push(transaction.clone());
            self.send_event(VtsEvent::TransactionCreated(transaction))?;
            self.send_event(VtsEvent::PositionUpdated(position.clone()))?;

            let filled_order = self.update_order_status(order.order_id, OrderStatus::Filled)?;
            self.send_event(VtsEvent::FuturesOrderFilled(filled_order))?;

            if position.position_state == PositionState::Closed {
                self.send_event(VtsEvent::PositionClosed(position.clone()))?;
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

            return Ok(position_id);
        } else {
            tracing::debug!("no existing position, create new position for order: {:#?}", order.order_id);
            let (position, transaction) = self.create_position(order, current_price)?;
            self.send_event(VtsEvent::TransactionCreated(transaction))?;

            self.update_order_position_id(order.order_id, position.position_id)?;
            self.send_event(VtsEvent::PositionCreated(position.clone()))?;
            if let Some(tp_order) = self.create_tp_order(order, &position) {
                self.unfilled_orders.push(tp_order.clone());
                self.send_event(VtsEvent::TakeProfitOrderCreated(tp_order))?;
            }

            // Create stop loss order
            if let Some(sl_order) = self.create_sl_order(order, &position) {
                self.unfilled_orders.push(sl_order.clone());
                self.send_event(VtsEvent::StopLossOrderCreated(sl_order))?;
            }

            let filled_order = self.update_order_status(order.order_id, OrderStatus::Filled)?;
            self.send_event(VtsEvent::FuturesOrderFilled(filled_order))?;
            return Ok(position.position_id);
        };
    }

    // Update positions
    pub fn update_current_positions(&mut self, exchange: &Exchange, symbol: &String, kline: &Kline) -> Result<(), VtsError> {
        let position_ids: Vec<PositionId> = self
            .current_positions
            .iter()
            .filter(|p| &p.exchange == exchange && &p.symbol == symbol)
            .map(|p| p.position_id)
            .collect();

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

            // Calculate new margin information
            let margin = Formula::calculate_margin(leverage, current_price, quantity);
            let margin_ratio = Formula::calculate_margin_ratio(available_balance, leverage, current_price, quantity);
            let force_price = Formula::calculate_force_price(&position.position_side, leverage, current_price, quantity);

            // Update position
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
        let (position, virtual_transaction) = {
            let available_balance = self.available_balance;
            let position = self.find_position_mut(position_id)?;
            position.update_with_tp_order(tp_order, available_balance, execute_datetime)
        };
        tracing::debug!("update position with tp order: {:#?}", position);

        // Send position updated event
        self.send_event(VtsEvent::PositionUpdated(position.clone()))?;

        // Send position closed event if fully closed
        if position.position_state == PositionState::Closed {
            self.send_event(VtsEvent::PositionClosed(position.clone()))?;
        }
        self.transactions.push(virtual_transaction.clone());
        self.send_event(VtsEvent::TransactionCreated(virtual_transaction))?;

        // Update tp order status to filled
        let updated_tp_order = self.update_order_status(tp_order.order_id, OrderStatus::Filled)?;
        self.send_event(VtsEvent::TakeProfitOrderFilled(updated_tp_order))?;

        // If fully closed, cancel all unfilled sl orders and tp orders, then move position to history
        if position.position_state == PositionState::Closed {
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
        let (position, virtual_transaction) = {
            let available_balance = self.available_balance;
            let position = self.find_position_mut(position_id)?;
            position.update_with_sl_order(sl_order, available_balance, execute_datetime)
        };

        // Send position updated event
        self.send_event(VtsEvent::PositionUpdated(position.clone()))?;

        // Send position closed event if fully closed
        if position.position_state == PositionState::Closed {
            self.send_event(VtsEvent::PositionClosed(position.clone()))?;
        }

        // send transaction
        self.transactions.push(virtual_transaction.clone());
        self.send_event(VtsEvent::TransactionCreated(virtual_transaction))?;

        // Update sl order status to filled
        let updated_sl_order = self.update_order_status(sl_order.order_id, OrderStatus::Filled)?;
        self.send_event(VtsEvent::StopLossOrderFilled(updated_sl_order))?;

        // If fully closed, cancel tp orders and move position to history
        if position.position_state == PositionState::Closed {
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

    pub fn close_position(
        &mut self,
        node_id: &NodeId,
        node_name: &NodeName,
        config_id: i32,
        symbol: &String,
        exchange: &Exchange,
    ) -> Result<PositionId, VtsError> {
        let (position_id, strategy_id, order_side, quantity) = {
            let position = self.find_position_for(symbol, exchange)?;
            let order_side = match position.position_side {
                PositionSide::Long => FuturesOrderSide::Short,
                PositionSide::Short => FuturesOrderSide::Long,
            };
            let strategy_id = position.strategy_id;
            let quantity = position.quantity;
            (position.position_id, strategy_id, order_side, quantity)
        };

        let current_price = self.find_kline_price(&exchange, &symbol)?.close;
        // Close position by creating a market order
        let market_order = VirtualOrder::create_order(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            config_id,
            exchange.clone(),
            symbol.clone(),
            order_side,
            OrderType::Market,
            quantity,
            current_price,
            None,
            None,
            None,
            None,
            None,
            self.current_datetime(),
        );
        self.unfilled_orders.push(market_order.clone());
        self.send_event(VtsEvent::FuturesOrderCreated(market_order.clone()))?;
        self.execute_order(&market_order, current_price)?;
        // position
        Ok(position_id)
    }

    pub fn close_all_positions(&mut self, node_id: &NodeId, node_name: &NodeName, config_id: i32) -> Result<Vec<PositionId>, VtsError> {
        let all_position_ids = self.current_positions.iter().map(|p| p.position_id).collect::<Vec<PositionId>>();

        for position_id in all_position_ids.iter() {
            let (strategy_id, order_side, quantity, exchange, symbol): (StrategyId, FuturesOrderSide, f64, Exchange, String) = {
                let position = self.find_position_mut(*position_id)?;
                let order_side = match position.position_side {
                    PositionSide::Long => FuturesOrderSide::Short,
                    PositionSide::Short => FuturesOrderSide::Long,
                };
                let strategy_id = position.strategy_id;
                let quantity = position.quantity;
                let exchange = position.exchange.clone();
                let symbol = position.symbol.clone();
                (strategy_id, order_side, quantity, exchange, symbol)
            };

            let current_price = self.find_kline_price(&exchange, &symbol)?.close;

            // Close position by creating a market order
            let market_order = VirtualOrder::create_order(
                strategy_id,
                node_id.clone(),
                node_name.clone(),
                config_id,
                exchange.clone(),
                symbol.clone(),
                order_side,
                OrderType::Market,
                quantity,
                current_price,
                None,
                None,
                None,
                None,
                None,
                self.current_datetime(),
            );
            tracing::debug!("close position, market order created: {:#?}", market_order);
            self.unfilled_orders.push(market_order.clone());
            self.send_event(VtsEvent::FuturesOrderCreated(market_order.clone()))?;
            self.execute_order(&market_order, current_price)?;
        }

        Ok(all_position_ids)
    }
}
