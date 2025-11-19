use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use database::{page::PageResult, query::strategy_config_query::StrategyConfigQuery};
use serde::{Deserialize, Serialize};
use snafu::Report;
use star_river_core::{error::{StarRiverErrorTrait}, strategy::StrategyInfo};
use utoipa::{IntoParams, ToSchema};
use strategy_core::strategy::StrategyConfig;
use crate::{api::response::ApiResponseEnum, error::{CharacterLengthExceedsLimitSnafu, EmptyCharacterSnafu, PageMustGreaterThanOneSnafu, TooManyItemsPerPageSnafu}, star_river::StarRiver};
use tracing::instrument;
use database::mutation::strategy_config_mutation::StrategyConfigMutation;

const STRATEGY_MANAGEMENT_TAG: &str = "Strategy Management";

#[derive(Serialize, Deserialize, IntoParams, ToSchema)]
#[schema(
    title = "Get strategy list query",
    description = "Get strategy list query",
    example = json!({
        "page": 1,
        "strategy_per_page": 10
    })
)]
pub struct GetStrategyListQuery {
    /// Page number (starts from 1)
    #[schema(example = 1, minimum = 1, default = 1)]
    pub page: u64,
    /// Number of strategies per page
    #[schema(example = 10, minimum = 1, maximum = 100, default = 10)]
    pub items_per_page: u64,
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/v1/strategy",
    tag = STRATEGY_MANAGEMENT_TAG,
    summary = "Get strategy list",
    params(GetStrategyListQuery),
    responses(
        (status = 200, description = "Success", body = ApiResponseEnum<PageResult<StrategyInfo>>),
        (status = 400, description = "Bad Request", body = ApiResponseEnum<PageResult<StrategyInfo>>)
    )
)]
pub async fn get_strategy_list(
    State(star_river): State<StarRiver>,
    Query(params): Query<GetStrategyListQuery>,
) -> (StatusCode, Json<ApiResponseEnum<PageResult<StrategyInfo>>>) {
    if params.page == 0 {
        let error = PageMustGreaterThanOneSnafu{
            page: params.page,
        }.build();
        return (error.http_status_code(), Json(ApiResponseEnum::error(error)));
    }
    if params.items_per_page == 0 || params.items_per_page > 100 {
        let error = TooManyItemsPerPageSnafu{
            items_per_page: params.items_per_page,
        }.build();
        return (error.http_status_code(), Json(ApiResponseEnum::error(error)));
    }
    
    if params.items_per_page > 100 {
        let error = TooManyItemsPerPageSnafu{
            items_per_page: params.items_per_page,
        }.build();
        return (error.http_status_code(), Json(ApiResponseEnum::error(error)));
    }

    let db = &star_river.database.lock().await.conn;
    let page_result = StrategyConfigQuery::get_strategy_list_in_page(db, params.page, params.items_per_page).await;

    match page_result {
        Ok(page_result) => {
            tracing::debug!("get strategy list successfully");
            (StatusCode::OK, Json(ApiResponseEnum::success(page_result)))
        }
        Err(e) => {
            let report = Report::from_error(&e);
            tracing::error!("get strategy list failed: {}", report);
            (e.http_status_code(), Json(ApiResponseEnum::error(e)))
        }
    }
}



#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/v1/strategy/{strategy_id}",
    tag = STRATEGY_MANAGEMENT_TAG,
    summary = "get strategy config by id",
    params(
        ("strategy_id" = i32, Path, description = "strategy id")
    ),
    responses(
        (status = 200, body = ApiResponseEnum<StrategyConfig>),
        (status = 400, body = ApiResponseEnum<StrategyConfig>)
    )
)]
#[instrument(skip(star_river))]
pub async fn get_strategy_by_id(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
) -> (StatusCode, Json<ApiResponseEnum<StrategyConfig>>) {
    // 模拟加载状态，睡眠3秒
    // sleep(Duration::from_secs(3)).await;
    
    let db = &star_river.database.lock().await.conn;
    let strategy = StrategyConfigQuery::get_strategy_by_id(db, strategy_id).await;
    match strategy {
        Ok(strategy) => {
            tracing::info!("get strategy {} config by id successfully", strategy_id);
            (StatusCode::OK, Json(ApiResponseEnum::success(strategy)))
        }
        Err(e) => {
            let report = Report::from_error(&e);
            tracing::error!("get strategy {} config by id failed: {}", strategy_id, report);
            (e.http_status_code(), Json(ApiResponseEnum::error(e)))
        }
    }
}




#[derive(Serialize, Deserialize, IntoParams, ToSchema)]
#[schema(
    title = "create strategy params",
    description = "create strategy params",
    example = json!({
        "name": "test strategy 1",
        "description": "test strategy 1 description",
    })
)]
pub struct CreateStrategyParams {
    /// 策略名称
    pub name: String,
    /// 策略描述
    pub description: String,
}

#[axum::debug_handler]
#[utoipa::path(
    post,
    path = "/api/v1/strategy",
    tag = STRATEGY_MANAGEMENT_TAG,
    summary = "Create strategy",
    request_body = CreateStrategyParams,
    responses(
        (status = 200, body = ApiResponseEnum<StrategyConfig>),
        (status = 400, body = ApiResponseEnum<StrategyConfig>)
    )
)]
pub async fn create_strategy(
    State(star_river): State<StarRiver>,
    Json(params): Json<CreateStrategyParams>,
) -> (StatusCode, Json<ApiResponseEnum<StrategyConfig>>) {


    // check name character length
    if params.name.len() > 20 {
        let error = CharacterLengthExceedsLimitSnafu{
            name: "strategy name".to_string(),
            length: params.name.len() as i32,
            max_length: 20 as i32,
        }.build();
        let report = Report::from_error(&error);
        tracing::error!("{}", report);
        return (error.http_status_code(), Json(ApiResponseEnum::error(error)));
    }


    if params.description.len() > 100 {
        let error = CharacterLengthExceedsLimitSnafu{
            name: "strategy description".to_string(),
            length: params.description.len() as i32,
            max_length: 100 as i32,
        }.build();
        let report = Report::from_error(&error);
        tracing::error!("{}", report);
        return (error.http_status_code(), Json(ApiResponseEnum::error(error)));
    }



    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigMutation::create_strategy(conn, params.name, params.description).await {
        Ok(strategy) => {
            tracing::info!("strategy created successfully. strategy id: {}", strategy.id);
            (
                StatusCode::CREATED,
                Json(ApiResponseEnum::success(strategy))
            )
        }
        Err(e) => {
            let report = Report::from_error(&e);
            tracing::error!("create strategy failed: {}", report);
            (e.http_status_code(), Json(ApiResponseEnum::error(e)))
        }
    }
}




#[derive(Serialize, Deserialize, Debug, IntoParams, ToSchema)]
#[schema(title = "Update strategy params", description = "Update strategy params")]
pub struct UpdateStrategyParams {
    /// Strategy name
    pub name: String,
    /// Strategy description
    pub description: String,
    /// Strategy trade mode
    pub trade_mode: String,
    /// Strategy nodes
    pub nodes: Option<serde_json::Value>,
    /// Strategy edges
    pub edges: Option<serde_json::Value>,
}

#[utoipa::path(
    put,
    path = "/api/v1/strategy/{strategy_id}",
    tag = STRATEGY_MANAGEMENT_TAG,
    summary = "Update strategy",
    params(
        UpdateStrategyParams
    ),
    responses(
        (status = 200, body = ApiResponseEnum<StrategyConfig>),  
        (status = 400, body = ApiResponseEnum<StrategyConfig>)
    )
)]
#[instrument(skip(star_river, params))]
pub async fn update_strategy(
    State(star_river): State<StarRiver>,
    Path(strategy_id): Path<i32>,
    Json(params): Json<UpdateStrategyParams>,
) -> (StatusCode, Json<ApiResponseEnum<StrategyConfig>>) {
    // check name character length
    if params.name.len() > 20 {
        let error = CharacterLengthExceedsLimitSnafu{
            name: "strategy name".to_string(),
            length: params.name.len() as i32,
            max_length: 20 as i32,
        }.build();
        let report = Report::from_error(&error);
        tracing::error!("{}", report);
        return (error.http_status_code(), Json(ApiResponseEnum::error(error)));
    }

    if params.description.len() > 100 {
        let error = CharacterLengthExceedsLimitSnafu{
            name: "strategy description".to_string(),
            length: params.description.len() as i32,
            max_length: 100 as i32,
        }.build();
        let report = Report::from_error(&error);
        tracing::error!("{}", report);
        return (error.http_status_code(), Json(ApiResponseEnum::error(error)));
    }

    if params.name.is_empty() {
        let error = EmptyCharacterSnafu{
            name: "strategy name".to_string(),
        }.build();
        let report = Report::from_error(&error);
        tracing::error!("{}", report);
        return (error.http_status_code(), Json(ApiResponseEnum::error(error)));
    }

    
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigMutation::update_strategy_by_id(
        conn,
        strategy_id,
        params.name,
        params.description,
        params.trade_mode,
        params.nodes,
        params.edges,
    )
    .await
    {
        Ok(strategy) => {
            tracing::info!("strategy {} updated successfully", strategy_id);
            (StatusCode::OK, Json(ApiResponseEnum::success(strategy)))
        }
        Err(e) => {
            let db_error = database::error::DatabaseError::from(e);
            let report = Report::from_error(&db_error);
            tracing::error!("update strategy {} failed: {}", strategy_id, report);
            (db_error.http_status_code(), Json(ApiResponseEnum::error(db_error)))
        }
    }
}


#[axum::debug_handler]
#[utoipa::path(
    delete,
    path = "/api/v1/strategy/{strategy_id}",
    tag = "策略管理",
    summary = "删除策略",
    params(
        ("strategy_id" = i32, Path, description = "要删除的策略ID")
    ),
    responses(
        (status = 200, description = "策略删除成功", content_type = "application/json" ),
        (status = 400, description = "策略删除失败", content_type = "application/json")
    )
)]
pub async fn delete_strategy(State(star_river): State<StarRiver>, Path(strategy_id): Path<i32>) -> (StatusCode, Json<ApiResponseEnum<()>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match StrategyConfigMutation::delete_strategy(conn, strategy_id).await {
        Ok(_) => {
            tracing::info!("Delete strategy {strategy_id} successfully");
            (StatusCode::OK, Json(ApiResponseEnum::success(())))
        }
        Err(e) => {
            let report = Report::from_error(&e);
            tracing::error!("{report}");
            (e.http_status_code(), Json(ApiResponseEnum::error(e)))
        }
    }
}
