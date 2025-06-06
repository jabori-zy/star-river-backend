use super::VirtualTradingSystem;
use types::position::virtual_position::VirtualPosition;
use types::order::virtual_order::VirtualOrder;
use types::custom_type::*;


impl VirtualTradingSystem {
    // 执行订单
    pub fn execute_order(&mut self, virtual_order: &VirtualOrder, current_price: f64) -> PositionId {
        // tracing::info!("执行订单: {:?}, 成交价格: {:?}", virtual_order, current_price);
        let virtual_position = VirtualPosition::new(virtual_order, current_price);
        let position_id = virtual_position.position_id;
        self.current_positions.push(virtual_position);
        position_id
    }
}
