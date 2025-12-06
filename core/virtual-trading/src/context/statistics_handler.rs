use super::VtsContext;
use crate::utils::Formula;

impl<E> VtsContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    // Update unrealized profit/loss
    pub fn update_unrealized_pnl(&mut self) {
        self.unrealized_pnl = self.current_positions.iter().map(|position| position.unrealized_profit).sum();
    }

    // Update realized profit/loss
    pub fn update_realized_pnl(&mut self) {
        // if profit is None, set default value to 0.0
        self.realized_pnl = self.transactions.iter().map(|transaction| transaction.profit.unwrap_or(0.0)).sum();
    }

    // Update used margin
    pub fn update_used_margin(&mut self) {
        self.used_margin = self.current_positions.iter().map(|position| position.margin).sum();
    }

    // Update margin ratio
    pub fn update_margin_ratio(&mut self) {
        if self.equity == 0.0 {
            self.margin_ratio = 0.0;
        } else {
            self.margin_ratio = self.used_margin / self.equity;
        }
    }

    // Update frozen margin
    pub fn update_frozen_margin(&mut self) {
        self.frozen_margin = self
            .unfilled_orders
            .iter()
            .map(|order| Formula::calculate_margin(self.leverage, order.open_price, order.quantity))
            .sum();
    }

    // Update account balance
    pub fn update_balance(&mut self) {
        self.balance = self.initial_balance + self.realized_pnl;
    }

    // Update equity
    pub fn update_equity(&mut self) {
        self.equity = self.balance + self.unrealized_pnl;
    }

    // Update available balance
    pub fn update_available_balance(&mut self) {
        self.available_balance = self.equity - self.used_margin - self.frozen_margin;
    }
}
