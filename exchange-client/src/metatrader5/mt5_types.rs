use serde::{Serialize, Deserialize};
use types::position::PositionNumberRequest;
use types::market::Exchange;
use types::position::PositionSide;



#[derive(Debug, Serialize)]
pub struct Mt5PositionNumberRequest {
    pub exchange: Exchange,
    pub symbol: String,
    pub position_side: Option<PositionSide>,
}

impl From<PositionNumberRequest> for Mt5PositionNumberRequest {
    fn from(value: PositionNumberRequest) -> Self {
        Mt5PositionNumberRequest {
            exchange: value.exchange,
            symbol: value.symbol,
            position_side: value.position_side
        }
        
    }

}