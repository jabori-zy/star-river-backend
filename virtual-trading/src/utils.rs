use super::VirtualTradingSystem;
use types::custom_type::{Margin, MarginRatio, Price};
use types::order::virtual_order::VirtualOrder;


impl VirtualTradingSystem {


    
    
    
    // 计算保证金
    pub fn calculate_margin(&self, price: Price, quantity: f64) -> Margin {
        // 计算需要的初始保证金
        // 保证金 = 开仓价格 * 持仓量 / 杠杆倍数
        let margin = price * quantity / self.leverage as f64;
        margin
    }

    // 计算保证金率
    pub fn calculate_margin_ratio(&self, price: Price, quantity: f64) -> MarginRatio {
        // 保证金率 = 保证金 / 保证金余额
        let margin_ratio = self.calculate_margin(price, quantity) / self.current_balance;
        margin_ratio
    }

    // 计算强平价格
    pub fn calculate_force_price(&self, price: Price, quantity: f64) -> Price {
        // 计算强平价格
        // 强平价格 = 开仓价格 - 保证金 / 持仓量
        let force_price = price - self.calculate_margin(price, quantity) / quantity;
        force_price
    }
    



}