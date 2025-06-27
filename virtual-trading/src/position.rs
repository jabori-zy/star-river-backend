use super::VirtualTradingSystem;
use types::position::virtual_position::VirtualPosition;
use types::order::virtual_order::VirtualOrder;
use types::order::OrderStatus;
use types::custom_type::*;
use types::transaction::virtual_transaction::VirtualTransaction;


impl VirtualTradingSystem {
    /// 执行订单, 返回持仓id
    /// 生成仓位和交易明细
    pub fn execute_order(&mut self, virtual_order: VirtualOrder, current_price: f64) -> PositionId {
        // tracing::info!("执行订单: {:?}, 成交价格: {:?}", virtual_order, current_price);
        // 执行订单，生成模拟仓位
        let virtual_position = VirtualPosition::new(&virtual_order, current_price);

        

        // 生成交易明细
        let transaction_id = self.get_transaction_id();
        let virtual_transaction = VirtualTransaction::new(transaction_id, &virtual_order, &virtual_position);

        // 修改订单的状态
        self.update_order_status(virtual_order.order_id, OrderStatus::Filled).unwrap();

        // 将交易明细添加到交易明细列表中
        self.transactions.push(virtual_transaction);

        let position_id = virtual_position.position_id;
        // 将仓位添加到当前持仓列表中
        self.current_positions.push(virtual_position);
        // 返回持仓id
        
        position_id
    }

    // 更新仓位
    pub fn update_position(&mut self) {
        for i in 0..self.current_positions.len() {
            let kline_cache_key = self.get_kline_cache_key(
                &self.current_positions[i].exchange, 
                &self.current_positions[i].symbol
            );
            
            if let Some(kline_cache_key) = kline_cache_key {
                if let Some(current_price) = self.kline_cache_data.get(&kline_cache_key) {
                    self.current_positions[i].update_position(*current_price);
                }
            }
        }
    }

}
