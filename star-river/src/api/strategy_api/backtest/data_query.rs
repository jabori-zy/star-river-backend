use crate::api::response::ApiResponse;
use crate::api::response::NewApiResponse;
use crate::star_river::StarRiver;
use axum::extract::State;
use axum::extract::{Json, Path, Query};
use axum::http::StatusCode;
use engine::strategy_engine::StrategyEngine;
use event_center::event::strategy_event::StrategyRunningLogEvent;
use serde::{Deserialize, Serialize};
use star_river_core::cache::Key;
use star_river_core::engine::EngineName;
use star_river_core::order::virtual_order::VirtualOrder;
use star_river_core::position::virtual_position::VirtualPosition;
use star_river_core::strategy_stats::StatsSnapshot;
use star_river_core::transaction::virtual_transaction::VirtualTransaction;
use utoipa::{IntoParams, ToSchema};
use std::str::FromStr;
use star_river_core::cache::CacheItem;

#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/virtual-orders",
    tag = "回测策略",
    summary = "获取虚拟订单",
    params(
        ("strategy_id" = i32, Path, description = "要获取虚拟订单的策略ID")
    ),
    responses(
        (status = 200, description = "获取虚拟订单成功", body = ApiResponse<Vec<VirtualOrder>>),
        (status = 400, description = "获取虚拟订单失败", body = ApiResponse<Vec<VirtualOrder>>)
    ))]
pub async fn get_virtual_orders(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<ApiResponse<Vec<VirtualOrder>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard
        .as_any_mut()
        .downcast_mut::<StrategyEngine>()
        .unwrap();
    let virtual_orders = strategy_engine.get_virtual_orders(strategy_id).await;
    if let Ok(virtual_orders) = virtual_orders {
        (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "success".to_string(),
                data: Some(virtual_orders),
            }),
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: "failed".to_string(),
                data: None,
            }),
        )
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
        (status = 200, description = "获取当前虚拟持仓成功", body = ApiResponse<Vec<VirtualPosition>>),
        (status = 400, description = "获取当前虚拟持仓失败", body = ApiResponse<Vec<VirtualPosition>>)
    )
)]
pub async fn get_current_positions(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<ApiResponse<Vec<VirtualPosition>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard
        .as_any_mut()
        .downcast_mut::<StrategyEngine>()
        .unwrap();
    let current_positions = strategy_engine
        .get_current_virtual_positions(strategy_id)
        .await;
    if let Ok(current_positions) = current_positions {
        (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "success".to_string(),
                data: Some(current_positions),
            }),
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: "failed".to_string(),
                data: None,
            }),
        )
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
        (status = 200, description = "获取虚拟交易明细成功", body = ApiResponse<Vec<VirtualTransaction>>),
        (status = 400, description = "获取虚拟交易明细失败", body = ApiResponse<Vec<VirtualTransaction>>)
    )
)]
pub async fn get_virtual_transactions(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<ApiResponse<Vec<VirtualTransaction>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard
        .as_any_mut()
        .downcast_mut::<StrategyEngine>()
        .unwrap();
    let virtual_transactions = strategy_engine.get_virtual_transactions(strategy_id).await;
    if let Ok(virtual_transactions) = virtual_transactions {
        (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "success".to_string(),
                data: Some(virtual_transactions),
            }),
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: "failed".to_string(),
                data: None,
            }),
        )
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
        (status = 200, description = "获取策略统计历史成功", body = ApiResponse<Vec<StatsSnapshot>>),
        (status = 400, description = "获取策略统计历史失败", body = ApiResponse<Vec<StatsSnapshot>>)
    )
)]
#[axum::debug_handler]
pub async fn get_stats_history(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
    Query(params): Query<GetStatsHistoryQuery>,
) -> (StatusCode, Json<ApiResponse<Vec<StatsSnapshot>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard
        .as_any_mut()
        .downcast_mut::<StrategyEngine>()
        .unwrap();
    let stats_history = strategy_engine
        .get_stats_history(strategy_id, params.play_index)
        .await;
    if let Ok(stats_history) = stats_history {
        (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "success".to_string(),
                data: Some(stats_history),
            }),
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: "failed".to_string(),
                data: None,
            }),
        )
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
        (status = 200, description = "获取历史虚拟持仓成功", body = ApiResponse<Vec<VirtualPosition>>),
        (status = 400, description = "获取历史虚拟持仓失败", body = ApiResponse<Vec<VirtualPosition>>)
    )
)]
pub async fn get_history_positions(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<ApiResponse<Vec<VirtualPosition>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard
        .as_any_mut()
        .downcast_mut::<StrategyEngine>()
        .unwrap();
    let history_positions = strategy_engine
        .get_history_virtual_positions(strategy_id)
        .await;
    if let Ok(history_positions) = history_positions {
        (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "success".to_string(),
                data: Some(history_positions),
            }),
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: "failed".to_string(),
                data: None,
            }),
        )
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
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard
        .as_any_mut()
        .downcast_mut::<StrategyEngine>()
        .unwrap();
    let strategy_status = strategy_engine.get_strategy_status(strategy_id).await;
    if let Ok(strategy_status) = strategy_status {
        (
            StatusCode::OK,
            Json(NewApiResponse::success(strategy_status)),
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(NewApiResponse::error(strategy_status.unwrap_err())),
        )
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
) -> (
    StatusCode,
    Json<NewApiResponse<Vec<StrategyRunningLogEvent>>>,
) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard
        .as_any_mut()
        .downcast_mut::<StrategyEngine>()
        .unwrap();
    let running_log = strategy_engine.get_running_log(strategy_id).await;
    if let Ok(running_log) = running_log {
        (StatusCode::OK, Json(NewApiResponse::success(running_log)))
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(NewApiResponse::error(running_log.unwrap_err())),
        )
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
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard
        .as_any_mut()
        .downcast_mut::<StrategyEngine>()
        .unwrap();

    match Key::from_str(&params.key) {
        Ok(key) => {
            strategy_engine
                .get_strategy_data(strategy_id, params.play_index, key)
                .await
                .map(|data| {
                    let result: Vec<serde_json::Value> = data
                        .iter()
                        .map(|cache_value| cache_value.to_json())
                        .collect();
                    (StatusCode::OK, Json(NewApiResponse::success(result)))
                })
                .unwrap_or_else(|e| (StatusCode::BAD_REQUEST, Json(NewApiResponse::error(e))))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(NewApiResponse::error(e))),
    }
}
