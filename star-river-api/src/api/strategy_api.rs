pub mod backtest;
pub mod strategy_management;

pub use strategy_management::update_strategy;

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use backtest_engine::engine_error::BacktestEngineError;
use database::mutation::strategy_config_mutation::StrategyConfigMutation;
use engine_core::EngineContextAccessor;
use serde::{Deserialize, Serialize};
use strategy_core::strategy::StrategyConfig;
use tracing::instrument;
use utoipa::{IntoParams, ToSchema};
use star_river_core::error::StarRiverErrorTrait;

use crate::{
    api::response::{ApiResponse, NewApiResponse},
    star_river::StarRiver,
};












