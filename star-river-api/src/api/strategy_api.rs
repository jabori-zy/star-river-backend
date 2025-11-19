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











#[utoipa::path(
    post,
    path = "/api/v1/strategy/{strategy_id}/stop",
    tag = "Strategy Management",
    summary = "stop strategy",
    params(
        ("strategy_id" = i32, Path, description = "The ID of the strategy to stop")
    ),
    responses(
        (status = 200, description = "Stop strategy successfully", content_type = "application/json")
    )
)]
#[instrument(skip(star_river))]
pub async fn stop_strategy(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<NewApiResponse<()>>) {
    tracing::info!(strategy_id = strategy_id, "stop strategy");
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<(), BacktestEngineError> = engine_guard
        .with_ctx_write_async(|ctx| Box::pin(async move { ctx.stop(strategy_id).await }))
        .await;

    if let Err(e) = result {
        
        return (e.http_status_code(), Json(NewApiResponse::error(e)));
    }

    (StatusCode::OK, Json(NewApiResponse::success(())))
}
