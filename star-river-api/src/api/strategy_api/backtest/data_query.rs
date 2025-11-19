use std::{collections::HashMap, str::FromStr};

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use backtest_engine::engine_error::BacktestEngineError;
use chrono::NaiveDateTime;
use engine_core::EngineContextAccessor;
use key::Key;
use serde::{Deserialize, Serialize};
use snafu::IntoError;
use star_river_core::{custom_type::NodeId};
use strategy_core::{
    benchmark::strategy_benchmark::StrategyPerformanceReport,
    event::log_event::StrategyRunningLogEvent,
    strategy::context_trait::{StrategyBenchmarkExt, StrategyVariableExt},
    variable::StrategyVariable,
};
use strategy_stats::StatsSnapshot;
use tracing::instrument;
use utoipa::{IntoParams, ToSchema};
use virtual_trading::types::{VirtualOrder, VirtualPosition, VirtualTransaction};

use crate::{api::response::NewApiResponse, error::{ApiError, ParseDataTimeFailedSnafu}, star_river::StarRiver};
use star_river_core::error::StarRiverErrorTrait;

#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/virtual-orders",
    tag = "回测策略",
    summary = "获取虚拟订单",
    params(
        ("strategy_id" = i32, Path, description = "要获取虚拟订单的策略ID")
    ),
    responses(
        (status = 200, description = "获取虚拟订单成功", body = NewApiResponse<Vec<VirtualOrder>>),
        (status = 400, description = "获取虚拟订单失败", body = NewApiResponse<Vec<VirtualOrder>>)
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
    tag = "回测策略",
    summary = "获取当前虚拟持仓",
    params(
        ("strategy_id" = i32, Path, description = "要获取当前虚拟持仓的策略ID")
    ),
    responses(
        (status = 200, description = "获取当前虚拟持仓成功", body = NewApiResponse<Vec<VirtualPosition>>),
        (status = 400, description = "获取当前虚拟持仓失败", body = NewApiResponse<Vec<VirtualPosition>>)
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
    tag = "回测策略",
    summary = "获取虚拟交易明细",
    params(
        ("strategy_id" = i32, Path, description = "要获取虚拟交易明细的策略ID")
    ),
    responses(
        (status = 200, description = "获取虚拟交易明细成功", body = NewApiResponse<Vec<VirtualTransaction>>),
        (status = 400, description = "获取虚拟交易明细失败", body = NewApiResponse<Vec<VirtualTransaction>>)
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
    title = "获取播放索引之前的策略统计历史",
    description = "获取播放索引之前的策略统计历史",
    example = json!({
        "play_index": 1
    })
)]
pub struct GetStatsHistoryQuery {
    pub play_index: i32,
}

#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/stats-history",
    tag = "回测策略",
    summary = "获取策略统计历史",
    params(
        ("strategy_id" = i32, Path, description = "要获取策略统计历史的策略ID"),
        ("play_index" = i32, Query, description = "要获取策略统计历史的播放索引"),
    ),
    responses(
        (status = 200, description = "获取策略统计历史成功", body = NewApiResponse<Vec<StatsSnapshot>>),
        (status = 400, description = "获取策略统计历史失败", body = NewApiResponse<Vec<StatsSnapshot>>)
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

    let play_index = params.play_index;
    let result: Result<Vec<StatsSnapshot>, BacktestEngineError> = engine_guard
        .with_ctx_read_async(|ctx| {
            Box::pin(async move {
                ctx.with_strategy_ctx_read_async(strategy_id, move |ctx| {
                    Box::pin(async move { ctx.get_stats_history(play_index).await })
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
    tag = "回测策略",
    summary = "获取历史虚拟持仓",
    params(
        ("strategy_id" = i32, Path, description = "要获取历史虚拟持仓的策略ID")
    ),
    responses(
        (status = 200, description = "获取历史虚拟持仓成功", body = NewApiResponse<Vec<VirtualPosition>>),
        (status = 400, description = "获取历史虚拟持仓失败", body = NewApiResponse<Vec<VirtualPosition>>)
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
    path = "/api/v1/strategy/backtest/{strategy_id}/status",
    tag = "Backtest Strategy",
    summary = "Get strategy status",
    params(
        ("strategy_id" = i32, Path, description = "The ID of the strategy to get status")
    ),
    responses(
        (status = 200, description = "Get strategy status successfully", body = NewApiResponse<String>),
        (status = 400, description = "Get strategy status failed", body = NewApiResponse<String>)
    )
)]
pub async fn get_strategy_status(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<NewApiResponse<String>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<String, BacktestEngineError> = engine_guard
        .with_ctx_read_async(|ctx| Box::pin(async move { ctx.get_strategy_status(strategy_id).await }))
        .await;

    match result {
        Ok(status) => (StatusCode::OK, Json(NewApiResponse::success(status))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(NewApiResponse::error(e))),
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
        (status = 200, description = "Get running log successfully", body = NewApiResponse<Vec<StrategyRunningLogEvent>>),
        (status = 400, description = "Get running log failed", body = NewApiResponse<Vec<StrategyRunningLogEvent>>)
    )
)]
pub async fn get_running_log(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<NewApiResponse<Vec<StrategyRunningLogEvent>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<Vec<StrategyRunningLogEvent>, BacktestEngineError> = engine_guard
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
        "play_index": 1,
        "key": ""
    })
)]
pub struct GetStrategyDataQuery {
    pub play_index: i32,
    pub key: String,
    pub limit: Option<i32>,
}

#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/data",
    tag = "Backtest Strategy",
    summary = "Get strategy data",
    params(
        ("strategy_id" = i32, Path, description = "The ID of the strategy to get strategy data"),
        ("play_index" = i32, Query, description = "The play index to get strategy data"),
        ("key" = String, Query, description = "The key to get strategy data")
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

    let play_index = params.play_index;
    let limit = params.limit;
    let result: Result<Vec<serde_json::Value>, BacktestEngineError> = engine_guard
        .with_ctx_read_async(|ctx| {
            Box::pin(async move {
                let data = ctx
                    .with_strategy_ctx_read_async(strategy_id, move |ctx| {
                        Box::pin(async move { ctx.get_strategy_data(play_index, key, limit).await })
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

#[derive(Serialize, Deserialize, IntoParams, ToSchema, Debug)]
#[schema(
    title = "get strategy data by datetime",
    description = "get strategy data by datetime",
    example = json!({
        "key": "",
        "datetime": "2024-01-01T00:00:00.000Z",
        "limit": 100
    })
)]
pub struct GetStrategyDataByDatetimeQuery {
    pub key: String,
    pub datetime: String,
    pub limit: Option<i32>,
}

#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/data-by-datetime",
    tag = "Backtest Strategy",
    summary = "Get strategy data by datetime",
    params(
        ("strategy_id" = i32, Path, description = "The ID of the strategy to get strategy data by datetime"),
        ("key" = String, Query, description = "The key to get strategy data by datetime"),
        ("datetime" = String, Query, description = "The datetime to get strategy data by datetime"),
        ("limit" = Option<i32>, Query, description = "The limit to get strategy data by datetime")
    ),
    responses(
        (status = 200, description = "Get strategy data by datetime successfully", body = NewApiResponse<Vec<utoipa::openapi::Object>>),
        (status = 400, description = "Get strategy data by datetime failed", body = NewApiResponse<Vec<utoipa::openapi::Object>>)
    )
)]
#[axum::debug_handler]
pub async fn get_strategy_data_by_datetime(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
    Query(params): Query<GetStrategyDataByDatetimeQuery>,
) -> (StatusCode, Json<NewApiResponse<Vec<serde_json::Value>>>) {
    let key = match Key::from_str(&params.key) {
        Ok(key) => key,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(NewApiResponse::error(e))),
    };

    let datetime = match NaiveDateTime::parse_from_str(&params.datetime, "%Y-%m-%dT%H:%M:%S%.fZ") {
        Ok(dt) => dt.and_utc(),
        Err(e) => {
            return {
                let error = ParseDataTimeFailedSnafu { datetime: params.datetime }.into_error(e);
                (error.http_status_code(), Json(NewApiResponse::error(error)))
            };
        }
    };

    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard: tokio::sync::MutexGuard<'_, backtest_engine::BacktestEngine> = engine.lock().await;

    let limit = params.limit;
    let result: Result<Vec<serde_json::Value>, BacktestEngineError> = engine_guard
        .with_ctx_read_async(|ctx| {
            Box::pin(async move {
                let data = ctx
                    .with_strategy_ctx_read_async(strategy_id, move |ctx| {
                        Box::pin(async move { ctx.get_strategy_data_by_datetime(key, datetime, limit).await })
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
    tag = "策略管理",
    summary = "获取策略缓存键",
    params(
        ("strategy_id" = i32, Path, description = "要获取缓存键的策略ID"),
    ),
    responses(
        (status = 200, description = "获取策略缓存键成功", content_type = "application/json")
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
        Err(e) => {
            
            (e.http_status_code(), Json(NewApiResponse::error(e)))
        }
    }
}
