// External crate imports
use chrono::{DateTime, Utc};
use key::KlineKey;
use snafu::OptionExt;
// Current crate imports
use star_river_core::{
    custom_type::*,
    kline::Kline,
    order::{FuturesOrderSide, OrderStatus},
    position::{PositionSide, PositionState},
};

// Local module imports
use super::VirtualTradingSystemContext;
use crate::{
    error::{MarginNotEnoughSnafu, PositionNotFoundSnafu, VirtualTradingSystemError},
    event::VtsEvent,
    types::{VirtualOrder, VirtualPosition, VirtualTransaction},
    utils::Formula,
};

impl<E> VirtualTradingSystemContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    pub fn get_positions_by_kline_key(&self, kline_key: &KlineKey) -> Vec<VirtualPosition> {
        self.current_positions
            .iter()
            .filter(|position| position.exchange == kline_key.exchange && position.symbol == kline_key.symbol)
            .cloned()
            .collect::<Vec<VirtualPosition>>()
    }

    pub fn generate_position_id(&self) -> PositionId {
        self.current_positions.len() as PositionId
    }

    pub fn create_position(
        &mut self,
        order: &VirtualOrder,
        current_price: f64,
        execute_datetime: DateTime<Utc>,
    ) -> Result<VirtualPosition, VirtualTradingSystemError> {
        // 判断保证金是否充足
        let margin = Formula::calculate_margin(self.leverage, current_price, order.quantity);
        if margin > self.available_balance {
            return Err(MarginNotEnoughSnafu {
                need_margin: margin,
                available_balance: self.available_balance,
            }
            .build());
        }

        let position_id = self.generate_position_id();
        let position_side = match order.order_side {
            FuturesOrderSide::OpenLong => PositionSide::Long,
            FuturesOrderSide::OpenShort => PositionSide::Short,
            FuturesOrderSide::CloseLong => PositionSide::Long,
            FuturesOrderSide::CloseShort => PositionSide::Short,
        };
        let force_price = Formula::calculate_force_price(&position_side, self.leverage, current_price, order.quantity);
        let margin_ratio = Formula::calculate_margin_ratio(self.available_balance, self.leverage, current_price, order.quantity);
        let virtual_position = VirtualPosition::new(
            position_id,
            position_side,
            &order,
            current_price,
            force_price,
            margin,
            margin_ratio,
            execute_datetime,
        );
        tracing::info!("position created successfully: {:#?}", virtual_position);
        self.current_positions.push(virtual_position.clone());
        Ok(virtual_position)
    }

    /// 执行开仓订单, 返回持仓id
    /// 生成仓位和交易明细
    pub fn execute_order(
        &mut self,
        order: &VirtualOrder,
        current_price: f64,
        execute_datetime: DateTime<Utc>,
    ) -> Result<PositionId, VirtualTradingSystemError> {
        tracing::info!("execute open order: {:#?}, execute price: {:?}", order, current_price);

        let virtual_position = self.create_position(order, current_price, execute_datetime)?;

        // 更新订单的仓位id
        self.update_order_position_id(order.order_id, virtual_position.position_id, execute_datetime)?;

        // 发送仓位创建事件
        let position_created_event = VtsEvent::PositionCreated(virtual_position.clone());
        self.send_event(position_created_event)?;

        // 创建止盈止损订单
        let tp_order = self.create_take_profit_order(&virtual_position, execute_datetime);
        if let Some(tp_order) = tp_order {
            tracing::info!("create take profit order: {:?}", tp_order);
            self.orders.push(tp_order.clone());
            let order_create_event = VtsEvent::TakeProfitOrderCreated(tp_order.clone());
            self.send_event(order_create_event)?;
        }

        let sl_order = self.create_stop_loss_order(&virtual_position, execute_datetime);
        if let Some(sl_order) = sl_order {
            tracing::info!("create stop loss order: {:?}", sl_order);
            self.orders.push(sl_order.clone());
            let order_create_event = VtsEvent::StopLossOrderCreated(sl_order.clone());
            self.send_event(order_create_event)?;
        }

        // 生成交易明细
        let transaction_id = self.get_transaction_id();
        let virtual_transaction = VirtualTransaction::new(transaction_id, &order, &virtual_position, execute_datetime);

        // 发送交易明细创建事件
        let transaction_created_event = VtsEvent::TransactionCreated(virtual_transaction.clone());
        self.send_event(transaction_created_event)?;

        // 修改订单的状态
        self.update_order_status(order.order_id, OrderStatus::Filled, execute_datetime)?;

        // 将交易明细添加到交易明细列表中
        self.transactions.push(virtual_transaction);

        let position_id = virtual_position.position_id;

        // 在这里发送订单成交事件
        let filled_order = self.get_order_by_id(&order.order_id)?;
        let order_filled_event = VtsEvent::FuturesOrderFilled(filled_order.clone());
        self.send_event(order_filled_event)?;

        Ok(position_id)
    }

    // 更新仓位
    pub fn update_position(&mut self, kline_key: &KlineKey, kline: &Kline) -> Result<(), VirtualTradingSystemError> {
        let positions = self.get_positions_by_kline_key(kline_key);
        if positions.is_empty() {
            return Ok(());
        }

        for position in positions.iter() {
            let current_price = kline.close;
            let current_datetime = kline.datetime;
            let quantity = position.quantity;

            // 计算新的保证金信息
            let margin = Formula::calculate_margin(self.leverage, current_price, quantity);
            let margin_ratio = Formula::calculate_margin_ratio(self.available_balance, self.leverage, current_price, quantity);
            let force_price = Formula::calculate_force_price(&position.position_side, self.leverage, current_price, quantity);

            // 更新仓位
            let position = self.get_position_by_id_mut(position.position_id)?;
            position.update(current_price, current_datetime, margin, margin_ratio, force_price);
            let position_updated_event = VtsEvent::PositionUpdated(position.clone());
            self.send_event(position_updated_event)?;
        }
        Ok(())
    }

    // 获取当前持仓
    pub fn get_current_positions(&self) -> &Vec<VirtualPosition> {
        &self.current_positions
    }

    pub fn get_history_positions(&self) -> Vec<VirtualPosition> {
        self.history_positions.clone()
    }

    pub fn get_position_by_id(&self, position_id: PositionId) -> Result<&VirtualPosition, VirtualTradingSystemError> {
        self.current_positions
            .iter()
            .find(|p| p.position_id == position_id)
            .context(PositionNotFoundSnafu { position_id: position_id })
    }

    pub fn get_position_by_id_mut(&mut self, position_id: PositionId) -> Result<&mut VirtualPosition, VirtualTradingSystemError> {
        self.current_positions
            .iter_mut()
            .find(|p| p.position_id == position_id)
            .context(PositionNotFoundSnafu { position_id: position_id })
    }

    // 将仓位从当前持仓列表中移除，并添加到历史持仓列表中
    pub fn remove_position(&mut self, position_id: PositionId) {
        self.current_positions.retain(|p| p.position_id != position_id);
    }

    // 执行止盈订单
    pub fn execute_tp_order(&mut self, tp_order: &VirtualOrder, execute_datetime: DateTime<Utc>) -> Result<(), VirtualTradingSystemError> {
        tracing::info!(
            "执行止盈订单: ID: {:?}, 方向: {:?}, 执行价格: {:?}",
            tp_order.order_id,
            tp_order.order_side,
            tp_order.open_price
        );

        let execute_price = tp_order.open_price;
        if let Some(position_id) = tp_order.position_id {
            // update current position
            {
                let position = self.get_position_by_id_mut(position_id)?;
                // 更新仓位的收益
                let unrealized_profit = match tp_order.order_side {
                    FuturesOrderSide::CloseLong => position.quantity * (execute_price - position.open_price),
                    FuturesOrderSide::CloseShort => position.quantity * (position.open_price - execute_price),
                    _ => 0.0,
                };

                position.unrealized_profit = unrealized_profit;
                position.current_price = execute_price;
                position.update_time = execute_datetime;
                position.position_state = PositionState::Closed;
                // 清空其他信息
                position.force_price = 0.0;
                position.margin = 0.0;
                position.margin_ratio = 0.0;
            }

            let position = self.get_position_by_id(position_id)?;
            // 发送最后一次仓位更新事件
            let position_updated_event = VtsEvent::PositionUpdated(position.clone());
            self.send_event(position_updated_event)?;

            // 发送仓位平仓事件
            let position_closed_event = VtsEvent::PositionClosed(position.clone());
            self.send_event(position_closed_event)?;
            // 将仓位添加到历史持仓列表中

            // 生成交易明细
            let transaction_id = self.get_transaction_id();
            let virtual_transaction = VirtualTransaction::new(transaction_id, tp_order, &position, execute_datetime);
            self.transactions.push(virtual_transaction.clone());
            // 发送交易明细创建事件
            let transaction_created_event = VtsEvent::TransactionCreated(virtual_transaction);
            self.send_event(transaction_created_event)?;

            // 修改止盈订单状态
            let updated_tp_order = self.update_order_status(tp_order.order_id, OrderStatus::Filled, execute_datetime)?;
            // 发送止盈订单成交事件
            let tp_order_filled_event = VtsEvent::TakeProfitOrderFilled(updated_tp_order);
            self.send_event(tp_order_filled_event)?;

            // 取消同仓位止损订单
            let sl_order = self.get_stop_loss_order(position_id);
            if let Some(sl_order) = sl_order {
                let updated_sl_order = self.update_order_status(sl_order.order_id, OrderStatus::Canceled, execute_datetime)?;
                // 发送止损订单取消事件
                let sl_order_canceled_event = VtsEvent::StopLossOrderCanceled(updated_sl_order);
                self.send_event(sl_order_canceled_event)?;
            }

            // remove position from current positions and add to history positions
            {
                let position = self.get_position_by_id(position_id)?.clone();
                self.remove_position(position_id);
                self.history_positions.push(position.clone());
            }
        }
        Ok(())
    }

    pub fn execute_sl_order(&mut self, sl_order: &VirtualOrder, execute_datetime: DateTime<Utc>) -> Result<(), VirtualTradingSystemError> {
        tracing::info!(
            "执行止损订单: ID: {:?}, 方向: {:?}, 执行价格: {:?}，执行数量: {:?}",
            sl_order.order_id,
            sl_order.order_side,
            sl_order.open_price,
            sl_order.quantity
        );

        let execute_price = sl_order.open_price;
        if let Some(position_id) = sl_order.position_id {
            // update current position
            {
                let position = self.get_position_by_id_mut(position_id)?;
                // 更新仓位的收益
                let unrealized_profit = match sl_order.order_side {
                    FuturesOrderSide::CloseLong => position.quantity * (execute_price - position.open_price),
                    FuturesOrderSide::CloseShort => position.quantity * (position.open_price - execute_price),
                    _ => 0.0,
                };

                position.unrealized_profit = unrealized_profit;
                position.current_price = execute_price;
                position.update_time = execute_datetime;
                position.position_state = PositionState::Closed;
                // 清空其他信息
                position.force_price = 0.0;
                position.margin = 0.0;
                position.margin_ratio = 0.0;
            }

            let position = self.get_position_by_id(position_id)?;
            // 发送最后一次仓位更新事件
            let position_updated_event = VtsEvent::PositionUpdated(position.clone());
            self.send_event(position_updated_event)?;

            // 发送仓位平仓事件
            let position_closed_event = VtsEvent::PositionClosed(position.clone());
            self.send_event(position_closed_event)?;

            // 生成交易明细
            let transaction_id = self.get_transaction_id();
            let virtual_transaction = VirtualTransaction::new(transaction_id, sl_order, &position, execute_datetime);
            self.transactions.push(virtual_transaction.clone());
            // 发送交易明细创建事件
            let transaction_created_event = VtsEvent::TransactionCreated(virtual_transaction);
            self.send_event(transaction_created_event)?;

            // 修改止损订单状态
            let updated_sl_order = self.update_order_status(sl_order.order_id, OrderStatus::Filled, execute_datetime)?;
            // 发送止损订单成交事件
            let sl_order_filled_event = VtsEvent::StopLossOrderFilled(updated_sl_order);
            self.send_event(sl_order_filled_event)?;

            // 取消同仓位止盈订单
            let tp_order = self.get_take_profit_order(position_id);
            if let Some(tp_order) = tp_order {
                let updated_tp_order = self.update_order_status(tp_order.order_id, OrderStatus::Canceled, execute_datetime)?;
                // 发送止盈订单取消事件
                let tp_order_canceled_event = VtsEvent::TakeProfitOrderCanceled(updated_tp_order);
                self.send_event(tp_order_canceled_event)?;
            }

            // remove position from current positions and add to history positions
            {
                let position = self.get_position_by_id(position_id)?.clone();
                self.remove_position(position_id);
                self.history_positions.push(position.clone());
            }
        }
        Ok(())
    }
}
