use types::position::{Position, PositionSide};
use crate::entities::position::Model as PositionModel;
use types::market::Exchange;
use std::str::FromStr;



impl From<PositionModel> for Position {
    fn from(position: PositionModel) -> Self {
        Position {
            position_id: position.id as i64,
            strategy_id: position.strategy_id as i64,
            node_id: position.node_id.clone(),
            exchange: Exchange::from_str(&position.exchange).expect("Invalid exchange"),
            account_id: position.account_id,
            exchange_position_id: position.exchange_position_id as i64,
            symbol: position.symbol,
            position_side: PositionSide::from_str(&position.position_side).expect("Invalid position side"),
            quantity: position.quantity,
            open_price: position.open_price,
            current_price: None,
            tp: position.tp,
            sl: position.sl,
            unrealized_profit: None,
            create_time: position.created_time.timestamp(),
            update_time: position.updated_time.timestamp(),
        }
    }
}
