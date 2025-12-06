use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use database::{mutation::strategy_config_mutation::StrategyConfigMutation, query::strategy_config_query::StrategyConfigQuery};
use serde::{Deserialize, Serialize};
use strategy_core::strategy::StrategyConfig;
use tracing::instrument;
use utoipa::{IntoParams, ToSchema};

use crate::{api::response::ApiResponse, star_river::StarRiver};

#[derive(Debug, Serialize, Deserialize, IntoParams, ToSchema)]
#[schema(
    title = "Update backtest chart configuration parameters",
    description = "Update backtest chart configuration parameters",
    example = json!({
        "charts": [
            {
                "id": "1",
                "chartName": "Kline Chart",
                "klineCacheKeyStr": "",
                "indicatorCacheKeyStr": [],
            }
        ],
        "layout": "vertical"
    })
)]
pub struct UpdateBacktestChartConfigParams {
    pub backtest_chart_config: serde_json::Value,
}

#[utoipa::path(
    post,
    path = "/api/v1/strategy/backtest/{strategy_id}/chart_config",
    tag = "Backtest Strategy",
    summary = "Update backtest chart configuration",
    params(UpdateBacktestChartConfigParams),
    request_body = UpdateBacktestChartConfigParams,
    responses(
        (status = 200, description = "Success", body = ApiResponse<StrategyConfig>),
        (status = 500, description = "Internal server error", body = ApiResponse<StrategyConfig>),
    )
)]
pub async fn update_backtest_chart_config(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
    Json(params): Json<UpdateBacktestChartConfigParams>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigMutation::update_backtest_chart_config(conn, strategy_id, Some(params.backtest_chart_config)).await {
        Ok(backtest_chart_config) => {
            tracing::info!(strategy_id = strategy_id, "update backtest chart config success");
            (
                StatusCode::OK,
                Json(ApiResponse {
                    code: 0,
                    message: "success".to_string(),
                    data: Some(backtest_chart_config),
                }),
            )
        }
        Err(e) => {
            tracing::error!(strategy_id = strategy_id, "update backtest chart config error: {}", e);
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
    path = "/api/v1/strategy/backtest/{strategy_id}/chart_config",
    tag = "Backtest Strategy",
    summary = "Get backtest chart configuration",
    responses(
        (status = 200, description = "Success", body = ApiResponse<serde_json::Value>),
        (status = 500, description = "Internal server error", body = ApiResponse<serde_json::Value>),
    )
)]
#[instrument(skip(star_river))]
pub async fn get_backtest_chart_config(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigQuery::get_backtest_chart_config_by_strategy_id(conn, strategy_id).await {
        Ok(backtest_chart_config) => {
            tracing::info!(strategy_id = strategy_id, "get backtest chart config success");
            (
                StatusCode::OK,
                Json(ApiResponse {
                    code: 0,
                    message: "success".to_string(),
                    data: Some(backtest_chart_config),
                }),
            )
        }
        Err(e) => {
            tracing::error!(strategy_id = strategy_id, "get backtest chart config error: {}", e);
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
