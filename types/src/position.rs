use crate::market::Exchange;
use strum::EnumString;
use strum::Display;
use serde::{Serialize, Deserialize};



#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Display)]
// 订单方向
pub enum PositionSide {
    #[strum(serialize = "long")]
    Long,
    #[strum(serialize = "short")]
    Short,
}

// 订单数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNumberRequest {
    pub exchange: Exchange,
    pub symbol: String,
    pub position_side: Option<PositionSide>,
}


// 仓位数量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNumber {
    pub exchange: Exchange,
    pub symbol: String,
    pub position_side: Option<PositionSide>,
    pub position_number: i32
}