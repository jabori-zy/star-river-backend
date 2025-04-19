use serde::{Serialize, Deserialize};
use axum::extract::State;
use axum::http::StatusCode;
use axum::extract::Query;
use axum::response::Json;
use crate::star_river::StarRiver;
use crate::api::response::ApiResponse;
use engine::EngineName;
use engine::account_engine::AccountEngine;


#[derive(Serialize, Deserialize)]
pub struct LoginMt5AccountParams {
    pub account_id: i32,
}


#[axum::debug_handler]
pub async fn login_mt5_account(
    State(star_river): State<StarRiver>,
    Json(params): Json<LoginMt5AccountParams>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let account_id = params.account_id;

    let engine_manager = star_river.engine_manager.lock().await;
    // 获取account_engine
    let engine = engine_manager.get_engine(EngineName::AccountEngine).await;
    let mut engine_guard = engine.lock().await;
    let account_engine = engine_guard.as_any_mut().downcast_mut::<AccountEngine>().unwrap();
    account_engine.register_mt5_exchange(account_id).await.unwrap();

    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))

}