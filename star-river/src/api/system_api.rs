use crate::api::response::ApiResponse;
use crate::api::response::NewApiResponse;
use crate::star_river::StarRiver;
use axum::extract::Json;
use axum::extract::State;
use axum::http::StatusCode;
use database::mutation::system_config_mutation::SystemConfigMutation;
use database::query::system_config_query::SystemConfigQuery;
use serde::{Deserialize, Serialize};
use snafu::IntoError;
use star_river_core::error::star_river_error::*;
use star_river_core::system::system_config::Localization;
use star_river_core::system::system_config::SystemConfig;
use star_river_core::system::system_config::SystemConfigManager;
use tracing::instrument;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemConfigUpdateParams {
    /// 本地化
    pub localization: Localization,
    /// 时区
    pub timezone: String,
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
#[instrument(skip(star_river, system_config_params), fields(localization = ?system_config_params.localization, timezone = %system_config_params.timezone))]
pub async fn update_system_config(
    State(star_river): State<StarRiver>,
    Json(system_config_params): Json<SystemConfigUpdateParams>,
) -> (StatusCode, Json<NewApiResponse<SystemConfig>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;

    let update_result = SystemConfigMutation::update_system_config(
        conn,
        system_config_params.localization,
        system_config_params.timezone,
    )
    .await;

    match update_result {
        Ok(system_config) => {
            // 更新系统配置
            SystemConfigManager::update_config(system_config.clone());
            let global_timezone = SystemConfigManager::get_timezone();
            let global_localization = SystemConfigManager::get_localization();
            if global_timezone == system_config.timezone || global_localization == system_config.localization {
                tracing::info!(
                    "update system config success. timezone: {:?}, localization: {:?}",
                    global_timezone,
                    global_localization
                );
            } else {
                tracing::error!(
                    "update system config failed. timezone: {:?}, localization: {:?}",
                    global_timezone,
                    global_localization
                );
            }

            (StatusCode::OK, Json(NewApiResponse::success(system_config)))
        }
        Err(e) => {
            let error = UpdateSystemConfigFailedSnafu {}.into_error(e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(NewApiResponse::error(error)))
        }
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
    match system_config {
        Ok(system_config) => (StatusCode::OK, Json(NewApiResponse::success(system_config))),
        Err(e) => {
            let error = GetSystemConfigFailedSnafu {}.into_error(e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(NewApiResponse::error(error)))
        }
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
