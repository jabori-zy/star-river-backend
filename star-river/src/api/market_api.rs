use crate::api::response::ApiResponse;
use crate::StarRiver;
use axum::extract::{Path, State};
use axum::{http::StatusCode, Json};
use engine::market_engine::MarketEngine;
use types::custom_type::AccountId;
use types::engine::EngineName;
use types::market::KlineInterval;
use types::market::Symbol;

#[utoipa::path(
    get,
    path = "/api/v1/market/symbol_list/{account_id}",
    tag = "市场",
    summary = "获取市场符号列表",
    params(
        ("account_id" = i32, Path, description = "account id")
    ),
    responses(
        (status = 200, description = "success", body = ApiResponse<Vec<Symbol>>),
        (status = 500, description = "internal server error")
    )
)]
pub async fn get_symbol_list(
    State(star_river): State<StarRiver>,
    Path(account_id): Path<AccountId>,
) -> (StatusCode, Json<ApiResponse<Vec<Symbol>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::MarketEngine).await;
    let mut engine_guard = engine.lock().await;
    let market_engine = engine_guard
        .as_any_mut()
        .downcast_mut::<MarketEngine>()
        .unwrap();
    let symbol_list = market_engine.get_symbol_list(account_id).await;
    match symbol_list {
        Ok(symbol_list) => (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "success".to_string(),
                data: Some(symbol_list),
            }),
        ),
        Err(e) => {
            tracing::error!("get symbol list error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    code: 1,
                    message: e.to_string(),
                    data: None,
                }),
            )
        }
    }
}

#[utoipa::path(

    get,
    path = "/api/v1/market/support_kline_intervals/{account_id}",
    tag = "市场",
    summary = "获取市场支持的k线间隔",
    params(
        ("account_id" = i32, Path, description = "account id")
    ),
    responses(
        (status = 200, description = "success", body = ApiResponse<Vec<KlineInterval>>),
        (status = 500, description = "internal server error")
    )
)]
pub async fn get_support_kline_intervals(
    State(star_river): State<StarRiver>,
    Path(account_id): Path<AccountId>,
) -> (StatusCode, Json<ApiResponse<Vec<KlineInterval>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::MarketEngine).await;
    let mut engine_guard = engine.lock().await;
    let market_engine = engine_guard
        .as_any_mut()
        .downcast_mut::<MarketEngine>()
        .unwrap();
    let support_kline_intervals = market_engine.get_support_kline_intervals(account_id).await;
    (
        StatusCode::OK,
        Json(ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(support_kline_intervals),
        }),
    )
}
