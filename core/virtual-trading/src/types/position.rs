use std::sync::atomic::Ordering;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use star_river_core::{
    custom_type::*,
    exchange::Exchange,
    order::FuturesOrderSide,
    position::{Position, PositionSide, PositionState},
};
use utoipa::ToSchema;

use super::id_generator::POSITION_ID_COUNTER;
use crate::{
    error::{OnlyOneDirectionSupportedSnafu, VtsError},
    types::{VirtualOrder, VirtualTransaction},
    utils::Formula,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VirtualPosition {
    pub position_id: PositionId,
    pub strategy_id: StrategyId,
    pub exchange: Exchange,
    pub symbol: String,
    pub position_side: PositionSide,
    pub position_state: PositionState, // 持仓状态
    pub quantity: f64,
    pub open_price: Price,
    pub current_price: Price,
    pub unrealized_profit: Pnl,    // 未实现盈亏
    pub leverage: Leverage,        // 杠杆倍数
    pub force_price: f64,          // 强平价格
    pub margin: Margin,            // 仓位占用的保证金
    pub margin_ratio: MarginRatio, // 保证金率
    pub roi: f64,                  // 收益率
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl VirtualPosition {
    pub fn new(
        position_side: PositionSide,
        strategy_id: StrategyId,
        exchange: Exchange,
        symbol: String,
        quantity: f64,
        current_price: Price,
        force_price: Price,
        margin: Margin,
        margin_ratio: MarginRatio,
        leverage: Leverage,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self {
            position_id: POSITION_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            strategy_id,
            exchange,
            symbol,
            position_side,
            position_state: PositionState::Open,
            quantity,
            open_price: current_price,
            current_price,
            unrealized_profit: 0.0,
            force_price,
            margin,
            margin_ratio,
            leverage,
            roi: 0.0,
            create_time: datetime,
            update_time: datetime,
        }
    }

    pub fn update(&mut self, current_price: Price, datetime: DateTime<Utc>, margin: Margin, margin_ratio: MarginRatio, force_price: Price) {
        self.current_price = current_price;
        self.update_time = datetime;
        self.unrealized_profit = match self.position_side {
            PositionSide::Long => self.quantity * (current_price - self.open_price),
            PositionSide::Short => self.quantity * (self.open_price - current_price),
        };
        self.roi = self.unrealized_profit / (self.open_price * self.quantity);
        self.margin = margin;
        self.margin_ratio = margin_ratio;
        self.force_price = force_price;
    }

    pub fn update_with_new_order(
        &mut self,
        order: &VirtualOrder,
        current_price: Price,
        available_balance: Balance,
        datetime: DateTime<Utc>,
    ) -> Result<(VirtualPosition, VirtualTransaction), VtsError> {
        match (&order.order_side, &self.position_side) {
            // same direction, add position quantity
            (FuturesOrderSide::Long, PositionSide::Long) | (FuturesOrderSide::Short, PositionSide::Short) => {
                let transaction = self.add_position(order, current_price, available_balance, datetime);
                return Ok((self.clone(), transaction));
            }
            // opposite direction, subtract position quantity
            (FuturesOrderSide::Long, PositionSide::Short) | (FuturesOrderSide::Short, PositionSide::Long) => {
                tracing::debug!(
                    "update_with_new_order, opposite direction, subtract position quantity: {:#?}",
                    order
                );
                // Use close_partially which handles both partial and full close
                let transaction = self.close_partially(order, current_price, order.quantity, available_balance, datetime);
                return Ok((self.clone(), transaction));
            }
        }
    }

    // return true if all closed, false if partial closed
    pub fn update_with_tp_order(
        &mut self,
        tp_order: &VirtualOrder,
        balance: Balance,
        datetime: DateTime<Utc>,
    ) -> (VirtualPosition, VirtualTransaction) {
        // Check quantity and route to appropriate close method
        if (tp_order.quantity - self.quantity).abs() < f64::EPSILON {
            // Full close
            let transaction = self.close_all(tp_order, tp_order.open_price, datetime);
            return (self.clone(), transaction);
        } else if tp_order.quantity < self.quantity {
            // Partial close
            let transaction = self.close_partially(tp_order, tp_order.open_price, tp_order.quantity, balance, datetime);
            return (self.clone(), transaction);
        } else {
            // Over quantity close - close all
            let transaction = self.close_all(tp_order, tp_order.open_price, datetime);
            return (self.clone(), transaction);
        }
    }

    /// Update position with stop loss order execution
    pub fn update_with_sl_order(
        &mut self,
        sl_order: &VirtualOrder,
        balance: Balance,
        datetime: DateTime<Utc>,
    ) -> (VirtualPosition, VirtualTransaction) {
        // Check quantity and route to appropriate close method
        if (sl_order.quantity - self.quantity).abs() < f64::EPSILON {
            // Full close
            let transaction = self.close_all(sl_order, sl_order.open_price, datetime);
            return (self.clone(), transaction);
        } else if sl_order.quantity < self.quantity {
            // Partial close
            let transaction = self.close_partially(sl_order, sl_order.open_price, sl_order.quantity, balance, datetime);
            return (self.clone(), transaction);
        } else {
            // Over quantity close - close all
            let transaction = self.close_all(sl_order, sl_order.open_price, datetime);
            return (self.clone(), transaction);
        }
    }

    /// Close all position with the given order
    /// Returns the realized profit and transaction record
    fn close_all(&mut self, order: &VirtualOrder, close_price: Price, datetime: DateTime<Utc>) -> VirtualTransaction {
        self.current_price = close_price;
        self.update_time = datetime;
        self.position_state = PositionState::Closed;

        // Calculate realized profit for the entire position
        let realized_profit = match self.position_side {
            PositionSide::Long => self.quantity * (close_price - self.open_price),
            PositionSide::Short => self.quantity * (self.open_price - close_price),
        };

        // Update unrealized profit (same as realized when fully closed)
        self.unrealized_profit = realized_profit;
        self.roi = self.unrealized_profit / (self.open_price * self.quantity);

        // Clear position metrics
        self.force_price = 0.0;
        self.margin = 0.0;
        self.margin_ratio = 0.0;
        self.quantity = 0.0;

        let transaction = VirtualTransaction::new(
            order.order_id,
            self.position_id,
            order.strategy_id,
            order.node_id.clone(),
            order.node_name.clone(),
            order.order_config_id,
            order.exchange.clone(),
            order.symbol.clone(),
            order.order_side.clone().into(),
            order.quantity,
            close_price,
            Some(realized_profit),
            datetime,
        );

        transaction
    }

    /// Add position with the given order (same direction)
    /// Returns the transaction record
    fn add_position(
        &mut self,
        order: &VirtualOrder,
        add_price: Price,
        available_balance: Balance,
        datetime: DateTime<Utc>,
    ) -> VirtualTransaction {
        let new_total_quantity = self.quantity + order.quantity;
        let new_open_price = (self.open_price * self.quantity + add_price * order.quantity) / new_total_quantity;

        // Update position quantity and open price
        self.quantity = new_total_quantity;
        self.open_price = new_open_price;
        self.current_price = add_price;
        self.update_time = datetime;

        // Recalculate unrealized profit
        self.unrealized_profit = match self.position_side {
            PositionSide::Long => self.quantity * (add_price - self.open_price),
            PositionSide::Short => self.quantity * (self.open_price - add_price),
        };
        self.roi = self.unrealized_profit / (self.open_price * self.quantity);

        // Recalculate margin metrics
        self.margin = Formula::calculate_margin(self.leverage, add_price, self.quantity);
        self.margin_ratio = Formula::calculate_margin_ratio(available_balance, self.leverage, add_price, self.quantity);
        self.force_price = Formula::calculate_force_price(&self.position_side, self.leverage, add_price, self.quantity);

        let transaction = VirtualTransaction::new(
            order.order_id,
            self.position_id,
            order.strategy_id,
            order.node_id.clone(),
            order.node_name.clone(),
            order.order_config_id,
            order.exchange.clone(),
            order.symbol.clone(),
            order.order_side.clone().into(),
            order.quantity,
            add_price,
            None, // No realized profit for adding position
            datetime,
        );

        transaction
    }

    /// Close part of the position with the given order
    /// Returns the transaction record
    /// If close_quantity >= position quantity, will close all position
    fn close_partially(
        &mut self,
        order: &VirtualOrder,
        close_price: Price,
        close_quantity: f64,
        balance: Balance,
        datetime: DateTime<Utc>,
    ) -> VirtualTransaction {
        // Check if we should close all instead
        if close_quantity >= self.quantity || (close_quantity - self.quantity).abs() < f64::EPSILON {
            tracing::debug!(
                "quantity is same, close all position: order_quantity: {:?}, position_quantity: {:?}",
                order.quantity,
                self.quantity
            );
            return self.close_all(order, close_price, datetime);
        }

        self.current_price = close_price;
        self.update_time = datetime;

        // Calculate realized profit for the closed portion
        let realized_profit = match self.position_side {
            PositionSide::Long => close_quantity * (close_price - self.open_price),
            PositionSide::Short => close_quantity * (self.open_price - close_price),
        };

        // Reduce position quantity
        self.quantity -= close_quantity;

        // Recalculate unrealized profit for remaining position
        self.unrealized_profit = match self.position_side {
            PositionSide::Long => self.quantity * (close_price - self.open_price),
            PositionSide::Short => self.quantity * (self.open_price - close_price),
        };
        self.roi = self.unrealized_profit / (self.open_price * self.quantity);

        // Recalculate margin metrics for remaining position
        self.margin = Formula::calculate_margin(self.leverage, close_price, self.quantity);
        self.margin_ratio = Formula::calculate_margin_ratio(balance, self.leverage, close_price, self.quantity);
        self.force_price = Formula::calculate_force_price(&self.position_side, self.leverage, close_price, self.quantity);

        let transaction = VirtualTransaction::new(
            order.order_id,
            self.position_id,
            order.strategy_id,
            order.node_id.clone(),
            order.node_name.clone(),
            order.order_config_id,
            order.exchange.clone(),
            order.symbol.clone(),
            order.order_side.clone().into(),
            close_quantity,
            close_price,
            Some(realized_profit),
            datetime,
        );

        transaction
    }
}
