use super::VirtualTradingSystem;


impl VirtualTradingSystem {
    
    
    // 更新未实现盈亏
    pub fn update_unrealized_pnl(&mut self) {
        self.unrealized_pnl = self.current_positions.iter().map(|position| position.unrealized_profit).sum();
    }
    



}