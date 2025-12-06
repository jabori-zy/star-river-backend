// pub mod virtual_position;

use std::{any::Any, fmt::Debug, str::FromStr};

use entity::position::Model as PositionModel;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use utoipa::ToSchema;

use crate::{exchange::Exchange, system::DateTimeUtc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPositionNumberParam {
    pub strategy_id: i32,
    pub node_id: String,
    pub position_number_request: GetPositionNumberParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPositionParam {
    pub strategy_id: i32,
    pub node_id: String,
    pub exchange: Exchange,
    pub position_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Display, ToSchema)]
// Position side
pub enum PositionSide {
    #[strum(serialize = "long")]
    Long,
    #[strum(serialize = "short")]
    Short,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display, ToSchema, PartialEq, Eq)]
pub enum PositionState {
    #[strum(serialize = "open")]
    Open, // Open position
    #[strum(serialize = "closed")]
    Closed, // Closed position
    #[strum(serialize = "partially_closed")]
    PartiallyClosed, // Partially closed
    #[strum(serialize = "forced_closed")]
    ForcedClosed, // Force closed
}

// Original position information returned by exchange
pub trait OriginalPosition: Debug + Send + Sync + Any + 'static {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn OriginalPosition>;
    fn get_exchange_position_id(&self) -> i64;
    fn get_symbol(&self) -> String;
    fn get_position_side(&self) -> PositionSide;
    fn get_quantity(&self) -> f64;
    fn get_open_price(&self) -> f64;
    fn get_tp(&self) -> Option<f64>;
    fn get_sl(&self) -> Option<f64>;
    fn get_exchange(&self) -> Exchange;
    fn get_unrealized_profit(&self) -> Option<f64>;
    fn get_extra_info(&self) -> Option<serde_json::Value>;
    fn get_create_time(&self) -> DateTimeUtc;
    fn get_update_time(&self) -> DateTimeUtc;
}

impl Clone for Box<dyn OriginalPosition> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub position_id: i32,
    pub strategy_id: i64,
    pub node_id: String,
    pub account_id: i32,
    pub exchange: Exchange,
    pub exchange_position_id: i64,
    pub symbol: String,
    pub position_side: PositionSide,
    pub position_state: PositionState, // Position state
    pub quantity: f64,
    pub open_price: f64,
    pub current_price: Option<f64>,
    pub tp: Option<f64>,
    pub sl: Option<f64>,
    pub unrealized_profit: Option<f64>,        // Unrealized profit and loss
    pub extra_info: Option<serde_json::Value>, // Extra info
    pub create_time: DateTimeUtc,
    pub update_time: DateTimeUtc,
}

impl From<PositionModel> for Position {
    fn from(model: PositionModel) -> Self {
        Self {
            position_id: model.id,
            strategy_id: model.strategy_id as i64,
            node_id: model.node_id,
            account_id: model.account_id,
            exchange: Exchange::from_str(&model.exchange).unwrap(),
            exchange_position_id: model.exchange_position_id as i64,
            symbol: model.symbol,
            position_side: PositionSide::from_str(&model.position_side).unwrap(),
            position_state: PositionState::from_str(&model.position_state).unwrap(),
            quantity: model.quantity,
            open_price: model.open_price,
            current_price: None,
            tp: model.tp,
            sl: model.sl,
            unrealized_profit: model.unrealized_profit,
            extra_info: model.extra_info,
            create_time: model.created_time,
            update_time: model.updated_time,
        }
    }
}

// Position count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPositionNumberParams {
    pub exchange: Exchange,
    pub symbol: String,
    pub position_side: Option<PositionSide>,
}

// Position number
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNumber {
    pub exchange: Exchange,
    pub symbol: String,
    pub position_side: Option<PositionSide>,
    pub position_number: i32,
}
