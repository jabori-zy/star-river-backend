use crate::api::response::NewApiResponse;
use crate::star_river::StarRiver;
use axum::extract::State;
use axum::extract::{Json, Path};
use axum::http::StatusCode;
use engine_core::EngineContextAccessor;
use backtest_engine::engine_error::BacktestEngineError;

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
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<(), BacktestEngineError> = engine_guard.with_ctx_write_async(|ctx| {
        Box::pin(async move {
            ctx.with_strategy_ctx_write_async(strategy_id, move |ctx| {
                Box::pin(async move {
                    ctx.play().await
                })
            }).await?.map_err(BacktestEngineError::from)?;
            Ok(())
        })
    }).await;

    match result {
        Ok(()) => (StatusCode::OK, Json(NewApiResponse::success(()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(NewApiResponse::error(e)))
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
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<i32, BacktestEngineError> = engine_guard.with_ctx_write_async(|ctx| {
        Box::pin(async move {
            let play_index = ctx.with_strategy_ctx_write_async(strategy_id, move |ctx| {
                Box::pin(async move {
                    ctx.play_one().await
                })
            }).await?.map_err(BacktestEngineError::from)?;
            Ok(play_index)
        })
    }).await;

    match result {
        Ok(played_signal_count) => (
            StatusCode::OK,
            Json(NewApiResponse::success(serde_json::json!({
                "played_signal_count": played_signal_count
            }))),
        ),
        Err(e) => (StatusCode::BAD_REQUEST, Json(NewApiResponse::error(e)))
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
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<(), BacktestEngineError> = engine_guard.with_ctx_write_async(|ctx| {
        Box::pin(async move {
            ctx.with_strategy_ctx_write_async(strategy_id, move |ctx| {
                Box::pin(async move {
                    ctx.pause().await
                })
            }).await?.map_err(BacktestEngineError::from)?;
            Ok(())
        })
    }).await;

    match result {
        Ok(()) => (StatusCode::OK, Json(NewApiResponse::success(()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(NewApiResponse::error(e)))
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
) -> (StatusCode, Json<NewApiResponse<serde_json::Value>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<i32, BacktestEngineError> = engine_guard.with_ctx_read_async(|ctx| {
        Box::pin(async move {
            let play_index = ctx.with_strategy_ctx_read_async(strategy_id, move |ctx| {
                Box::pin(async move {
                    ctx.play_index().await
                })
            }).await?;
            Ok(play_index)
        })
    }).await;

    match result {
        Ok(play_index) => (
            StatusCode::OK,
            Json(NewApiResponse::success(serde_json::json!({
                "play_index": play_index
            }))),
        ),
        Err(e) => (StatusCode::NOT_FOUND, Json(NewApiResponse::error(e)))
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
    let engine = engine_manager.backtest_engine().await;
    let engine_guard = engine.lock().await;

    let result: Result<(), BacktestEngineError> = engine_guard.with_ctx_write_async(|ctx| {
        Box::pin(async move {
            ctx.with_strategy_ctx_write_async(strategy_id, move |ctx| {
                Box::pin(async move {
                    ctx.reset().await
                })
            }).await?.map_err(BacktestEngineError::from)?;
            Ok(())
        })
    }).await;

    match result {
        Ok(()) => (StatusCode::OK, Json(NewApiResponse::success(()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(NewApiResponse::error(e)))
    }
}
