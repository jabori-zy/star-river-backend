use super::VirtualTradingSystemContext;
use crate::utils::Formula;

impl<E> VirtualTradingSystemContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    // 更新未实现盈亏
    pub fn update_unrealized_pnl(&mut self) {
        self.unrealized_pnl = self.current_positions.iter().map(|position| position.unrealized_profit).sum();
    }

    // 更新已实现盈亏
    pub fn update_realized_pnl(&mut self) {
        // if profit is None, set default value to 0.0
        self.realized_pnl = self.transactions.iter().map(|transaction| transaction.profit.unwrap_or(0.0)).sum();
    }

    // 更新已使用保证金
    pub fn update_used_margin(&mut self) {
        self.used_margin = self.current_positions.iter().map(|position| position.margin).sum();
    }

    // 更新保证金率
    pub fn update_margin_ratio(&mut self) {
        if self.equity == 0.0 {
            self.margin_ratio = 0.0;
        } else {
            self.margin_ratio = self.used_margin / self.equity;
        }
    }

    // 更新冻结保证金
    pub fn update_frozen_margin(&mut self) {
        self.frozen_margin = self
            .unfilled_orders
            .iter()
            .map(|order| Formula::calculate_margin(self.leverage, order.open_price, order.quantity))
            .sum();
    }

    // 更新账户余额
    pub fn update_balance(&mut self) {
        self.balance = self.initial_balance + self.realized_pnl;
    }

    // 更新净值
    pub fn update_equity(&mut self) {
        self.equity = self.balance + self.unrealized_pnl;
    }

    // 更新可用余额
    pub fn update_available_balance(&mut self) {
        self.available_balance = self.equity - self.used_margin - self.frozen_margin;
    }
}
