pub mod mt5_account;

use std::{any::Any, fmt::Debug, str::FromStr};

use entity::{account_config::Model as AccountConfigModel, account_info::Model as AccountInfoModel};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    exchange::{Exchange, ExchangeStatus},
    system::DateTimeUtc,
};

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum Account {
//     Mt5Account(Mt5Account),
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub account_config: AccountConfig,
    pub account_info: Option<AccountInfo>,
    pub exchange_status: ExchangeStatus,
}

impl Account {
    pub fn new(config: AccountConfig, info: Option<AccountInfo>, exchange_status: ExchangeStatus) -> Self {
        Self {
            account_config: config,
            account_info: info,
            exchange_status,
        }
    }

    pub fn get_account_id(&self) -> i32 {
        self.account_config.id
    }

    pub fn get_account_name(&self) -> String {
        self.account_config.account_name.clone()
    }

    pub fn get_exchange(&self) -> Exchange {
        self.account_config.exchange.clone()
    }

    pub fn get_is_available(&self) -> bool {
        self.account_config.is_available
    }

    pub fn get_account_config(&self) -> AccountConfig {
        self.account_config.clone()
    }

    pub fn get_account_info(&self) -> Option<AccountInfo> {
        self.account_info.clone()
    }

    pub fn get_exchange_status(&self) -> ExchangeStatus {
        self.exchange_status.clone()
    }

    pub fn set_exchange_status(&mut self, status: ExchangeStatus) {
        self.exchange_status = status;
    }

    pub fn set_account_info(&mut self, account_info: AccountInfo) {
        self.account_info = Some(account_info);
    }

    pub fn set_account_config(&mut self, account_config: AccountConfig) {
        self.account_config = account_config;
    }
}

// System account configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccountConfig {
    pub id: i32,                   // Account ID
    pub account_name: String,      // Account name
    pub exchange: Exchange,        // Exchange
    pub config: serde_json::Value, // Account config
    pub is_available: bool,        // Is available
    pub is_deleted: bool,          // Is deleted
    pub sort_index: i32,           // Sort index
    #[schema(value_type = String, example = "2021-01-01 00:00:00")]
    pub create_time: DateTimeUtc, // Create time
    #[schema(value_type = String, example = "2021-01-01 00:00:00")]
    pub update_time: DateTimeUtc, // Update time
}

impl From<AccountConfigModel> for AccountConfig {
    fn from(model: AccountConfigModel) -> Self {
        let exchange = match model.exchange.as_str() {
            "metatrader5" => Exchange::Metatrader5(model.account_config["server"].as_str().unwrap_or("").to_string()),
            _ => Exchange::from_str(model.exchange.as_str()).unwrap(),
        };
        Self {
            id: model.id,
            account_name: model.account_name,
            exchange,
            config: model.account_config,
            is_available: model.is_available,
            is_deleted: model.is_delete,
            sort_index: model.sort_index,
            create_time: model.create_time,
            update_time: model.update_time,
        }
    }
}

// Account info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub id: i32,
    pub account_id: i32,          // Config ID
    pub info: serde_json::Value,  // Account info
    pub create_time: DateTimeUtc, // Create time
    pub update_time: DateTimeUtc, // Update time
}

impl From<AccountInfoModel> for AccountInfo {
    fn from(model: AccountInfoModel) -> Self {
        Self {
            id: model.id,
            account_id: model.account_id,
            info: model.info.unwrap(),
            create_time: model.create_time,
            update_time: model.update_time,
        }
    }
}

// Original account
pub trait AccountTrait: Debug + Send + Sync + Any + 'static {
    fn clone_box(&self) -> Box<dyn AccountTrait>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_account_id(&self) -> i32; // Get account ID
    fn get_account_name(&self) -> String; // Get account name
    fn get_exchange(&self) -> Exchange; // Get exchange
    fn get_is_available(&self) -> bool; // Get is available
    fn get_account_config(&self) -> AccountConfig; // Get account config
    fn get_account_info(&self) -> Option<AccountInfo>; // Get account info
    fn get_exchange_status(&self) -> ExchangeStatus; // Get exchange status
    fn set_exchange_status(&mut self, status: ExchangeStatus); // Set exchange status
    fn set_account_info(&mut self, account_info: AccountInfo); // Set account info
}

impl Clone for Box<dyn AccountTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// Binance account config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BinanceAccountConfig {
    pub api_key: String,
    pub api_secret: String,
}

pub trait ExchangeAccountConfig: Debug + Send + Sync + Any + 'static {
    fn clone_box(&self) -> Box<dyn ExchangeAccountConfig>;
    fn as_any(&self) -> &dyn Any;
    fn get_account_id(&self) -> i32; // Get account ID
    fn get_account_name(&self) -> String; // Get account name
    fn get_exchange(&self) -> Exchange; // Get exchange
    fn get_is_available(&self) -> bool; // Get is available
}

impl Clone for Box<dyn ExchangeAccountConfig> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait OriginalAccountInfo: Debug + Send + Sync + Any + 'static {
    fn clone_box(&self) -> Box<dyn OriginalAccountInfo>;
    fn as_any(&self) -> &dyn Any;
    fn get_account_id(&self) -> i32; // Get account ID
    fn to_json(&self) -> serde_json::Value; // Convert to JSON
}

impl Clone for Box<dyn OriginalAccountInfo> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
