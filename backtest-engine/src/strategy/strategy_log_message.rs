use serde::{Deserialize, Serialize};
use star_river_core::custom_type::OrderId;
use strategy_core::{log_message, log_message::*};
log_message!(
    StrategyRunStateLogMsg,
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

log_message!(
    FuturesOrderCreatedMsg,
    params: (
        strategy_name: String,
        order_id: OrderId,
        order_config_id: i32,
        price: f64,
        side: String,
    ),
    en: "#[{strategy_name}] futures order created - Order ID: {order_id}, Config ID: {order_config_id} price: {price} side: {side}",
    zh: "#[{strategy_name}] 期货订单创建 - 订单ID: {order_id}, 配置ID: {order_config_id} 价格: {price} 方向: {side}"
);

log_message!(
    FuturesOrderPlacedMsg,
    params: (
        strategy_name: String,
        order_id: OrderId,
        order_config_id: i32
    ),
    en: "#[{strategy_name}] futures order placed successfully - Order ID: {order_id}, Config ID: {order_config_id}",
    zh: "#[{strategy_name}] 订单下单成功 - 订单ID: {order_id}, 配置ID: {order_config_id}"
);

log_message!(
    FuturesOrderPartialFilledMsg,
    params: (
        strategy_name: String,
        order_id: OrderId,
        filled_quantity: f64,
        remaining_quantity: f64
    ),
    en: "#[{strategy_name}] futures order partially filled - Order ID: {order_id}, Filled: {filled_quantity}, Remaining: {remaining_quantity}",
    zh: "#[{strategy_name}] 订单部分成交 - 订单ID: {order_id}, 已成交: {filled_quantity}, 剩余: {remaining_quantity}"
);

log_message!(
    FuturesOrderFilledMsg,
    params: (
        strategy_name: String,
        order_id: OrderId,
        filled_quantity: f64,
        filled_price: f64
    ),
    en: "#[{strategy_name}] futures order completely filled - Order ID: {order_id}, Quantity: {filled_quantity}, Price: {filled_price}",
    zh: "#[{strategy_name}] 订单完全成交 - 订单ID: {order_id}, 数量: {filled_quantity}, 价格: {filled_price}"
);

log_message!(
    FuturesOrderCanceledMsg,
    params: (
        strategy_name: String,
        order_id: OrderId
    ),
    en: "#[{strategy_name}] futures order canceled - Order ID: {order_id}",
    zh: "#[{strategy_name}] 订单已取消 - 订单ID: {order_id}"
);

log_message!(
    FuturesOrderExpiredMsg,
    params: (
        strategy_name: String,
        order_id: OrderId
    ),
    en: "#[{strategy_name}] futures order expired - Order ID: {order_id}",
    zh: "#[{strategy_name}] 订单已过期 - 订单ID: {order_id}"
);

log_message!(
    FuturesOrderRejectedMsg,
    params: (
        strategy_name: String,
        order_id: OrderId,
        reason: String
    ),
    en: "#[{strategy_name}] futures order rejected - Order ID: {order_id}, Reason: {reason}",
    zh: "#[{strategy_name}] 订单被拒绝 - 订单ID: {order_id}, 原因: {reason}"
);

log_message!(
    FuturesOrderErrorMsg,
    params: (
        strategy_name: String,
        order_id: OrderId,
        error: String
    ),
    en: "#[{strategy_name}] futures order error - Order ID: {order_id}, Error: {error}",
    zh: "#[{strategy_name}] 订单错误 - 订单ID: {order_id}, 错误: {error}"
);
