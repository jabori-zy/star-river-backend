// External crate imports
use chrono::{DateTime, Utc};
use key::KlineKey;
use star_river_core::kline::Kline;
// Current crate imports
use star_river_core::{
    custom_type::*,
    exchange::Exchange,
    order::{FuturesOrderSide, OrderStatus, OrderType, TpslType},
    position::PositionSide,
};

// Local module imports
use super::VirtualTradingSystemContext;
use crate::{
    error::{OrderNotFoundSnafu, UnsupportedOrderTypeSnafu, VirtualTradingSystemError},
    event::VtsEvent,
    types::{VirtualOrder, VirtualPosition},
};

impl<E> VirtualTradingSystemContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    // 生成订单ID, 从0开始
    fn generate_order_id(&self) -> i32 {
        self.orders.len() as i32
    }

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
    ) -> Result<(), VirtualTradingSystemError> {
        let order_id = self.generate_order_id();
        let kline_key = self.get_kline_key(&exchange, &symbol)?;
        let current_datetime = self.current_datetime();
        let kline = self.get_kline_price(&kline_key)?;
        let current_price = kline.close;
        // order create closure
        let create_order = |price| -> Result<VirtualOrder, VirtualTradingSystemError> {
            let order = VirtualOrder::create_order(
                strategy_id,
                order_id,
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
            tracing::debug!("order created: {:?}", order);
            let order_create_event = VtsEvent::FuturesOrderCreated(order.clone());
            self.send_event(order_create_event)?;
            // 插入订单
            self.orders.push(order.clone());
            Ok(order)
        };
        // 根据订单类型判断是否需要立即成交
        match &order_type {
            // 市价单
            OrderType::Market => {
                let market_order = create_order(current_price)?;

                // 创建完成后，直接成交订单
                self.execute_order(&market_order, current_price, current_datetime)?;
            }
            // 限价单
            OrderType::Limit => {
                // if limit price >= close price, execute order immidetely
                match order_side {
                    FuturesOrderSide::OpenLong | FuturesOrderSide::CloseShort => {
                        // if configered price is higher than current price, execute order immidetely
                        if price >= current_price {
                            let limit_order = create_order(current_price)?;
                            self.execute_order(&limit_order, current_price, current_datetime)?;
                            let directly_execute_event = VtsEvent::LimitOrderExecutedDirectly { limit_price: price, order: limit_order };
                            self.send_event(directly_execute_event)?;
                        } else {
                            let limit_order = create_order(price)?;
                            self.orders.push(limit_order);
                        }
                    }
                    FuturesOrderSide::OpenShort | FuturesOrderSide::CloseLong => {
                        if price <= current_price {
                            let limit_order = create_order(current_price)?;
                            self.execute_order(&limit_order, current_price, current_datetime)?;
                            let directly_execute_event = VtsEvent::LimitOrderExecutedDirectly { limit_price: price, order: limit_order };
                            self.send_event(directly_execute_event)?;
                        } else {
                            let limit_order = create_order(price)?;
                            self.orders.push(limit_order);
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

    pub fn update_order_status(
        &mut self,
        order_id: OrderId,
        order_status: OrderStatus,
        datetime: DateTime<Utc>,
    ) -> Result<VirtualOrder, VirtualTradingSystemError> {
        let order = self.get_order_by_id_mut(&order_id)?;
        // 更新订单状态
        order.order_status = order_status;
        // 更新订单更新时间
        order.update_time = datetime;
        Ok(order.clone())
    }

    pub fn update_order_position_id(
        &mut self,
        order_id: OrderId,
        position_id: PositionId,
        datetime: DateTime<Utc>,
    ) -> Result<(), VirtualTradingSystemError> {
        if let Some(order) = self.orders.iter_mut().find(|o| o.order_id == order_id) {
            order.position_id = Some(position_id);
            order.update_time = datetime;
            Ok(())
        } else {
            Err(OrderNotFoundSnafu { order_id }.build())
        }
    }

    // 获取未成交订单
    pub fn get_unfilled_orders(&self) -> Vec<VirtualOrder> {
        self.orders
            .iter()
            .filter(|order| order.order_status == OrderStatus::Created || order.order_status == OrderStatus::Placed)
            .cloned()
            .collect::<Vec<VirtualOrder>>()
    }

    // find unfilled order by kline key
    pub fn get_unfilled_orders_by_kline_key(&self, kline_key: &KlineKey) -> Vec<VirtualOrder> {
        self.orders
            .iter()
            .filter(|order| {
                order.exchange == kline_key.exchange
                    && order.symbol == kline_key.symbol
                    && (order.order_status == OrderStatus::Created || order.order_status == OrderStatus::Placed)
            })
            .cloned()
            .collect::<Vec<VirtualOrder>>()
    }

    // 检查未成交订单（包括挂单，止盈止损订单），如果未成交，则立即成交
    pub fn check_unfilled_orders(&mut self, kline_key: &KlineKey, kline: &Kline) -> Result<(), VirtualTradingSystemError> {
        // 获取未成交订单
        let unfilled_orders = self.get_unfilled_orders_by_kline_key(kline_key);

        if unfilled_orders.is_empty() {
            return Ok(());
        }

        let high_price = kline.high;
        let low_price = kline.low;
        let execute_datetime = kline.datetime;
        for order in unfilled_orders {
            match order.order_type {
                OrderType::Limit => {
                    match order.order_side {
                        FuturesOrderSide::OpenLong => {
                            // 限价开多：最低价格 <= 订单价格时执行
                            if low_price <= order.open_price {
                                // 限价单的成交价格应该是挂单的价格
                                self.execute_order(&order, order.open_price, execute_datetime).unwrap();
                            }
                        }
                        FuturesOrderSide::OpenShort => {
                            // 限价开空：最高价格 >= 订单价格时执行
                            if high_price >= order.open_price {
                                // 限价单的成交价格应该是挂单的价格
                                self.execute_order(&order, order.open_price, execute_datetime).unwrap();
                            }
                        }
                        _ => {}
                    }
                }
                OrderType::StopMarket => {
                    match order.order_side {
                        FuturesOrderSide::CloseLong => {
                            // 平多止损：最低价格 <= 止损价格时执行
                            if low_price <= order.open_price {
                                self.execute_sl_order(&order, execute_datetime)?;
                            }
                        }
                        FuturesOrderSide::CloseShort => {
                            // 平空止损：最高价格 >= 止损价格时执行
                            if high_price >= order.open_price {
                                self.execute_sl_order(&order, execute_datetime)?;
                            }
                        }
                        _ => {}
                    }
                }
                OrderType::TakeProfitMarket => {
                    match order.order_side {
                        FuturesOrderSide::CloseLong => {
                            // 平多止盈：最高价格 >= 止盈价格时执行
                            if high_price >= order.open_price {
                                self.execute_tp_order(&order, execute_datetime)?;
                            }
                        }
                        FuturesOrderSide::CloseShort => {
                            // 平空止盈：最低价格 <= 止盈价格时执行
                            if low_price <= order.open_price {
                                self.execute_tp_order(&order, execute_datetime)?;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn create_take_profit_order(&mut self, position: &VirtualPosition, execute_datetime: DateTime<Utc>) -> Option<VirtualOrder> {
        // 生成止盈订单
        if let Some(tp) = position.tp {
            let order_id = self.generate_order_id();

            let order_side = match position.position_side {
                PositionSide::Long => FuturesOrderSide::CloseLong,
                PositionSide::Short => FuturesOrderSide::CloseShort,
            };

            let tp_order = VirtualOrder::create_take_profit_order(
                position.position_id,
                order_id,
                position.strategy_id,
                position.node_id.clone(),
                position.node_name.clone(),
                position.order_config_id,
                position.exchange.clone(),
                position.symbol.clone(),
                order_side,
                position.quantity,
                tp,
                execute_datetime,
            );
            return Some(tp_order);
        }
        None
    }

    pub fn create_stop_loss_order(&mut self, position: &VirtualPosition, current_datetime: DateTime<Utc>) -> Option<VirtualOrder> {
        // 生成止盈止损订单
        if let Some(sl) = position.sl {
            let order_id = self.generate_order_id();

            let order_side = match position.position_side {
                PositionSide::Long => FuturesOrderSide::CloseLong,
                PositionSide::Short => FuturesOrderSide::CloseShort,
            };

            let sl_order = VirtualOrder::create_stop_loss_order(
                position.position_id,
                position.strategy_id,
                order_id,
                position.node_id.clone(),
                position.node_name.clone(),
                position.order_config_id,
                position.exchange.clone(),
                position.symbol.clone(),
                order_side,
                position.quantity, // 全部止盈
                sl,                // 止损订单的开仓价格，就是主订单中的止损价格
                current_datetime,
            );
            return Some(sl_order);
        }
        None
    }
}
