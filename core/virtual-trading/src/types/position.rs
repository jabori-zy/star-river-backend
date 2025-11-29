use std::sync::atomic::Ordering;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use star_river_core::{
    custom_type::*,
    exchange::Exchange,
    order::FuturesOrderSide,
    position::{PositionSide, PositionState},
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

    pub fn update_with_new_order(&mut self, order: &VirtualOrder, current_price: Price, datetime: DateTime<Utc>) -> Result<(), VtsError> {
        // Validate that the order can add to this position
        // Only open orders with matching side can add to position
        let can_add = match (&order.order_side, &self.position_side) {
            (FuturesOrderSide::Long, PositionSide::Long) => true,
            (FuturesOrderSide::Short, PositionSide::Short) => true,
            _ => false,
        };

        // direction mismatch
        if !can_add {
            return Err(OnlyOneDirectionSupportedSnafu {
                order_side: order.order_side.to_string(),
                position_side: self.position_side.to_string(),
            }
            .build());
        }
        // Calculate new total quantity
        let new_total_quantity = self.quantity + order.quantity;

        // Calculate weighted average open price
        // Formula: (old_quantity * old_price + new_quantity * new_price) / total_quantity
        let new_open_price = (self.open_price * self.quantity + current_price * order.quantity) / new_total_quantity;

        // Update position fields
        self.quantity = new_total_quantity;
        self.open_price = new_open_price;
        self.current_price = current_price;
        self.update_time = datetime;

        // Recalculate unrealized profit with new quantity and average price
        self.unrealized_profit = match self.position_side {
            PositionSide::Long => self.quantity * (current_price - self.open_price),
            PositionSide::Short => self.quantity * (self.open_price - current_price),
        };
        self.roi = self.unrealized_profit / (self.open_price * self.quantity);

        // Note: margin, margin_ratio, and force_price need to be updated separately
        // as they require leverage and available_balance which are not available here
        Ok(())
    }

    // return true if all closed, false if partial closed
    pub fn update_with_tp_order(
        &mut self,
        tp_order: &VirtualOrder,
        balance: Balance,
        datetime: DateTime<Utc>,
    ) -> (bool, VirtualTransaction) {
        self.current_price = tp_order.open_price;
        self.update_time = datetime;

        // 检查止盈数量和仓位数量
        if (tp_order.quantity - self.quantity).abs() < f64::EPSILON {
            // 全部平仓
            self.position_state = PositionState::Closed;
            self.unrealized_profit = match self.position_side {
                PositionSide::Long => self.quantity * (self.current_price - self.open_price),
                PositionSide::Short => self.quantity * (self.open_price - self.current_price),
            };
            self.roi = self.unrealized_profit / (self.open_price * self.quantity);
            self.force_price = 0.0;
            self.margin = 0.0;
            self.margin_ratio = 0.0;
            self.quantity = 0.0;

            let new_transaction = VirtualTransaction::new(
                tp_order.order_id,
                self.position_id,
                tp_order.strategy_id,
                tp_order.node_id.clone(),
                tp_order.node_name.clone(),
                tp_order.order_config_id,
                tp_order.exchange.clone(),
                tp_order.symbol.clone(),
                tp_order.order_side.clone().into(),
                tp_order.quantity,
                tp_order.open_price,
                Some(self.unrealized_profit),
                datetime,
            );
            return (true, new_transaction);
        } else if tp_order.quantity < self.quantity {
            // 部分平仓
            self.quantity -= tp_order.quantity;
            self.unrealized_profit = match self.position_side {
                PositionSide::Long => self.quantity * (self.current_price - self.open_price),
                PositionSide::Short => self.quantity * (self.open_price - self.current_price),
            };
            self.roi = self.unrealized_profit / (self.open_price * self.quantity);
            self.margin = Formula::calculate_margin(self.leverage, self.current_price, self.quantity);
            self.margin_ratio = Formula::calculate_margin_ratio(balance, self.leverage, self.current_price, self.quantity);
            self.force_price = Formula::calculate_force_price(&self.position_side, self.leverage, self.current_price, self.quantity);

            let realized_profit = match self.position_side {
                PositionSide::Long => tp_order.quantity * (self.current_price - tp_order.open_price),
                PositionSide::Short => tp_order.quantity * (tp_order.open_price - self.current_price),
            };
            let new_transaction = VirtualTransaction::new(
                tp_order.order_id,
                self.position_id,
                tp_order.strategy_id,
                tp_order.node_id.clone(),
                tp_order.node_name.clone(),
                tp_order.order_config_id,
                tp_order.exchange.clone(),
                tp_order.symbol.clone(),
                tp_order.order_side.clone().into(),
                tp_order.quantity,
                tp_order.open_price,
                Some(realized_profit),
                datetime,
            );
            return (false, new_transaction);
        } else {
            // 止盈订单的数量大于仓位数量，全部平仓并返回错误
            self.position_state = PositionState::Closed;
            self.unrealized_profit = match self.position_side {
                PositionSide::Long => self.quantity * (self.current_price - self.open_price),
                PositionSide::Short => self.quantity * (self.open_price - self.current_price),
            };
            self.roi = self.unrealized_profit / (self.open_price * self.quantity);
            self.force_price = 0.0;
            self.margin = 0.0;
            self.margin_ratio = 0.0;

            self.quantity = 0.0;
            let new_transaction = VirtualTransaction::new(
                tp_order.order_id,
                self.position_id,
                tp_order.strategy_id,
                tp_order.node_id.clone(),
                tp_order.node_name.clone(),
                tp_order.order_config_id,
                tp_order.exchange.clone(),
                tp_order.symbol.clone(),
                tp_order.order_side.clone().into(),
                tp_order.quantity,
                tp_order.open_price,
                Some(self.unrealized_profit),
                datetime,
            );
            return (true, new_transaction);
        }
    }

    /// Update position with stop loss order execution
    pub fn update_with_sl_order(
        &mut self,
        sl_order: &VirtualOrder,
        balance: Balance,
        datetime: DateTime<Utc>,
    ) -> (bool, VirtualTransaction) {
        self.current_price = sl_order.open_price;
        self.update_time = datetime;

        if (sl_order.quantity - self.quantity).abs() < f64::EPSILON {
            // Full close
            self.position_state = PositionState::Closed;
            self.unrealized_profit = match self.position_side {
                PositionSide::Long => self.quantity * (self.current_price - self.open_price),
                PositionSide::Short => self.quantity * (self.open_price - self.current_price),
            };
            self.roi = self.unrealized_profit / (self.open_price * self.quantity);
            self.force_price = 0.0;
            self.margin = 0.0;
            self.margin_ratio = 0.0;
            self.quantity = 0.0;

            let new_transaction = VirtualTransaction::new(
                sl_order.order_id,
                self.position_id,
                sl_order.strategy_id,
                sl_order.node_id.clone(),
                sl_order.node_name.clone(),
                sl_order.order_config_id,
                sl_order.exchange.clone(),
                sl_order.symbol.clone(),
                sl_order.order_side.clone().into(),
                sl_order.quantity,
                sl_order.open_price,
                Some(self.unrealized_profit),
                datetime,
            );
            return (true, new_transaction);
        } else if sl_order.quantity < self.quantity {
            // Partial close
            self.quantity -= sl_order.quantity;
            self.unrealized_profit = match self.position_side {
                PositionSide::Long => self.quantity * (self.current_price - self.open_price),
                PositionSide::Short => self.quantity * (self.open_price - self.current_price),
            };
            self.roi = self.unrealized_profit / (self.open_price * self.quantity);
            self.margin = Formula::calculate_margin(self.leverage, self.current_price, self.quantity);
            self.margin_ratio = Formula::calculate_margin_ratio(balance, self.leverage, self.current_price, self.quantity);
            self.force_price = Formula::calculate_force_price(&self.position_side, self.leverage, self.current_price, self.quantity);

            let realized_profit = match self.position_side {
                PositionSide::Long => sl_order.quantity * (self.current_price - sl_order.open_price),
                PositionSide::Short => sl_order.quantity * (sl_order.open_price - self.current_price),
            };
            let new_transaction = VirtualTransaction::new(
                sl_order.order_id,
                self.position_id,
                sl_order.strategy_id,
                sl_order.node_id.clone(),
                sl_order.node_name.clone(),
                sl_order.order_config_id,
                sl_order.exchange.clone(),
                sl_order.symbol.clone(),
                sl_order.order_side.clone().into(),
                sl_order.quantity,
                sl_order.open_price,
                Some(realized_profit),
                datetime,
            );
            return (false, new_transaction);
        } else {
            // SL order quantity exceeds position quantity, close all and return error
            self.position_state = PositionState::Closed;
            self.unrealized_profit = match self.position_side {
                PositionSide::Long => self.quantity * (self.current_price - self.open_price),
                PositionSide::Short => self.quantity * (self.open_price - self.current_price),
            };
            self.roi = self.unrealized_profit / (self.open_price * self.quantity);
            self.force_price = 0.0;
            self.margin = 0.0;
            self.margin_ratio = 0.0;

            self.quantity = 0.0;
            let new_transaction = VirtualTransaction::new(
                sl_order.order_id,
                self.position_id,
                sl_order.strategy_id,
                sl_order.node_id.clone(),
                sl_order.node_name.clone(),
                sl_order.order_config_id,
                sl_order.exchange.clone(),
                sl_order.symbol.clone(),
                sl_order.order_side.clone().into(),
                sl_order.quantity,
                sl_order.open_price,
                Some(self.unrealized_profit),
                datetime,
            );
            return (true, new_transaction);
        }
    }
}
