use database::mutation::system_config_mutation::SystemConfigMutation;
use axum::http::StatusCode;
use axum::extract::State;
use axum::extract::Json;
use crate::star_river::StarRiver;
use crate::api::response::ApiResponse;
use types::system::system_config::SystemConfig;
use database::query::system_config_query::SystemConfigQuery;
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use types::system::system_config::SystemConfigUpdateParams;


#[utoipa::path(
    put,
    path = "/api/v1/system/config",
    tag = "系统配置",
    summary = "更新系统配置",
    request_body = SystemConfigUpdateParams,
    responses(
        (status = 200, description = "更新系统配置成功", content_type = "application/json", body = ApiResponse<SystemConfig>),
    )
)]
pub async fn update_system_config(
    State(star_river): State<StarRiver>,
    Json(system_config_params): Json<SystemConfigUpdateParams>,
) -> (StatusCode, Json<ApiResponse<SystemConfig>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;

    tracing::debug!("更新系统配置: {:?}", system_config_params);
    
    let update_result = SystemConfigMutation::update_system_config(conn, system_config_params).await;

    match update_result {
        Ok(system_config) => {
            (StatusCode::OK, Json(ApiResponse {
                code: 0,
                message: "更新系统配置成功".to_string(),
                data: Some(system_config),
            }))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
                code: -1,
                message: e.to_string(),
                data: None,
            }))
        }
    }
}



#[utoipa::path(
    get,
    path = "/api/v1/system/config",
    tag = "系统配置",
    summary = "获取系统配置",
    responses(
        (status = 200, description = "获取系统配置成功", content_type = "application/json", body = ApiResponse<SystemConfig>),
    )
)]
pub async fn get_system_config(
    State(star_river): State<StarRiver>,
) -> (StatusCode, Json<ApiResponse<SystemConfig>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;

    let system_config = SystemConfigQuery::get_system_config(conn).await;
    match system_config {
        Ok(system_config) => {
            (StatusCode::OK, Json(ApiResponse {
                code: 0,
                message: "获取系统配置成功".to_string(),
                data: Some(system_config),
            }))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
                code: -1,
                message: e.to_string(),
                data: None,
            }))
        }
    }
}