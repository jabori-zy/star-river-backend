// External crate imports
use chrono::{DateTime, Utc};
use snafu::Report;
// Current crate imports
use star_river_core::{
    custom_type::*,
    exchange::Exchange,
    order::{FuturesOrderSide, OrderStatus, OrderType, TpslType},
    position::PositionSide,
};

// Local module imports
use super::VirtualTradingSystem;
use crate::{
    error::{KlineKeyNotFoundSnafu, UnsupportedOrderTypeSnafu, VirtualTradingSystemError},
    event::VirtualTradingSystemEvent,
    types::{VirtualOrder, VirtualPosition},
};

impl VirtualTradingSystem {
    // 生成订单ID, 从0开始
    fn generate_order_id(&self) -> i32 {
        self.orders.len() as i32
    }

    // 创建订单
    pub fn create_order(
        &mut self,
        strategy_id: i32,
        node_id: String,
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
        let kline_key = self.get_kline_key(&exchange, &symbol);
        let current_datetime = self.current_datetime;
        if let Some(kline_key) = kline_key {
            // 根据订单类型判断是否需要立即成交
            match &order_type {
                // 市价单
                OrderType::Market => {
                    // 市价单立即成交
                    // 获取当前价格
                    let current_kline = self.kline_price.get(&kline_key).unwrap();
                    let current_price = current_kline.close;
                    let current_datetime = current_kline.datetime;
                    // 市价单忽略创建订单时的价格，而是使用最新的价格
                    let market_order = VirtualOrder::new(
                        None,
                        order_id,
                        strategy_id,
                        node_id,
                        order_config_id,
                        exchange.clone(),
                        symbol.clone(),
                        order_side,
                        order_type,
                        quantity,
                        current_price,
                        tp,
                        sl,
                        tp_type,
                        sl_type,
                        point,
                        current_datetime,
                    );
                    tracing::debug!("market order created: {:?}", market_order);

                    let order_create_event = VirtualTradingSystemEvent::FuturesOrderCreated(market_order.clone());
                    let _ = self.event_publisher.send(order_create_event);

                    // 插入订单
                    self.orders.push(market_order.clone());

                    // 创建完成后，直接成交订单
                    self.execute_order(&market_order, current_price, current_datetime).unwrap();
                }
                // 限价单
                OrderType::Limit => {
                    // 限价单不立即成交
                    let limit_order = VirtualOrder::new(
                        None,
                        order_id,
                        strategy_id,
                        node_id,
                        order_config_id,
                        exchange,
                        symbol,
                        order_side,
                        order_type,
                        quantity,
                        price,
                        tp,
                        sl,
                        tp_type,
                        sl_type,
                        point,
                        current_datetime,
                    );
                    tracing::debug!("limit order created: {:?}", limit_order);
                    let order_create_event = VirtualTradingSystemEvent::FuturesOrderCreated(limit_order.clone());
                    let _ = self.event_publisher.send(order_create_event);
                    // 插入订单
                    self.orders.push(limit_order.clone());
                }
                _ => {
                    let error = UnsupportedOrderTypeSnafu {
                        order_type: order_type.to_string(),
                    }
                    .build();
                    let report = Report::from_error(&error);
                    tracing::error!("{}", report);
                    return Err(error);
                }
            }
        } else {
            // 如果k线缓存key不存在，则不成交
            let error = KlineKeyNotFoundSnafu {
                exchange: exchange.to_string(),
                symbol: symbol.to_string(),
            }
            .build();
            let report = Report::from_error(&error);
            tracing::error!("{}", report);
            return Err(error);
        }
        Ok(())
    }

    pub fn update_order_status(
        &mut self,
        order_id: OrderId,
        order_status: OrderStatus,
        datetime: DateTime<Utc>,
    ) -> Result<VirtualOrder, String> {
        if let Some(order) = self.orders.iter_mut().find(|o| o.order_id == order_id) {
            // 更新订单状态
            order.order_status = order_status;
            // 更新订单更新时间
            order.update_time = datetime;
            Ok(order.clone())
        } else {
            Err(format!("订单不存在: {:?}", order_id))
        }
    }

    pub fn update_order_position_id(&mut self, order_id: OrderId, position_id: PositionId, datetime: DateTime<Utc>) -> Result<(), String> {
        if let Some(order) = self.orders.iter_mut().find(|o| o.order_id == order_id) {
            order.position_id = Some(position_id);
            order.update_time = datetime;
            Ok(())
        } else {
            Err(format!("订单不存在: {:?}", order_id))
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

    // 检查未成交订单（包括挂单，止盈止损订单），如果未成交，则立即成交
    pub fn check_unfilled_orders(&mut self) {
        // 获取未成交订单
        let unfilled_orders = self.get_unfilled_orders();
        for order in unfilled_orders {
            if let Some(kline_key) = self.get_kline_key(&order.exchange, &order.symbol) {
                if let Some(current_kline) = self.kline_price.get(&kline_key) {
                    let high_price = current_kline.high;
                    let low_price = current_kline.low;

                    match order.order_type {
                        OrderType::Limit => {
                            match order.order_side {
                                FuturesOrderSide::OpenLong => {
                                    // 限价开多：最低价格 <= 订单价格时执行
                                    if low_price <= order.open_price {
                                        // 限价单的成交价格应该是挂单的价格
                                        self.execute_order(&order, order.open_price, self.current_datetime).unwrap();
                                    }
                                }
                                FuturesOrderSide::OpenShort => {
                                    // 限价开空：最高价格 >= 订单价格时执行
                                    if high_price >= order.open_price {
                                        // 限价单的成交价格应该是挂单的价格
                                        self.execute_order(&order, order.open_price, self.current_datetime).unwrap();
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
                                        self.execute_sl_order(&order);
                                    }
                                }
                                FuturesOrderSide::CloseShort => {
                                    // 平空止损：最高价格 >= 止损价格时执行
                                    if high_price >= order.open_price {
                                        self.execute_sl_order(&order);
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
                                        self.execute_tp_order(&order);
                                    }
                                }
                                FuturesOrderSide::CloseShort => {
                                    // 平空止盈：最低价格 <= 止盈价格时执行
                                    if low_price <= order.open_price {
                                        self.execute_tp_order(&order);
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn create_take_profit_order(&mut self, position: &VirtualPosition) -> Option<VirtualOrder> {
        // 生成止盈订单
        if let Some(tp) = position.tp {
            let order_id = self.generate_order_id();

            let order_side = match position.position_side {
                PositionSide::Long => FuturesOrderSide::CloseLong,
                PositionSide::Short => FuturesOrderSide::CloseShort,
            };

            let tp_order = VirtualOrder::new(
                Some(position.position_id),
                order_id,
                position.strategy_id,
                position.node_id.clone(),
                position.order_config_id,
                position.exchange.clone(),
                position.symbol.clone(),
                order_side,
                OrderType::TakeProfitMarket, // 虚拟交易系统，只使用市价止盈
                position.quantity,           // 全部止盈
                tp,                          // 止盈订单的开仓价格，就是主订单中的止盈价格
                None,
                None,
                None,
                None,
                None,
                self.current_datetime,
            );
            return Some(tp_order);
        }
        None
    }

    pub fn create_stop_loss_order(&mut self, position: &VirtualPosition) -> Option<VirtualOrder> {
        // 生成止盈止损订单
        if let Some(sl) = position.sl {
            let order_id = self.generate_order_id();

            let order_side = match position.position_side {
                PositionSide::Long => FuturesOrderSide::CloseLong,
                PositionSide::Short => FuturesOrderSide::CloseShort,
            };

            let sl_order = VirtualOrder::new(
                Some(position.position_id),
                order_id,
                position.strategy_id,
                position.node_id.clone(),
                position.order_config_id,
                position.exchange.clone(),
                position.symbol.clone(),
                order_side,
                OrderType::StopMarket, // 虚拟交易系统，只使用市价止损
                position.quantity,     // 全部止盈
                sl,                    // 止损订单的开仓价格，就是主订单中的止损价格
                None,
                None,
                None,
                None,
                None,
                self.current_datetime,
            );
            return Some(sl_order);
        }
        None
    }
}
