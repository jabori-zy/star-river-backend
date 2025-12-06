use std::{collections::HashMap, str::FromStr};

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use backtest_engine::engine_error::BacktestEngineError;
use chrono::DateTime;
use engine_core::EngineContextAccessor;
use key::Key;
use serde::{Deserialize, Serialize};
use star_river_core::{custom_type::NodeId, error::StarRiverErrorTrait};
use strategy_core::{
    benchmark::strategy_benchmark::StrategyPerformanceReport,
    event::node_common_event::NodeRunningLogEvent,
    strategy::context_trait::{StrategyBenchmarkExt, StrategyVariableExt},
    variable::StrategyVariable,
};
use strategy_stats::StatsSnapshot;
use tracing::instrument;
use utoipa::{IntoParams, ToSchema};
use virtual_trading::types::{VirtualOrder, VirtualPosition, VirtualTransaction};

use super::BACKTEST_CONTROL_TAG;
use crate::{
    api::response::{ApiResponseEnum, NewApiResponse},
    star_river::StarRiver,
};

#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/virtual-orders",
    tag = "Backtest Strategy",
    summary = "Get virtual orders",
    params(
        ("strategy_id" = i32, Path, description = "The ID of the strategy to get virtual orders")
    ),
    responses(
        (status = 200, description = "Get virtual orders successfully", body = NewApiResponse<Vec<VirtualOrder>>),
        (status = 400, description = "Get virtual orders failed", body = NewApiResponse<Vec<VirtualOrder>>)
    ))]
pub async fn get_virtual_orders(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<NewApiResponse<Vec<VirtualOrder>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<Vec<VirtualOrder>, BacktestEngineError> = engine_guard
        .with_ctx_read_async(|ctx| {
            Box::pin(async move {
                ctx.with_strategy_ctx_read_async(strategy_id, |ctx| Box::pin(async move { ctx.get_virtual_orders().await }))
                    .await
            })
        })
        .await;

    match result {
        Ok(virtual_orders) => (StatusCode::OK, Json(NewApiResponse::success(virtual_orders))),
        Err(e) => {
            let status_code = match &e {
                _ => StatusCode::NOT_FOUND,
            };
            (status_code, Json(NewApiResponse::error(e)))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/current-positions",
    tag = "Backtest Strategy",
    summary = "Get current virtual positions",
    params(
        ("strategy_id" = i32, Path, description = "The ID of the strategy to get current virtual positions")
    ),
    responses(
        (status = 200, description = "Get current virtual positions successfully", body = NewApiResponse<Vec<VirtualPosition>>),
        (status = 400, description = "Get current virtual positions failed", body = NewApiResponse<Vec<VirtualPosition>>)
    )
)]
pub async fn get_current_positions(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<NewApiResponse<Vec<VirtualPosition>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<Vec<VirtualPosition>, BacktestEngineError> = engine_guard
        .with_ctx_read_async(|ctx| {
            Box::pin(async move {
                ctx.with_strategy_ctx_read_async(strategy_id, |ctx| Box::pin(async move { ctx.get_current_positions().await }))
                    .await
            })
        })
        .await;

    match result {
        Ok(current_positions) => (StatusCode::OK, Json(NewApiResponse::success(current_positions))),
        Err(e) => (StatusCode::NOT_FOUND, Json(NewApiResponse::error(e))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/virtual-transactions",
    tag = "Backtest Strategy",
    summary = "Get virtual transactions",
    params(
        ("strategy_id" = i32, Path, description = "The ID of the strategy to get virtual transactions")
    ),
    responses(
        (status = 200, description = "Get virtual transactions successfully", body = NewApiResponse<Vec<VirtualTransaction>>),
        (status = 400, description = "Get virtual transactions failed", body = NewApiResponse<Vec<VirtualTransaction>>)
    )
)]
pub async fn get_virtual_transactions(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<NewApiResponse<Vec<VirtualTransaction>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<Vec<VirtualTransaction>, BacktestEngineError> = engine_guard
        .with_ctx_read_async(|ctx| {
            Box::pin(async move {
                ctx.with_strategy_ctx_read_async(strategy_id, |ctx| Box::pin(async move { ctx.get_transactions().await }))
                    .await
            })
        })
        .await;

    match result {
        Ok(virtual_transactions) => (StatusCode::OK, Json(NewApiResponse::success(virtual_transactions))),
        Err(e) => (StatusCode::NOT_FOUND, Json(NewApiResponse::error(e))),
    }
}

#[derive(Serialize, Deserialize, IntoParams, ToSchema)]
#[schema(
    title = "Get strategy statistics history before play index",
    description = "Get strategy statistics history before play index",
    example = json!({
        "datetime": "2024-01-01T00:00:00.000Z"
    })
)]
pub struct GetStatsHistoryQuery {
    pub datetime: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/stats-history",
    tag = "Backtest Strategy",
    summary = "Get strategy statistics history",
    params(
        ("strategy_id" = i32, Path, description = "The ID of the strategy to get statistics history"),
        ("play_index" = i32, Query, description = "The play index to get statistics history"),
    ),
    responses(
        (status = 200, description = "Get strategy statistics history successfully", body = NewApiResponse<Vec<StatsSnapshot>>),
        (status = 400, description = "Get strategy statistics history failed", body = NewApiResponse<Vec<StatsSnapshot>>)
    )
)]
#[axum::debug_handler]
pub async fn get_stats_history(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
    Query(params): Query<GetStatsHistoryQuery>,
) -> (StatusCode, Json<NewApiResponse<Vec<StatsSnapshot>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    // tracing::info!("get stats history: {}", params.datetime);
    let datetime = DateTime::parse_from_rfc3339(&params.datetime).unwrap().to_utc();

    let result: Result<Vec<StatsSnapshot>, BacktestEngineError> = engine_guard
        .with_ctx_read_async(|ctx| {
            Box::pin(async move {
                ctx.with_strategy_ctx_read_async(strategy_id, move |ctx| {
                    Box::pin(async move { ctx.get_stats_history(datetime).await })
                })
                .await
            })
        })
        .await;

    match result {
        Ok(stats_history) => (StatusCode::OK, Json(NewApiResponse::success(stats_history))),
        Err(e) => (StatusCode::NOT_FOUND, Json(NewApiResponse::error(e))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/history-positions",
    tag = "Backtest Strategy",
    summary = "Get history virtual positions",
    params(
        ("strategy_id" = i32, Path, description = "The ID of the strategy to get history virtual positions")
    ),
    responses(
        (status = 200, description = "Get history virtual positions successfully", body = NewApiResponse<Vec<VirtualPosition>>),
        (status = 400, description = "Get history virtual positions failed", body = NewApiResponse<Vec<VirtualPosition>>)
    )
)]
pub async fn get_history_positions(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<NewApiResponse<Vec<VirtualPosition>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<Vec<VirtualPosition>, BacktestEngineError> = engine_guard
        .with_ctx_read_async(|ctx| {
            Box::pin(async move {
                ctx.with_strategy_ctx_read_async(strategy_id, |ctx| Box::pin(async move { ctx.get_history_positions().await }))
                    .await
            })
        })
        .await;

    match result {
        Ok(history_positions) => (StatusCode::OK, Json(NewApiResponse::success(history_positions))),
        Err(e) => (StatusCode::NOT_FOUND, Json(NewApiResponse::error(e))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/run-state",
    tag = BACKTEST_CONTROL_TAG,
    summary = "Get strategy run state",
    params(
        ("strategy_id" = i32, Path, description = "The ID of the strategy to get run state")
    ),
    responses(
        (status = 200, description = "Get strategy run state successfully", body = ApiResponseEnum<String>),
        (status = 400, description = "Get strategy run state failed", body = ApiResponseEnum<String>)
    )
)]
pub async fn get_strategy_run_state(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<ApiResponseEnum<String>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result = engine_guard
        .with_ctx_read_async(|ctx| Box::pin(async move { ctx.get_strategy_run_state(strategy_id).await }))
        .await;

    match result {
        Ok(status) => (StatusCode::OK, Json(ApiResponseEnum::success(status))),
        Err(e) => {
            e.report_log();
            (e.http_status_code(), Json(ApiResponseEnum::error(e)))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/running-log",
    tag = "Backtest Strategy",
    summary = "Get running log",
    params(
        ("strategy_id" = i32, Path, description = "The ID of the strategy to get running log")
    ),
    responses(
        (status = 200, description = "Get running log successfully", body = NewApiResponse<Vec<NodeRunningLogEvent>>),
        (status = 400, description = "Get running log failed", body = NewApiResponse<Vec<NodeRunningLogEvent>>)
    )
)]
pub async fn get_running_log(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<NewApiResponse<Vec<NodeRunningLogEvent>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<Vec<NodeRunningLogEvent>, BacktestEngineError> = engine_guard
        .with_ctx_read_async(|ctx| {
            Box::pin(async move {
                ctx.with_strategy_ctx_read_async(strategy_id, |ctx| Box::pin(async move { ctx.running_log().await }))
                    .await
            })
        })
        .await;

    match result {
        Ok(running_log) => (StatusCode::OK, Json(NewApiResponse::success(running_log))),
        Err(e) => (StatusCode::NOT_FOUND, Json(NewApiResponse::error(e))),
    }
}

#[derive(Serialize, Deserialize, IntoParams, ToSchema, Debug)]
#[schema(
    title = "get strategy data",
    description = "get strategy data",
    example = json!({
        "datetime": "2024-01-01T00:00:00.000Z",
        "key": ""
    })
)]
pub struct GetStrategyDataQuery {
    pub datetime: Option<String>,
    pub key: String,
    pub limit: Option<i32>,
}

#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/data",
    tag = "Backtest Strategy",
    summary = "Get strategy data",
    params(
        GetStrategyDataQuery
    ),
    responses(
        (status = 200, description = "Get strategy data successfully", body = NewApiResponse<Vec<utoipa::openapi::Object>>),
        (status = 400, description = "Get strategy data failed", body = NewApiResponse<Vec<utoipa::openapi::Object>>)
    )
)]
pub async fn get_strategy_data(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
    Query(params): Query<GetStrategyDataQuery>,
) -> (StatusCode, Json<NewApiResponse<Vec<serde_json::Value>>>) {
    let key = match Key::from_str(&params.key) {
        Ok(key) => key,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(NewApiResponse::error(e))),
    };

    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let datetime = match params.datetime {
        Some(dt) => {
            // tracing::info!("get strategy data datetime: {}", dt);
            let datetime = DateTime::parse_from_rfc3339(&dt).unwrap().to_utc();

            Some(datetime)
        }
        None => None,
    };

    let limit = params.limit;
    let result: Result<Vec<serde_json::Value>, BacktestEngineError> = engine_guard
        .with_ctx_read_async(|ctx| {
            Box::pin(async move {
                let data = ctx
                    .with_strategy_ctx_read_async(strategy_id, move |ctx| {
                        Box::pin(async move { ctx.get_strategy_data(datetime, None, key, limit).await })
                    })
                    .await?
                    .map_err(BacktestEngineError::from)?;
                Ok(data)
            })
        })
        .await;

    match result {
        Ok(data) => (StatusCode::OK, Json(NewApiResponse::success(data))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(NewApiResponse::error(e))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/variable",
    tag = "Backtest Strategy",
    summary = "Get strategy variable",
    params(
        ("strategy_id" = i32, Path, description = "The ID of the strategy to get strategy variable")
    ),
    responses(
        (status = 200, description = "Get strategy variable successfully", body = NewApiResponse<Vec<StrategyVariable>>),
        (status = 400, description = "Get strategy variable failed", body = NewApiResponse<Vec<StrategyVariable>>)
    )
)]
#[axum::debug_handler]
pub async fn get_strategy_variable(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<NewApiResponse<Vec<StrategyVariable>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<Vec<StrategyVariable>, BacktestEngineError> = engine_guard
        .with_ctx_read_async(|ctx| {
            Box::pin(async move {
                ctx.with_strategy_ctx_read_async(strategy_id, |ctx| Box::pin(async move { ctx.strategy_variables().await }))
                    .await
            })
        })
        .await;

    match result {
        Ok(strategy_variable) => (StatusCode::OK, Json(NewApiResponse::success(strategy_variable))),
        Err(e) => (StatusCode::NOT_FOUND, Json(NewApiResponse::error(e))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/performance-report",
    tag = "Backtest Strategy",
    summary = "Get strategy performance report",
    params(
        ("strategy_id" = i32, Path, description = "The ID of the strategy to get strategy performance report")
    ),
    responses(
        (status = 200, description = "Get strategy performance report successfully", body = NewApiResponse<StrategyPerformanceReport>),
        (status = 400, description = "Get strategy performance report failed", body = NewApiResponse<StrategyPerformanceReport>)
    )
)]
#[axum::debug_handler]
pub async fn get_strategy_performance_report(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<NewApiResponse<StrategyPerformanceReport>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<StrategyPerformanceReport, BacktestEngineError> = engine_guard
        .with_ctx_read_async(|ctx| {
            Box::pin(async move {
                ctx.with_strategy_ctx_read_async(strategy_id, |ctx| Box::pin(async move { ctx.strategy_performance_report().await }))
                    .await
            })
        })
        .await;

    match result {
        Ok(strategy_performance_report) => (StatusCode::OK, Json(NewApiResponse::success(strategy_performance_report))),
        Err(e) => (StatusCode::NOT_FOUND, Json(NewApiResponse::error(e))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/strategy/{strategy_id}/cache-keys",
    tag = "Strategy Management",
    summary = "Get strategy cache keys",
    params(
        ("strategy_id" = i32, Path, description = "The ID of the strategy to get cache keys"),
    ),
    responses(
        (status = 200, description = "Get strategy cache keys successfully", content_type = "application/json")
    )
)]
#[instrument(skip(star_river))]
pub async fn get_strategy_keys(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<NewApiResponse<Vec<String>>>) {
    tracing::info!(strategy_id = strategy_id, "get strategy cache keys");
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<HashMap<Key, NodeId>, BacktestEngineError> = engine_guard
        .with_ctx_read_async(|ctx| {
            Box::pin(async move {
                ctx.with_strategy_ctx_read_async(strategy_id, |ctx| Box::pin(async move { ctx.keys().await }))
                    .await
            })
        })
        .await;

    match result {
        Ok(keys_map) => {
            let keys_str = keys_map.keys().map(|cache_key| cache_key.get_key_str()).collect::<Vec<String>>();
            (StatusCode::OK, Json(NewApiResponse::success(keys_str)))
        }
        Err(e) => (e.http_status_code(), Json(NewApiResponse::error(e))),
    }
}
