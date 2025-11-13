use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use engine_core::EngineContextAccessor;
use exchange_core::ExchangeRunState;
use star_river_core::{custom_type::AccountId, error::StarRiverErrorTrait};

use crate::{StarRiver, api::response::NewApiResponse};

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
) -> (StatusCode, Json<NewApiResponse<ExchangeRunState>>) {
    let exchange_manager = star_river.engine_manager.lock().await;
    let engine = exchange_manager.exchange_engine().await;
    let engine_guard = engine.lock().await;
    let exchange_status = engine_guard
        .with_ctx_read_async(|ctx| Box::pin(async move { ctx.exchange_status(&account_id).await }))
        .await;
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
    let engine = exchange_manager.exchange_engine().await;
    let engine_guard = engine.lock().await;
    let result = engine_guard
        .with_ctx_write_async(|ctx| Box::pin(async move { ctx.register_exchange(account_id).await }))
        .await;
    match result {
        Ok(()) => (StatusCode::OK, Json(NewApiResponse::success(()))),
        Err(e) => (e.http_status_code(), Json(NewApiResponse::error(e))),
    }
}
