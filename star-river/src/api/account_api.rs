use axum::extract::{Json, Query, State};

use crate::StarRiver;
use axum::http::StatusCode;
use serde::{Serialize, Deserialize};
use crate::api::response::ApiResponse;
use engine::EngineName;
use engine::account_engine::account_engine_types::AccountConfig;
use engine::account_engine::AccountEngine;
use engine::account_engine::account_engine_types::MetaTrader5AccountConfig;


#[derive(Serialize, Deserialize)]
pub struct AddAccountConfigRequest {
    pub account_name: String,
    pub exchange: String,
    pub account_config: serde_json::Value,
}




pub async fn add_account_config(
    State(star_river): State<StarRiver>,
    Json(request): Json<AddAccountConfigRequest>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let account_engine = star_river.engine_manager.lock().await.get_engine(EngineName::AccountEngine).await;
    let mut account_engine_guard = account_engine.lock().await;
    let account_engine = account_engine_guard.as_any_mut().downcast_mut::<AccountEngine>().unwrap();

    let account_config = match request.exchange.as_str() {
        "MetaTrader5" => {
            let config= serde_json::from_value::<MetaTrader5AccountConfig>(request.account_config).unwrap();
            AccountConfig::MetaTrader5(config)
        },
        _ => return (StatusCode::BAD_REQUEST, Json(ApiResponse {
            code: 1,
            message: "invalid exchange".to_string(),
            data: None,
        })),
    };
    account_engine.add_account_config(request.account_name, account_config).await.unwrap();
    (StatusCode::OK, Json(ApiResponse {
        code: 0,
        message: "success".to_string(),
        data: None,
    }))
}
