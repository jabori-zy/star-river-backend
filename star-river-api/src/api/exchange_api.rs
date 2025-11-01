use super::{StarRiver, EngineName};
use axum::extract::{Path, State};
use axum::{Json, http::StatusCode};
use star_river_core::custom_type::AccountId;
use star_river_core::market::ExchangeStatus;
use crate::api::response::NewApiResponse;
use engine::exchange_engine::ExchangeEngine;







#[utoipa::path(
    get,
    path = "/api/v1/exchange/status/{account_id}",
    tag = "Exchange Management",
    summary = "get exchange status",
    description = "get exchange status by account id",
    params(
        ("account_id" = AccountId, Path, description = "account id")
    ),
    responses(
        (status = 200, description = "get exchange status success", content_type = "application/json"),
        (status = 400, description = "get exchange status error", content_type = "application/json")
    )
)]
pub async fn get_exchange_status(
    State(star_river): State<StarRiver>,
    Path(account_id): Path<AccountId>,
) -> (StatusCode, Json<NewApiResponse<ExchangeStatus>>) {
    let exchange_manager = star_river.engine_manager.lock().await;
    let engine = exchange_manager.get_engine(EngineName::ExchangeEngine).await;
    let mut engine_guard = engine.lock().await;
    let exchange_engine = engine_guard.as_any_mut().downcast_mut::<ExchangeEngine>().unwrap();
    let exchange_status = exchange_engine.get_exchange_status(account_id).await;
    (StatusCode::OK, Json(NewApiResponse::success(exchange_status)))
}




#[utoipa::path(
    post,
    path = "/api/v1/exchange/connect/{account_id}",
    tag = "Exchange Management",
    summary = "connect to a exchange",
    description = "connect to a exchange by account id",
    params(
        ("account_id" = AccountId, Path, description = "account id")
    ),
    responses(
        (status = 200, description = "connect to a exchange success", content_type = "application/json"),
        (status = 500, description = "connect to a exchange error", content_type = "application/json")
    )
)]
// connect to a exchange
pub async fn connect_exchange(
    State(star_river): State<StarRiver>,
    Path(account_id): Path<AccountId>,
) -> (StatusCode, Json<NewApiResponse<()>>) {
    let exchange_manager = star_river.engine_manager.lock().await;
    let engine = exchange_manager.get_engine(EngineName::ExchangeEngine).await;
    let mut engine_guard = engine.lock().await;
    let exchange_engine = engine_guard.as_any_mut().downcast_mut::<ExchangeEngine>().unwrap();
    let result = exchange_engine.register_exchange(account_id).await;
    if result.is_ok() {
        (StatusCode::OK, Json(NewApiResponse::success(())))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(NewApiResponse::error(result.unwrap_err())))
    }

}