use crate::api::response::ApiResponse;
use crate::star_river::StarRiver;
use axum::extract::State;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::Json;
use database::mutation::account_config_mutation::AccountConfigMutation;
use database::query::account_config_query::AccountConfigQuery;
use engine::account_engine::AccountEngine;
use event_center::account_event::AccountEvent;
use event_center::Event;
use event_center::EventCenterSingleton;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{Display, EnumString};
use types::account::AccountConfig;
use types::engine::EngineName;
use types::market::Exchange;
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, Deserialize, IntoParams, ToSchema)]
#[schema(title = "登录MT5账户参数", description = "登录指定MT5账户")]
pub struct LoginMt5AccountParams {
    pub account_id: i32,
}

#[axum::debug_handler]
#[utoipa::path(
    post,
    path = "/api/v1/account/start_mt5_terminal",
    tag = "账户管理",
    summary = "启动MT5客户端",
    description = "启动MT5客户端并登录指定MT5账户",
    request_body = LoginMt5AccountParams,
    responses(
        (status = 200, description = "登录成功", content_type = "application/json"),
        (status = 400, description = "登录失败", content_type = "application/json")
    )
)]
pub async fn start_mt5_terminal(
    State(star_river): State<StarRiver>,
    Json(params): Json<LoginMt5AccountParams>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let account_id = params.account_id;

    let engine_manager = star_river.engine_manager.lock().await;
    // 获取account_engine
    let engine = engine_manager.get_engine(EngineName::AccountEngine).await;
    let mut engine_guard = engine.lock().await;
    let account_engine = engine_guard
        .as_any_mut()
        .downcast_mut::<AccountEngine>()
        .unwrap();
    account_engine.register_exchange(account_id).await.unwrap();

    (
        StatusCode::OK,
        Json(ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: None,
        }),
    )
}

#[derive(Serialize, Deserialize, ToSchema, EnumString, Display)]
#[schema(title = "交易所类型", description = "交易所类型")]
pub enum ExchangeType {
    #[serde(rename = "metatrader5")]
    #[strum(serialize = "metatrader5")]
    Metatrader5,
    #[serde(rename = "binance")]
    #[strum(serialize = "binance")]
    Binance,
    #[serde(rename = "huobi")]
    #[strum(serialize = "huobi")]
    Huobi,
    #[serde(rename = "okx")]
    #[strum(serialize = "okx")]
    Okx,
}

#[derive(Serialize, Deserialize, IntoParams, ToSchema)]
#[schema(
    title = "获取指定交易所的账户配置参数",
    description = "获取指定交易所的账户配置"
)]
pub struct GetAccountConfigByExchangeQuery {
    /// 交易所
    #[schema(example = "metatrader5")]
    pub exchange: Option<ExchangeType>,
}

#[utoipa::path(
    get,
    path = "/api/v1/account/config",
    tag = "账户管理",
    summary = "获取账户配置",
    params(GetAccountConfigByExchangeQuery),
    responses(
        (status = 200, description = "获取成功", content_type = "application/json", body = ApiResponse<Vec<AccountConfig>>),
        (status = 400, description = "获取失败", content_type = "application/json", body = ApiResponse<String>)
    )
)]
pub async fn get_account_configs(
    State(star_river): State<StarRiver>,
    Query(params): Query<GetAccountConfigByExchangeQuery>,
) -> (StatusCode, Json<ApiResponse<Vec<AccountConfig>>>) {
    let db = &star_river.database.lock().await.conn;
    let account_config = match params.exchange {
        Some(exchange) => {
            AccountConfigQuery::get_account_config_by_exchange(db, exchange.to_string())
                .await
                .unwrap()
        }
        None => AccountConfigQuery::get_all_account_config(db)
            .await
            .unwrap(),
    };
    (
        StatusCode::OK,
        Json(ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(account_config),
        }),
    )
}

#[derive(Serialize, Deserialize, IntoParams, ToSchema)]
#[schema(
    title = "MT5账户配置",
    description = "MetaTrader 5交易账户的配置信息，包含登录凭据和终端路径",
    example = json!({
        "login": 76898751,
        "password": "HhazJ520....",
        "server": "Exness-MT5Trial5",
        "terminal_path": "D:/Program Files/MetaTrader 5-1/terminal64.exe"
    })
)]
pub struct Mt5AccountConfigParams {
    login: i64,
    password: String,
    server: String,
    terminal_path: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
#[schema(
    title = "账户配置类型",
    description = "支持的交易账户配置类型，目前支持MetaTrader 5",
    discriminator(property_name = "type")
)]
pub enum AccountConfigType {
    /// MetaTrader 5账户配置
    #[schema(title = "MT5账户配置")]
    Mt5(Mt5AccountConfigParams),
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(
    title = "新增账户配置请求",
    description = "创建新的交易账户配置所需的参数",
    example = json!({
        "account_name": "测试账户1",
        "exchange": "metatrader5",
        "account_config": {
            "login": 76898751,
            "password": "HhazJ520....",
            "server":"Exness-MT5Trial5",
            "terminal_path": "D:/Program Files/MetaTrader 5-1/terminal64.exe"
        }
    }),
)]
pub struct AddAccountConfigParams {
    /// 账户名称
    pub account_name: String,
    /// 交易所
    pub exchange: String,

    /// 账户配置
    pub account_config: AccountConfigType,
}

#[utoipa::path(
    post,
    path = "/api/v1/account/config",
    tag = "账户管理",
    summary = "新增账户配置",
    request_body = AddAccountConfigParams,
    responses(
        (status = 200, description = "账户配置创建成功", content_type = "application/json", body = ApiResponse<AccountConfig>),
        (status = 400, description = "账户配置创建失败", content_type = "application/json", body = ApiResponse<String>)
    )
)]
// 新增账户配置
pub async fn add_account_config(
    State(star_river): State<StarRiver>,
    Json(request): Json<AddAccountConfigParams>,
) -> (StatusCode, Json<ApiResponse<AccountConfig>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;

    let account_config_json =
        serde_json::to_value(&request.account_config).expect("Invalid account config");
    tracing::info!("account_config_json: {:?}", account_config_json);
    match AccountConfigMutation::insert_account_config(
        conn,
        request.account_name,
        Exchange::from_str(request.exchange.as_str()).expect("Invalid exchange"),
        account_config_json,
    )
    .await
    {
        Ok(account_config) => {
            // 添加成功之后，发布账户配置已添加事件
            tracing::info!("添加账户配置成功: {:?}", account_config);
            // let event_center = star_river.event_center.lock().await;
            // let event_publisher = EventCenterSingleton::publisher().await.unwrap();
            // let event_publisher = event_center.get_event_publisher();
            let event = Event::Account(AccountEvent::AccountConfigAdded(account_config.id));
            EventCenterSingleton::publish(event).await.unwrap();

            (
                StatusCode::OK,
                Json(ApiResponse {
                    code: 0,
                    message: "success".to_string(),
                    data: Some(account_config),
                }),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: e.to_string(),
                data: None,
            }),
        ),
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/account/config/{account_id}",
    tag = "账户管理",
    summary = "删除账户配置",
    description = "根据账户ID删除指定的账户配置，删除成功后会发布账户配置已删除事件",
    params(
        ("account_id" = i32, Path, description = "要删除的账户配置ID")
    ),
    responses(
        (status = 200, description = "账户配置删除成功", content_type = "application/json"),
        (status = 400, description = "账户配置删除失败", content_type = "application/json")
    )
)]
pub async fn delete_account_config(
    State(star_river): State<StarRiver>,
    Path(account_id): Path<i32>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match AccountConfigMutation::delete_account_config(conn, account_id).await {
        Ok(_) => {
            // 删除成功之后，发布账户配置已删除事件
            // let event_center = star_river.event_center.lock().await;
            // let event_publisher = event_center.get_event_publisher();
            let event_publisher = EventCenterSingleton::publisher().await.unwrap();
            let event = Event::Account(AccountEvent::AccountConfigDeleted(account_id));
            EventCenterSingleton::publish(event).await.unwrap();

            (
                StatusCode::OK,
                Json(ApiResponse {
                    code: 0,
                    message: "账户配置删除成功".to_string(),
                    data: None,
                }),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: e.to_string(),
                data: None,
            }),
        ),
    }
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams)]
#[schema(
    title = "更新账户配置请求",
    description = "更新账户配置所需的参数",
    example = json!({
        "account_name": "测试账户1",
        "account_config": {
            "login": 76898751,
            "password": "HhazJ520....",
            "server": "Exness-MT5Trial5",
            "terminal_path": "D:/Program Files/MetaTrader 5-1/terminal64.exe"
        },
        "is_available": true,
        "sort_index": 1
    })
)]
pub struct UpdateAccountConfigParams {
    pub account_name: String,
    pub account_config: AccountConfigType,
    pub is_available: bool,
    pub sort_index: i32,
}

#[utoipa::path(
    put,
    path = "/api/v1/account/config/{account_id}",
    tag = "账户管理",
    summary = "更新账户配置",
    request_body = UpdateAccountConfigParams,
    responses(
        (status = 200, description = "账户配置更新成功", content_type = "application/json", body = ApiResponse<AccountConfig>),
        (status = 400, description = "账户配置更新失败", content_type = "application/json", body = ApiResponse<String>)
    )
)]
pub async fn update_account_config(
    State(star_river): State<StarRiver>,
    Path(account_id): Path<i32>,
    Json(params): Json<UpdateAccountConfigParams>,
) -> (StatusCode, Json<ApiResponse<AccountConfig>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    let account_config_json =
        serde_json::to_value(&params.account_config).expect("Invalid account config");
    match AccountConfigMutation::update_account_config(
        conn,
        account_id,
        params.account_name,
        account_config_json,
        params.is_available,
        params.sort_index,
    )
    .await
    {
        Ok(account_config) => {
            // 更新成功之后，发布账户配置已更新事件
            // let event_center = star_river.event_center.lock().await;
            // let event_publisher = event_center.get_event_publisher();
            let event_publisher = EventCenterSingleton::publisher().await.unwrap();
            let event = Event::Account(AccountEvent::AccountConfigUpdated(account_config.clone()));
            EventCenterSingleton::publish(event).await.unwrap();

            (
                StatusCode::OK,
                Json(ApiResponse {
                    code: 0,
                    message: "账户配置更新成功".to_string(),
                    data: Some(account_config),
                }),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: e.to_string(),
                data: None,
            }),
        ),
    }
}

#[derive(Serialize, Deserialize, IntoParams, ToSchema)]
#[schema(
    title = "更新账户可用状态参数",
    description = "通过查询参数更新账户的可用状态"
)]
pub struct UpdateAccountIsAvailableQuery {
    /// 账户是否可用
    #[schema(example = true)]
    pub is_available: bool,
}

#[utoipa::path(
    patch,
    path = "/api/v1/account/config/{account_id}/status",
    tag = "账户管理",
    summary = "更新账户可用状态",
    description = "更新指定账户配置的可用状态",
    params(
        ("account_id" = i32, Path, description = "账户配置ID"),
        UpdateAccountIsAvailableQuery
    ),
    responses(
        (status = 200, description = "状态更新成功", content_type = "application/json", body = ApiResponse<AccountConfig>),
        (status = 400, description = "状态更新失败", content_type = "application/json", body = ApiResponse<String>)
    )
)]
pub async fn update_account_is_available(
    State(star_river): State<StarRiver>,
    Path(account_id): Path<i32>,
    Query(query): Query<UpdateAccountIsAvailableQuery>,
) -> (StatusCode, Json<ApiResponse<AccountConfig>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match AccountConfigMutation::update_account_config_is_available(
        conn,
        account_id,
        query.is_available,
    )
    .await
    {
        Ok(account_config) => {
            // 更新成功之后，发布账户配置已更新事件
            // let event_center = star_river.event_center.lock().await;
            // let event_publisher = event_center.get_event_publisher();
            // let event_publisher = EventCenterSingleton::publisher().await.unwrap();
            let event = Event::Account(AccountEvent::AccountConfigUpdated(account_config.clone()));
            EventCenterSingleton::publish(event).await.unwrap();

            (
                StatusCode::OK,
                Json(ApiResponse {
                    code: 0,
                    message: "状态更新成功".to_string(),
                    data: Some(account_config),
                }),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: e.to_string(),
                data: None,
            }),
        ),
    }
}
