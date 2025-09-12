use crate::api::response::ApiResponse;
use crate::star_river::StarRiver;
use crate::api::response::NewApiResponse;
use axum::extract::Json;
use axum::extract::State;
use axum::http::StatusCode;
use database::mutation::system_config_mutation::SystemConfigMutation;
use database::query::system_config_query::SystemConfigQuery;
use serde::{Deserialize, Serialize};
use star_river_core::system::system_config::SystemConfig;
use star_river_core::system::system_config::Localization;
use utoipa::ToSchema;
use chrono::Utc;
use star_river_core::error::system_error::*;
use snafu::IntoError;
use tracing::instrument;




#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemConfigUpdateParams {
    /// 本地化
    pub localization: Localization,
    /// 时区
    pub timezone: String
}

#[utoipa::path(
    put,
    path = "/api/v1/system/config",
    tag = "System Config",
    summary = "Update system config",
    request_body = SystemConfigUpdateParams,
    responses(
        (status = 200, description = "Update system config success", content_type = "application/json", body = ApiResponse<SystemConfig>),
    )
)]
#[instrument(skip(star_river))]
pub async fn update_system_config(
    State(star_river): State<StarRiver>,
    Json(system_config_params): Json<SystemConfigUpdateParams>,
) -> (StatusCode, Json<NewApiResponse<SystemConfig>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;

    tracing::info!("update system config. localization: {:?}, timezone: {:?}", system_config_params.localization, system_config_params.timezone);

    let update_result = SystemConfigMutation::update_system_config(conn, system_config_params.localization, system_config_params.timezone).await;

    match update_result {
        Ok(system_config) => (
            StatusCode::OK,
            Json(NewApiResponse::success(system_config)),
        ),
        Err(e) => {
            let error = UpdateSystemConfigFailedSnafu {}.into_error(e);
            (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(NewApiResponse::error(error)),
        )},
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/system/config",
    tag = "System Config",
    summary = "Get system config",
    responses(
        (status = 200, description = "获取系统配置成功", content_type = "application/json", body = ApiResponse<SystemConfig>),
    )
)]
#[instrument(skip(star_river))]
pub async fn get_system_config(
    State(star_river): State<StarRiver>,
) -> (StatusCode, Json<NewApiResponse<SystemConfig>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;

    let system_config = SystemConfigQuery::get_system_config(conn).await;
    tracing::debug!("get system config. system_config: {:?}", system_config);
    match system_config {
        Ok(system_config) => (
            StatusCode::OK,
            Json(NewApiResponse::success(system_config)),
        ),
        Err(e) => {
            let error = GetSystemConfigFailedSnafu {}.into_error(e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(NewApiResponse::error(error)),
            )   
        },
    }
}


#[utoipa::path(
    get,
    path = "/api/v1/system/timezones",
    tag = "System Config",
    summary = "Get timezones",
    responses(
        (status = 200, description = "get timezones success", content_type = "application/json", body = NewApiResponse<Vec<String>>),
    )
)]
#[instrument]
pub async fn get_timezones() -> (StatusCode, Json<NewApiResponse<Vec<String>>>) {

    let timezones = chrono_tz::TZ_VARIANTS.iter().map(|tz| tz.name().to_string()).collect();
    (StatusCode::OK, Json(NewApiResponse::success(timezones)))
}
