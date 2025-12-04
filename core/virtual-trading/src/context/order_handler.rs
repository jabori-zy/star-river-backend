use snafu::OptionExt;
use star_river_core::kline::Kline;
// Current crate imports
use star_river_core::{
    custom_type::*,
    exchange::Exchange,
    order::{FuturesOrderSide, OrderStatus, OrderType, TpslType},
    position::PositionSide,
};

// Local module imports
use super::VtsContext;
use crate::{
    error::{OrderNotFoundSnafu, UnsupportedOrderTypeSnafu, VtsError},
    event::VtsEvent,
    types::{VirtualOrder, VirtualPosition},
};

impl<E> VtsContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    pub fn find_unfilled_order(&self, order_id: &OrderId) -> Result<&VirtualOrder, VtsError> {
        self.unfilled_orders
            .iter()
            .find(|order| &order.order_id == order_id)
            .context(OrderNotFoundSnafu { order_id: *order_id })
    }

    pub fn find_unfilled_order_mut(&mut self, order_id: &OrderId) -> Result<&mut VirtualOrder, VtsError> {
        self.unfilled_orders
            .iter_mut()
            .find(|order| &order.order_id == order_id)
            .context(OrderNotFoundSnafu { order_id: *order_id })
    }

    pub fn find_tp_order_ids(&self, symbol: &String, exchange: &Exchange) -> Vec<OrderId> {
        self.unfilled_orders
            .iter()
            .filter(|order| order.exchange == *exchange && order.symbol == *symbol && order.order_type == OrderType::TakeProfitMarket)
            .map(|order| order.order_id)
            .collect()
    }

    pub fn find_sl_order_ids(&self, symbol: &String, exchange: &Exchange) -> Vec<OrderId> {
        self.unfilled_orders
            .iter()
            .filter(|order| order.exchange == *exchange && order.symbol == *symbol && order.order_type == OrderType::StopMarket)
            .map(|order| order.order_id)
            .collect()
    }

    pub fn find_unfilled_order_ids_for(&self, exchange: &Exchange, symbol: &String) -> Vec<OrderId> {
        self.unfilled_orders
            .iter()
            .filter(|order| {
                order.exchange == *exchange
                    && order.symbol == *symbol
                    && (order.order_status == OrderStatus::Created || order.order_status == OrderStatus::Placed)
            })
            .map(|order| order.order_id)
            .collect()
    }

    pub fn unfilled_order_count(&self) -> usize {
        self.unfilled_orders.len()
    }

    pub fn history_order_count(&self) -> usize {
        self.history_orders.len()
    }

    pub fn unfilled_order_count_of_symbol(&self, symbol: &String, exchange: &Exchange) -> usize {
        self.unfilled_orders
            .iter()
            .filter(|order| &order.symbol == symbol && &order.exchange == exchange)
            .count()
    }

    pub fn history_order_count_of_symbol(&self, symbol: &String, exchange: &Exchange) -> usize {
        self.history_orders
            .iter()
            .filter(|order| &order.symbol == symbol && &order.exchange == exchange)
            .count()
    }

    pub fn update_order_status(&mut self, order_id: OrderId, order_status: OrderStatus) -> Result<VirtualOrder, VtsError> {
        let current_datetime = self.current_datetime();

        // Find the order index in unfilled_orders
        let order_idx = self
            .unfilled_orders
            .iter()
            .position(|order| order.order_id == order_id)
            .context(OrderNotFoundSnafu { order_id })?;

        // Update order status and time
        let order = &mut self.unfilled_orders[order_idx];
        order.order_status = order_status.clone();
        order.update_time = current_datetime;

        // If order is filled, move it to history
        if order_status == OrderStatus::Filled
            || order_status == OrderStatus::Canceled
            || order_status == OrderStatus::Expired
            || order_status == OrderStatus::Rejected
        {
            let order = self.unfilled_orders.remove(order_idx);
            self.history_orders.push(order.clone());
            Ok(order)
        } else {
            Ok(order.clone())
        }
    }

    pub fn update_order_position_id(&mut self, order_id: OrderId, position_id: PositionId) -> Result<(), VtsError> {
        let current_datetime = self.current_datetime();
        let order = self
            .unfilled_orders
            .iter_mut()
            .find(|order| order.order_id == order_id)
            .context(OrderNotFoundSnafu { order_id })?;

        order.position_id = Some(position_id);
        order.update_time = current_datetime;
        Ok(())
    }
}

impl<E> VtsContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    // 创建订单
    pub fn create_order(
        &mut self,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        order_config_id: i32,
        symbol: String,
        exchange: Exchange,
        price: f64,
        order_side: FuturesOrderSide,
        order_type: OrderType,
        quantity: f64,
        tp: Option<f64>,
        sl: Option<f64>,
        tp_type: Option<TpslType>,
        sl_type: Option<TpslType>,
        point: Option<f64>,
    ) -> Result<(), VtsError> {
        let current_datetime = self.current_datetime();
        let kline = self.find_kline_price(&exchange, &symbol)?;
        let current_price = kline.close;
        // order create closure
        let create_order = |price| -> Result<VirtualOrder, VtsError> {
            let order = VirtualOrder::create_order(
                strategy_id,
                node_id,
                node_name,
                order_config_id,
                exchange,
                symbol,
                order_side.clone(),
                order_type.clone(),
                quantity,
                price,
                tp,
                sl,
                tp_type,
                sl_type,
                point,
                current_datetime,
            );
            let order_create_event = VtsEvent::FuturesOrderCreated(order.clone());
            self.send_event(order_create_event)?;
            // 插入订单
            self.unfilled_orders.push(order.clone());
            Ok(order)
        };
        // 根据订单类型判断是否需要立即成交
        match &order_type {
            // 市价单
            OrderType::Market => {
                let market_order = create_order(current_price)?;

                // 创建完成后，直接成交订单
                self.execute_order(&market_order, current_price)?;
            }
            // 限价单
            OrderType::Limit => {
                // if limit price >= close price, execute order immidetely
                match order_side {
                    FuturesOrderSide::Long => {
                        // if configered price is higher than current price, execute order immidetely
                        if price >= current_price {
                            let limit_order = create_order(current_price)?;
                            self.execute_order(&limit_order, current_price)?;
                            let directly_execute_event = VtsEvent::LimitOrderExecutedDirectly {
                                limit_price: price,
                                order: limit_order,
                            };
                            self.send_event(directly_execute_event)?;
                        } else {
                            create_order(price)?;
                        }
                    }
                    FuturesOrderSide::Short => {
                        if price <= current_price {
                            let limit_order = create_order(current_price)?;
                            self.execute_order(&limit_order, current_price)?;
                            let directly_execute_event = VtsEvent::LimitOrderExecutedDirectly {
                                limit_price: price,
                                order: limit_order,
                            };
                            self.send_event(directly_execute_event)?;
                        } else {
                            create_order(price)?;
                        }
                    }
                }
            }
            _ => {
                return Err(UnsupportedOrderTypeSnafu {
                    order_type: order_type.to_string(),
                }
                .build());
            }
        }
        Ok(())
    }

    // 检查未成交订单（包括挂单，止盈止损订单），如果未成交，则立即成交
    pub fn check_unfilled_orders(&mut self, exchange: &Exchange, symbol: &String, kline: &Kline) -> Result<(), VtsError> {
        // 获取未成交订单
        let unfilled_order_ids = self.find_unfilled_order_ids_for(exchange, symbol);

        if unfilled_order_ids.is_empty() {
            return Ok(());
        }

        let high_price = kline.high;
        let low_price = kline.low;
        for order_id in unfilled_order_ids {
            let order = self.find_unfilled_order(&order_id).map(|order| order.clone());
            if let Ok(order) = order {
                match order.order_type {
                    OrderType::Limit => {
                        match order.order_side {
                            FuturesOrderSide::Long => {
                                // 限价开多：最低价格 <= 订单价格时执行
                                if low_price <= order.open_price {
                                    // 限价单的成交价格应该是挂单的价格
                                    self.execute_order(&order, order.open_price)?;
                                }
                            }
                            FuturesOrderSide::Short => {
                                // 限价开空：最高价格 >= 订单价格时执行
                                if high_price >= order.open_price {
                                    // 限价单的成交价格应该是挂单的价格
                                    self.execute_order(&order, order.open_price)?;
                                }
                            }
                        }
                    }
                    OrderType::StopMarket => {
                        match order.order_side {
                            FuturesOrderSide::Long => {
                                // 平多止损：最低价格 <= 止损价格时执行
                                if high_price >= order.open_price {
                                    self.execute_sl_order(&order)?;
                                }
                            }
                            FuturesOrderSide::Short => {
                                // 平空止损：最高价格 >= 止损价格时执行
                                if low_price <= order.open_price {
                                    self.execute_sl_order(&order)?;
                                }
                            }
                        }
                    }
                    OrderType::TakeProfitMarket => {
                        match order.order_side {
                            FuturesOrderSide::Long => {
                                // 平多止盈：最高价格 >= 止盈价格时执行
                                if low_price <= order.open_price {
                                    self.execute_tp_order(&order)?;
                                }
                            }
                            FuturesOrderSide::Short => {
                                // 平空止盈：最低价格 <= 止盈价格时执行
                                if high_price >= order.open_price {
                                    self.execute_tp_order(&order)?;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                continue;
            }
        }
        Ok(())
    }

    pub fn create_tp_order(&mut self, order: &VirtualOrder, position: &VirtualPosition) -> Option<VirtualOrder> {
        // if order has tp, create take profit order
        if let Some(tp) = order.tp {
            let tp_order_side = match position.position_side {
                PositionSide::Long => FuturesOrderSide::Short,
                PositionSide::Short => FuturesOrderSide::Long,
            };

            let tp_order = VirtualOrder::create_take_profit_order(
                Some(position.position_id),
                position.strategy_id,
                order.node_id.clone(),
                order.node_name.clone(),
                order.order_config_id,
                order.exchange.clone(),
                order.symbol.clone(),
                tp_order_side,
                position.quantity,
                tp,
                self.current_datetime(),
            );
            return Some(tp_order);
        }
        None
    }

    pub fn create_sl_order(&mut self, order: &VirtualOrder, position: &VirtualPosition) -> Option<VirtualOrder> {
        // 生成止盈止损订单
        if let Some(sl) = order.sl {
            let sl_order_side = match position.position_side {
                PositionSide::Long => FuturesOrderSide::Short,
                PositionSide::Short => FuturesOrderSide::Long,
            };

            let sl_order = VirtualOrder::create_stop_loss_order(
                Some(position.position_id),
                order.strategy_id,
                order.node_id.clone(),
                order.node_name.clone(),
                order.order_config_id,
                order.exchange.clone(),
                order.symbol.clone(),
                sl_order_side,
                order.quantity, // 全部止盈
                sl,             // 止损订单的开仓价格，就是主订单中的止损价格
                self.current_datetime(),
            );
            return Some(sl_order);
        }
        None
    }
}
