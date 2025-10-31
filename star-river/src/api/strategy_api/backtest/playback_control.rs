use crate::api::response::ApiResponse;
use crate::api::response::NewApiResponse;
use crate::star_river::StarRiver;
use axum::extract::State;
use axum::extract::{Json, Path};
use axum::http::StatusCode;
// use engine::backtest_strategy_engine::BacktestStrategyEngine;
use engine::backtest_engine::BacktestEngine as BacktestStrategyEngine;
use star_river_core::engine::EngineName;

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
pub async fn play(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<NewApiResponse<()>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<BacktestStrategyEngine>().unwrap();
    let result = strategy_engine.play(strategy_id).await;
    if let Ok(()) = result {
        (StatusCode::OK, Json(NewApiResponse::success(())))
    } else {
        (StatusCode::BAD_REQUEST, Json(NewApiResponse::error(result.unwrap_err())))
    }
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
pub async fn play_one(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<NewApiResponse<serde_json::Value>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<BacktestStrategyEngine>().unwrap();
    let played_signal_count = strategy_engine.play_one_kline(strategy_id).await;
    if let Ok(played_signal_count) = played_signal_count {
        (
            StatusCode::OK,
            Json(NewApiResponse::success(serde_json::json!({
                "played_signal_count": played_signal_count
            }))),
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(NewApiResponse::error(played_signal_count.unwrap_err())),
        )
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
pub async fn pause(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<NewApiResponse<()>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<BacktestStrategyEngine>().unwrap();
    let result = strategy_engine.pause(strategy_id).await;
    if let Ok(()) = result {
        (StatusCode::OK, Json(NewApiResponse::success(())))
    } else {
        (StatusCode::BAD_REQUEST, Json(NewApiResponse::error(result.unwrap_err())))
    }
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
pub async fn get_play_index(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<BacktestStrategyEngine>().unwrap();
    let play_index = strategy_engine.get_play_index(strategy_id).await;
    if let Ok(play_index) = play_index {
        (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "success".to_string(),
                data: Some(serde_json::json!({
                    "play_index": play_index
                })),
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
pub async fn reset(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<NewApiResponse<()>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::StrategyEngine).await;
    let mut engine_guard = engine.lock().await;
    let strategy_engine = engine_guard.as_any_mut().downcast_mut::<BacktestStrategyEngine>().unwrap();
    let result = strategy_engine.reset(strategy_id).await;
    if let Ok(()) = result {
        (StatusCode::OK, Json(NewApiResponse::success(())))
    } else {
        (StatusCode::BAD_REQUEST, Json(NewApiResponse::error(result.unwrap_err())))
    }
}
