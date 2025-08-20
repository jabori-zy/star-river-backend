use super::VirtualTradingSystem;
use types::custom_type::{Balance, Leverage, Margin, MarginRatio, Price};
use types::order::{FuturesOrderSide, virtual_order::VirtualOrder};
use types::position::PositionSide;
use types::position::virtual_position::VirtualPosition;



pub struct Statistics;

impl Statistics {
    // 计算保证金
    pub fn calculate_margin(leverage: Leverage, price: Price, quantity: f64) -> Margin {
        // 计算需要的初始保证金
        // 保证金 = 开仓价格 * 持仓量 / 杠杆倍数
        let margin = price * quantity / leverage as f64;
        margin
    }

    // 计算保证金率
    pub fn calculate_margin_ratio(current_balance: Balance, leverage: Leverage, price: Price, quantity: f64) -> MarginRatio {
        // 保证金率 = 保证金 / 保证金余额
        let margin_ratio = Self::calculate_margin(leverage, price, quantity) / current_balance;
        margin_ratio
    }

    // 计算强平价格
    pub fn calculate_force_price(position_side: &PositionSide, leverage: Leverage, price: Price, quantity: f64) -> Price {
        // 计算强平价格
        // 强平价格 = 开仓价格 - 保证金 / 持仓量
        let force_price = match position_side {
            PositionSide::Long   => price - Self::calculate_margin(leverage, price, quantity) / quantity, // 多仓的强平价格： 开仓价格 - 保证金 / 持仓量
            PositionSide::Short  => price + Self::calculate_margin(leverage, price, quantity) / quantity, // 空仓的强平价格： 开仓价格 + 保证金 / 持仓量
        };
        force_price
    }
    



}