use crate::StarRiver;
use crate::api::response::NewApiResponse;
use axum::extract::{Path, State, Query};
use axum::{Json, http::StatusCode};
use engine::market_engine::MarketEngine;
use star_river_core::custom_type::AccountId;
use star_river_core::engine::EngineName;
use star_river_core::error::engine_error::{ExchangeEngineError, MarketEngineError};
use star_river_core::market::KlineInterval;
use star_river_core::market::Symbol;
use serde::Deserialize;



#[utoipa::path(
    get,
    path = "/api/v1/market/symbol_list/{account_id}",
    tag = "市场",
    summary = "获取代币列表",
    params(
        ("account_id" = i32, Path, description = "account id")
    ),
    responses(
        (status = 200, description = "success", body = NewApiResponse<Vec<Symbol>>),
        (status = 500, description = "internal server error")
    )
)]
pub async fn get_symbol_list(
    State(star_river): State<StarRiver>,
    Path(account_id): Path<AccountId>,
) -> (StatusCode, Json<NewApiResponse<Vec<Symbol>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::MarketEngine).await;
    let mut engine_guard = engine.lock().await;
    let market_engine = engine_guard.as_any_mut().downcast_mut::<MarketEngine>().unwrap();
    let symbol_list = market_engine.get_symbol_list(account_id).await;
    match symbol_list {
        Ok(symbol_list) => (
            StatusCode::OK,
            Json(NewApiResponse::success(symbol_list)),
        ),
        Err(e) => {
            tracing::error!("get symbol list error: {}", e);
            match e {
                MarketEngineError::ExchangeEngine { ref source, .. } => {
                    match source {
                        ExchangeEngineError::ExchangeClientNotRegistered { .. } => {
                            (
                                StatusCode::NOT_FOUND,
                                Json(NewApiResponse::error(e)),
                            )
                        }
                        _ => {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(NewApiResponse::error(e)),
                            )
                        }
                    }
                }
                _ => {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(NewApiResponse::error(e)),
                    )
                }
            }
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
        (status = 200, description = "success", body = NewApiResponse<Vec<KlineInterval>>),
        (status = 500, description = "internal server error")
    )
)]
pub async fn get_support_kline_intervals(
    State(star_river): State<StarRiver>,
    Path(account_id): Path<AccountId>,
) -> (StatusCode, Json<NewApiResponse<Vec<KlineInterval>>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::MarketEngine).await;
    let mut engine_guard = engine.lock().await;
    let market_engine = engine_guard.as_any_mut().downcast_mut::<MarketEngine>().unwrap();
    let support_kline_intervals = market_engine.get_support_kline_intervals(account_id).await;
    match support_kline_intervals {
        Ok(support_kline_intervals) => (
            StatusCode::OK,
            Json(NewApiResponse::success(support_kline_intervals)),
        ),
        Err(e) => {
            tracing::error!("get support kline intervals error: {}", e);
            match e {
                MarketEngineError::ExchangeEngine { ref source, .. } => {
                    match source {
                        ExchangeEngineError::ExchangeClientNotRegistered { .. } => {
                            (
                                StatusCode::NOT_FOUND,
                                Json(NewApiResponse::error(e)),
                            )
                        }
                        _ => {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(NewApiResponse::error(e)),
                            )
                        }
                    }
                }
                _ => {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(NewApiResponse::error(e)),
                    )
                }
            }
        }
    }
}



#[derive(Deserialize)]
pub struct SymbolQuery {
    pub symbol: String,
}


// 获取单个symbol
#[utoipa::path(
    get,
    path = "/api/v1/market/symbol/{account_id}",
    tag = "市场",
    summary = "获取单个交易对信息",
    params(
        ("account_id" = i32, Path, description = "account id"),
        ("symbol" = String, Query, description = "symbol")
    ),
    responses(
        (status = 200, description = "success", body = NewApiResponse<Symbol>),
        (status = 500, description = "internal server error")
    )
)]
pub async fn get_symbol(
    State(star_river): State<StarRiver>,
    Path(account_id): Path<AccountId>,
    Query(query): Query<SymbolQuery>,
) -> (StatusCode, Json<NewApiResponse<Symbol>>) {
    let engine_manager = star_river.engine_manager.lock().await;
    let engine = engine_manager.get_engine(EngineName::MarketEngine).await;
    let mut engine_guard = engine.lock().await;
    let market_engine = engine_guard.as_any_mut().downcast_mut::<MarketEngine>().unwrap();
    let symbol = market_engine.get_symbol(account_id, query.symbol).await;
    match symbol {
        Ok(symbol) => (
            StatusCode::OK,
            Json(NewApiResponse::success(symbol)),
        ),
        Err(e) => {
            tracing::error!("get symbol error: {}", e);
            match e {
                MarketEngineError::ExchangeEngine { ref source, .. } => {
                    match source {
                        ExchangeEngineError::ExchangeClientNotRegistered { .. } => {
                            (
                                StatusCode::NOT_FOUND,
                                Json(NewApiResponse::error(e)),
                            )
                        }
                        _ => {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(NewApiResponse::error(e)),
                            )
                        }
                    }
                }
                _ => {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(NewApiResponse::error(e)),
                    )
                }
            }
        }
    }
}
