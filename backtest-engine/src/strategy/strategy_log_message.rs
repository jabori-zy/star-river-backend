use serde::{Deserialize, Serialize};
use strategy_core::{log_message, log_message::*};
log_message!(
    StrategyStateLogMsg,
    params: (
        strategy_name: String,
        strategy_state: String,
    ),
    en: "#[{strategy_name}] current state is: {strategy_state}",
    zh: "#[{strategy_name}] 当前状态是：{strategy_state}"
);


log_message!(
    LongLimitOrderExecutedDirectlyMsg,
    params: (
        strategy_name: String,
        limit_price: f64,
        current_price: f64,
        order_id: i32,
    ),
    en: "#[{strategy_name}] order price {limit_price} is greater than current price {current_price}, limit order executed directly - Order ID: {order_id}",
    zh: "#[{strategy_name}] 订单价格{limit_price}大于当前价格{current_price}, 限价单直接成交, 订单ID: {order_id}"
);



log_message!(
    ShortLimitOrderExecutedDirectlyMsg,
    params: (
        strategy_name: String,
        limit_price: f64,
        current_price: f64,
        order_id: i32,
    ),
    en: "#[{strategy_name}] order price {limit_price} is less than current price {current_price}, limit order executed directly - Order ID: {order_id}",
    zh: "#[{strategy_name}] 订单价格{limit_price}小于当前价格{current_price}, 限价单直接成交, 订单ID: {order_id}"
);
