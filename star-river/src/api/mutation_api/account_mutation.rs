use axum::extract::{Json, Query, State, Path};
use utoipa::{IntoParams, ToSchema};
use crate::StarRiver;
use axum::http::StatusCode;
use serde::{Serialize, Deserialize};
use crate::api::response::ApiResponse;
use database::mutation::account_config_mutation::AccountConfigMutation;
use types::market::Exchange;
use std::str::FromStr;
use event_center::Event;
use event_center::account_event::AccountEvent;
use types::account::ExchangeAccountConfig;
use serde_json::json;
use types::account::AccountConfig;


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
    tag = "account",
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

    let account_config_json = serde_json::to_value(&request.account_config).expect("Invalid account config");
    tracing::info!("account_config_json: {:?}", account_config_json);
    match AccountConfigMutation::insert_account_config(
        conn, 
        request.account_name, 
        Exchange::from_str(request.exchange.as_str()).expect("Invalid exchange"),
        account_config_json
    ).await {
        Ok(account_config) => {
            // 添加成功之后，发布账户配置已添加事件
            tracing::info!("添加账户配置成功: {:?}", account_config);
            let event_center = star_river.event_center.lock().await;
            let event_publisher = event_center.get_event_publisher();
            let event = Event::Account(AccountEvent::AccountConfigAdded(account_config.id));
            event_publisher.publish(event).await.unwrap();

            (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "success".to_string(),
                data: Some(account_config),
            })
        )
    },
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: e.to_string(),
                data: None,
            })
        ),
    }
}


#[derive(Serialize, Deserialize)]
pub struct DeleteAccountConfigRequest {
    pub id: i32,
}


pub async fn delete_account_config(
    State(star_river): State<StarRiver>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    match AccountConfigMutation::delete_account_config(conn, id).await {
        Ok(_) => {
            // 删除成功之后，发布账户配置已删除事件
            let event_center = star_river.event_center.lock().await;
            let event_publisher = event_center.get_event_publisher();
            let event = Event::Account(AccountEvent::AccountConfigDeleted(id));
            event_publisher.publish(event).await.unwrap();

            (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "删除成功".to_string(),
                data: None,
            })
        )},
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: e.to_string(),
                data: None,
            })
        ),
    }
}


#[derive(Serialize, Deserialize)]
pub struct UpdateAccountConfigRequest {
    pub id: i32,
    pub account_name: String,
    pub account_config: serde_json::Value,
    pub is_available: bool,
    pub sort_index: i32,
}


pub async fn update_account_config(
    State(star_river): State<StarRiver>,
    Path(id): Path<i32>,
    Json(mut request): Json<UpdateAccountConfigRequest>,
) -> (StatusCode, Json<ApiResponse<AccountConfig>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    // 使用路径参数中的id，覆盖请求体中的id
    request.id = id;
    match AccountConfigMutation::update_account_config(
        conn, 
        request.id, 
        request.account_name, 
        request.account_config, 
        request.is_available, 
        request.sort_index
    ).await {
        Ok(account_config) => {
            // 更新成功之后，发布账户配置已更新事件
            let event_center = star_river.event_center.lock().await;
            let event_publisher = event_center.get_event_publisher();
            let event = Event::Account(AccountEvent::AccountConfigUpdated(account_config.clone()));
            event_publisher.publish(event).await.unwrap();

            (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "更新成功".to_string(),
                data: Some(account_config),
            })
        )
    },
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: e.to_string(),
                data: None,
            })
        ),
    }
}



#[derive(Serialize, Deserialize,Debug)]
pub struct UpdateIsAvailableRequest {
    pub id: i32,
    pub is_available: bool,
}


pub async fn update_account_config_is_available(
    State(star_river): State<StarRiver>,
    Path(id): Path<i32>,
    Json(mut request): Json<UpdateIsAvailableRequest>,
) -> (StatusCode, Json<ApiResponse<AccountConfig>>) {
    let database = star_river.database.lock().await;
    let conn = &database.conn;
    // 使用路径参数中的id，覆盖请求体中的id
    request.id = id;
    match AccountConfigMutation::update_account_config_is_available(conn, request.id, request.is_available).await {
        Ok(account_config) => {
            // 更新成功之后，发布账户配置已更新事件
            let event_center = star_river.event_center.lock().await;
            let event_publisher = event_center.get_event_publisher();
            let event = Event::Account(AccountEvent::AccountConfigUpdated(account_config.clone()));
            event_publisher.publish(event).await.unwrap();

            (
            StatusCode::OK,
            Json(ApiResponse {
                code: 0,
                message: "更新成功".to_string(),
                data: Some(account_config),
            })
        )
    },
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                code: -1,
                message: e.to_string(),
                data: None,
            })
        ),
    }
}