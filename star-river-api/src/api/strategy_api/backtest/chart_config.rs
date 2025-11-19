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
    title = "更新回测图表配置参数",
    description = "更新回测图表配置参数",
    example = json!({
        "charts": [
            {
                "id": "1",
                "chartName": "K线图",
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
    tag = "回测策略",
    summary = "更新回测图表配置",
    params(UpdateBacktestChartConfigParams),
    request_body = UpdateBacktestChartConfigParams,
    responses(
        (status = 200, description = "成功", body = ApiResponse<StrategyConfig>),
        (status = 500, description = "内部服务器错误", body = ApiResponse<StrategyConfig>),
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
    tag = "回测策略",
    summary = "获取回测图表配置",
    responses(
        (status = 200, description = "成功", body = ApiResponse<serde_json::Value>),
        (status = 500, description = "内部服务器错误", body = ApiResponse<serde_json::Value>),
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
