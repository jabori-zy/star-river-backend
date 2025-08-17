use crate::star_river::StarRiver;
use axum::http::StatusCode;
use axum::extract::State;
use serde::{Serialize, Deserialize};
use axum::extract::{Json,Path,Query};
use crate::api::response::ApiResponse;
use utoipa::{IntoParams, ToSchema};
use types::strategy::Strategy;
use database::mutation::strategy_config_mutation::StrategyConfigMutation;
use database::query::strategy_config_query::StrategyConfigQuery;
use tracing::instrument;
use types::engine::EngineName;
use engine::strategy_engine::StrategyEngine;
use types::order::virtual_order::VirtualOrder;
use types::position::virtual_position::VirtualPosition;
use types::strategy_stats::StatsSnapshot;

#[derive(Debug, Serialize, Deserialize, IntoParams, ToSchema)]
#[schema(
    title = "更新回测图表配置参数",
    description = "更新回测图表配置参数",
    example = json!({
        "charts": [
            {
                "id": "1",
                "chartName": "K线图",
                "klineCacheKeyStr": "",
                "indicatorCacheKeyStr": [],
            }
        ],
        "layout": "vertical"
    })
)]
pub struct UpdateBacktestChartConfigParams {
    pub backtest_chart_config: Option<serde_json::Value>,
}


#[utoipa::path(
    post,
    path = "/api/v1/strategy/backtest/{strategy_id}/chart_config",
    tag = "回测策略",
    summary = "更新回测图表配置",
    params(UpdateBacktestChartConfigParams),
    request_body = UpdateBacktestChartConfigParams,
    responses(
        (status = 200, description = "成功", body = ApiResponse<Strategy>),
        (status = 500, description = "内部服务器错误", body = ApiResponse<Strategy>),
    )
)]
#[instrument(skip(star_river))]
pub async fn update_backtest_chart_config(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
    Json(params): Json<UpdateBacktestChartConfigParams>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigMutation::update_backtest_chart_config(conn, strategy_id, params.backtest_chart_config).await {
        Ok(backtest_chart_config) => {
            tracing::info!(strategy_id = strategy_id, "update backtest chart config success");
            (StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "success".to_string(),
                data: Some(backtest_chart_config),
            }),
        )
        },
        Err(e) => {
            tracing::error!(strategy_id = strategy_id, "update backtest chart config error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                code: 1,
                message: e.to_string(),
                data: None,
            }))
        }
    }
}


#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/chart_config",
    tag = "回测策略",
    summary = "获取回测图表配置",
    responses(
        (status = 200, description = "成功", body = ApiResponse<serde_json::Value>),
        (status = 500, description = "内部服务器错误", body = ApiResponse<serde_json::Value>),
    )
)]

#[instrument(skip(star_river))]
pub async fn get_backtest_chart_config(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigQuery::get_backtest_chart_config_by_strategy_id(conn, strategy_id).await {
        Ok(backtest_chart_config) => {
            tracing::info!(strategy_id = strategy_id, "get backtest chart config success");
            (StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "success".to_string(),
                data: Some(backtest_chart_config),
            }))
        },
        Err(e) => {
            tracing::error!(strategy_id = strategy_id, "get backtest chart config error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                code: 1,
                message: e.to_string(),
                data: None,
            }))
        }
    }
}


#[utoipa::path(
    post,
    path = "/api/v1/strategy/backtest/{strategy_id}/play",
    tag = "回测策略",
    summary = "播放k线",
    params(
        ("strategy_id" = i32, Path, description = "要播放的策略ID")
    ),
    responses(
        (status = 200, description = "播放策略成功")
    )
)]
pub async fn play(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<()>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    strategy_engine.play(strategy_id).await.unwrap();
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))
}



#[utoipa::path(
    post,
    path = "/api/v1/strategy/backtest/{strategy_id}/play-one",
    tag = "回测策略",
    summary = "播放单个K线",
    params(
        ("strategy_id" = i32, Path, description = "要播放单个K线的策略ID")
    ),
    responses(
        (status = 200, description = "播放单个K线成功"),
        (status = 400, description = "播放单个K线失败")
    )
)]
pub async fn play_one(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    let played_signal_count = strategy_engine.play_one_kline(strategy_id).await;
    if let Ok(played_signal_count) = played_signal_count {
        (StatusCode::OK, Json(ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(serde_json::json!({
                "played_signal_count": played_signal_count
            })),
        }))
    } else {
        (StatusCode::BAD_REQUEST, Json(ApiResponse {
            code: -1,
            message: "failed".to_string(),
            data: None,
        }))
    }
}



#[utoipa::path(
    post,
    path = "/api/v1/strategy/backtest/{strategy_id}/pause",
    tag = "回测策略",
    summary = "暂停播放k线",
    params(
        ("strategy_id" = i32, Path, description = "要暂停的策略ID")
    ),
    responses(
        (status = 200, description = "暂停策略成功"),
        (status = 400, description = "暂停策略失败")
    )
)]
pub async fn pause(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<()>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    strategy_engine.pause(strategy_id).await.unwrap();
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))
}


#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest/{strategy_id}/play-index",
    tag = "回测策略",
    summary = "获取播放索引",
    params(
        ("strategy_id" = i32, Path, description = "要获取播放索引的策略ID")
    ),
    responses(
        (status = 200, description = "获取播放索引成功"),
        (status = 400, description = "获取播放索引失败")
    )
)]

pub async fn get_play_index(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    let play_index = strategy_engine.get_play_index(strategy_id).await;
    if let Ok(play_index) = play_index {
        (StatusCode::OK, Json(ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(serde_json::json!({
                "play_index": play_index
            })),
        }))
    } else {
        (StatusCode::BAD_REQUEST, Json(ApiResponse {
            code: -1,
            message: "failed".to_string(),
            data: None,
        }))
    }
}


#[utoipa::path(
    post,
    path = "/api/v1/strategy/backtest/{strategy_id}/reset",
    tag = "回测策略",
    summary = "重置播放",
    params(
        ("strategy_id" = i32, Path, description = "要重置的策略ID")
    ),
    responses(
        (status = 200, description = "重置策略成功"),
        (status = 400, description = "重置策略失败")
    )
)]
pub async fn reset(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<()>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    strategy_engine.reset(strategy_id).await.unwrap();
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))
}



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

pub async fn get_virtual_orders(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<Vec<VirtualOrder>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    let virtual_orders = strategy_engine.get_virtual_orders(strategy_id).await;
    if let Ok(virtual_orders) = virtual_orders {
        (StatusCode::OK, Json(ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(virtual_orders),
        }))
    } else {
        (StatusCode::BAD_REQUEST, Json(ApiResponse {
            code: -1,
            message: "failed".to_string(),
            data: None,
        }))
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


pub async fn get_current_positions(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponse<Vec<VirtualPosition>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    let current_positions = strategy_engine.get_current_virtual_positions(strategy_id).await;
    if let Ok(current_positions) = current_positions {
        (StatusCode::OK, Json(ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(current_positions),
        }))
    } else {
        (StatusCode::BAD_REQUEST, Json(ApiResponse {
            code: -1,
            message: "failed".to_string(),
            data: None,
        }))
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
pub async fn get_stats_history(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>, Query(params): Query<GetStatsHistoryQuery>) -> (StatusCode, Json<ApiResponse<Vec<StatsSnapshot>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<StrategyEngine>().unwrap();
    let stats_history = strategy_engine.get_stats_history(strategy_id, params.play_index).await;
    if let Ok(stats_history) = stats_history {
        (StatusCode::OK, Json(ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(stats_history),
        }))
    } else {
        (StatusCode::BAD_REQUEST, Json(ApiResponse {
            code: -1,
            message: "failed".to_string(),
            data: None,
        }))
    }
}
