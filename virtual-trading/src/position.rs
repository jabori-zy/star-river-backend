use event_center::Channel::Position;
use super::VirtualTradingSystem;
use types::position::virtual_position::VirtualPosition;
use types::order::virtual_order::VirtualOrder;
use types::order::OrderStatus;
use types::custom_type::*;
use types::transaction::virtual_transaction::VirtualTransaction;
use types::virtual_trading_system::event::VirtualTradingSystemEvent;
use types::order::{TpslType, OrderType, FuturesOrderSide};
use types::position::{PositionSide, PositionState};
use chrono::DateTime;


impl VirtualTradingSystem {


    pub fn generate_position_id(&self) -> PositionId {
        self.current_positions.len() as PositionId
    }




    /// 执行订单, 返回持仓id
    /// 生成仓位和交易明细
    pub fn execute_order(&mut self, order_id: OrderId, current_price: f64, execute_timestamp: i64) -> Result<PositionId, String> {
        // tracing::info!("执行订单: {:?}, 成交价格: {:?}", virtual_order, current_price);

        let order = self.get_order(order_id).unwrap().clone();

        // 判断保证金是否充足
        let margin = self.calculate_margin(current_price, order.quantity);
        if margin > self.current_balance {
            return Err(format!("保证金不足，需要{}，当前余额{}", margin, self.current_balance));
        }


        // 计算强平价格
        let force_price = self.calculate_force_price(current_price, order.quantity);

        // 计算保证金率
        let margin_ratio = self.calculate_margin_ratio(current_price, order.quantity);
        tracing::debug!("margin: {}, margin_ratio: {}, force_price: {}", margin, margin_ratio, force_price);


        let position_id = self.generate_position_id();

        // 执行订单，生成模拟仓位
        let virtual_position = VirtualPosition::new(position_id, &order, current_price, force_price, margin, margin_ratio, execute_timestamp);

        // 更新订单的仓位id
        self.update_order_position_id(order_id, position_id, execute_timestamp).unwrap();

        // 发送仓位创建事件
        let position_created_event = VirtualTradingSystemEvent::PositionCreated(virtual_position.clone());
        let _ = self.event_publisher.send(position_created_event);

        // 创建止盈止损订单
        let tp_order = self.create_take_profit_order(&order, position_id, execute_timestamp);
        if let Some(tp_order) = tp_order {
            self.orders.push(tp_order.clone());
            let order_create_event = VirtualTradingSystemEvent::TakeProfitOrderCreated(tp_order.clone());
            let _ = self.event_publisher.send(order_create_event);
        }
        
        let sl_order = self.create_stop_loss_order(&order, position_id, execute_timestamp);
        if let Some(sl_order) = sl_order {
            self.orders.push(sl_order.clone());
            let order_create_event = VirtualTradingSystemEvent::StopLossOrderCreated(sl_order.clone());
            let _ = self.event_publisher.send(order_create_event);
        }

        

        // 生成交易明细
        let transaction_id = self.get_transaction_id();
        let virtual_transaction = VirtualTransaction::new(transaction_id, &order, &virtual_position, execute_timestamp);

        // 发送交易明细创建事件
        let transaction_created_event = VirtualTradingSystemEvent::TransactionCreated(virtual_transaction.clone());
        let _ = self.event_publisher.send(transaction_created_event);

        // 修改订单的状态
        self.update_order_status(order_id, OrderStatus::Filled, execute_timestamp).unwrap();

        // 将交易明细添加到交易明细列表中
        self.transactions.push(virtual_transaction);

        let position_id = virtual_position.position_id;
        // 将仓位添加到当前持仓列表中
        self.current_positions.push(virtual_position);
        
        // 在这里发送订单成交事件
        let filled_order = self.get_order(order_id).unwrap();
        let order_filled_event = VirtualTradingSystemEvent::FuturesOrderFilled(filled_order.clone());
        let _ = self.event_publisher.send(order_filled_event);
        
        Ok(position_id)
    }

    // 更新仓位
    pub fn update_position(&mut self) {

        let mut execute_tp_order = Vec::new();
        let mut execute_sl_order = Vec::new();

        for i in 0..self.current_positions.len() {
            let kline_key = {
                let position = &self.current_positions[i];
                self.get_kline_key(&position.exchange, &position.symbol)
            };
            
            if let Some(kline_key) = kline_key {
                if let Some((current_price, _)) = self.kline_price.get(&kline_key) {
                    let current_price_val = *current_price;
                    
                    // 获取仓位信息进行止盈止损检查
                    let (position_id,tp, sl, position_side) = {
                        let position = &self.current_positions[i];
                        (position.position_id, position.tp, position.sl, position.position_side.clone())
                    };
                    
                    // 先判断是否触发了仓位的止盈
                    if let Some(tp) = tp {

                        let should_execute_tp_order = match position_side {
                            PositionSide::Long => current_price_val >= tp,
                            PositionSide::Short => current_price_val <= tp
                        };

                        if should_execute_tp_order {
                            // 处理止盈订单
                            if let Some(take_profit_order) = self.get_take_profit_order(position_id) {
                                // 将止盈订单克隆后添加到待执行的订单列表中
                                execute_tp_order.push(take_profit_order.clone());
                            }
                        }

                    }
                    
                    // 再判断是否触发了仓位的止损
                    if let Some(sl) = sl {
                        let should_execute_sl_order = match position_side {
                            PositionSide::Long => current_price_val < sl,
                            PositionSide::Short => current_price_val > sl
                        };

                        if should_execute_sl_order {
                            // 处理止损订单
                            if let Some(stop_loss_order) = self.get_stop_loss_order(position_id) {
                                // 将止损订单克隆后添加到待执行的订单列表中
                                execute_sl_order.push(stop_loss_order.clone());
                            }
                        }
                    }
                }
            }
        }

        // 先执行止盈止损订单
        for order in execute_tp_order {
            self.execute_tp_order(&order);
        }

        for order in execute_sl_order {
            self.execute_sl_order(&order);
        }


        // 计算新的保证金信息
        // let margin = self.calculate_margin(current_price_val, quantity);
        // let margin_ratio = self.calculate_margin_ratio(current_price_val, quantity);
        // let force_price = self.calculate_force_price(current_price_val, quantity);
        
        // // 更新仓位
        // let position = &mut self.current_positions[i];
        // position.update_position(current_price_val, self.timestamp, margin, margin_ratio, force_price);
        // let position_updated_event = VirtualTradingSystemEvent::PositionUpdated(position.clone());
        // let _ = self.event_publisher.send(position_updated_event);
    }


    pub fn get_current_positions(&self) -> Vec<VirtualPosition> {
        self.current_positions.clone()
    }

    pub fn get_current_position(&self, position_id: PositionId) -> Option<VirtualPosition> {
        self.current_positions.iter().find(|p| p.position_id == position_id).cloned()
    }

    // 将仓位从当前持仓列表中移除，并添加到历史持仓列表中
    pub fn remove_position(&mut self, position_id: PositionId) {
        self.current_positions.retain(|p| p.position_id != position_id);
    }


    // 执行止盈订单
    fn execute_tp_order(&mut self, order: &VirtualOrder) {

        let execute_price = order.open_price;
        if let Some(position_id) = order.position_id {
            let position = self.get_current_position(position_id);
            self.remove_position(position_id);
            if let Some(mut position) = position {
                // 更新仓位的收益

                let unrealized_profit = match order.order_side {
                    FuturesOrderSide::CloseLong => position.quantity * (execute_price - position.open_price),
                    FuturesOrderSide::CloseShort => position.quantity * (position.open_price - execute_price),
                    _ => 0.0,
                };

                position.unrealized_profit = unrealized_profit;
                position.current_price = execute_price;
                position.update_time = DateTime::from_timestamp_millis(self.timestamp).unwrap();
                position.position_state = PositionState::Closed;
                // 清空其他信息
                position.force_price = 0.0;
                position.margin = 0.0;
                position.margin_ratio = 0.0;
                // 将仓位添加到历史持仓列表中
                self.history_positions.push(position.clone());

                // 生成交易明细
                let transaction_id = self.get_transaction_id();
                let virtual_transaction = VirtualTransaction::new(transaction_id, order, &position, self.timestamp);
                self.transactions.push(virtual_transaction);

                // 修改止盈订单状态
                self.update_order_status(order.order_id, OrderStatus::Filled, self.timestamp).unwrap();
                // 取消同仓位止损订单
                let sl_order = self.get_stop_loss_order(position_id);
                if let Some(sl_order) = sl_order {
                    self.update_order_status(sl_order.order_id, OrderStatus::Canceled, self.timestamp).unwrap();
                }

            }
        }
        

        

    }

    fn execute_sl_order(&mut self, order: &VirtualOrder) {

        let execute_price = order.open_price;
        if let Some(position_id) = order.position_id {
            let position = self.get_current_position(position_id);
            self.remove_position(position_id);
            if let Some(mut position) = position {
                // 更新仓位的收益

                let unrealized_profit = match order.order_side {
                    FuturesOrderSide::CloseLong => position.quantity * (position.open_price - execute_price), // 多仓的亏损状态： 平仓价 < 开仓价
                    FuturesOrderSide::CloseShort => position.quantity * (execute_price - position.open_price), // 空仓的亏损状态： 平仓价 > 开仓价
                    _ => 0.0,
                };

                position.unrealized_profit = unrealized_profit;
                position.current_price = execute_price;
                position.update_time = DateTime::from_timestamp_millis(self.timestamp).unwrap();
                position.position_state = PositionState::Closed;
                // 清空其他信息
                position.force_price = 0.0;
                position.margin = 0.0;
                position.margin_ratio = 0.0;
                // 将仓位添加到历史持仓列表中
                self.history_positions.push(position.clone());

                // 生成交易明细
                let transaction_id = self.get_transaction_id();
                let virtual_transaction = VirtualTransaction::new(transaction_id, order, &position, self.timestamp);
                self.transactions.push(virtual_transaction);

                // 修改止盈订单状态
                self.update_order_status(order.order_id, OrderStatus::Filled, self.timestamp).unwrap();
                // 取消同仓位止损订单
                let sl_order = self.get_stop_loss_order(position_id);
                if let Some(sl_order) = sl_order {
                    self.update_order_status(sl_order.order_id, OrderStatus::Canceled, self.timestamp).unwrap();
                }

            }
        }
        

        

    }

}
