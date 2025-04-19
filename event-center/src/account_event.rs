use serde::{Deserialize, Serialize};
use strum::Display;
use types::market::Exchange;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event_name")]
pub enum AccountEvent {
    #[strum(serialize = "account-config-added")]
    #[serde(rename = "account-config-added")]
    AccountConfigAdded((i32, Exchange)), // mt5账户配置已添加
    #[strum(serialize = "account-config-deleted")]
    #[serde(rename = "account-config-deleted")]
    AccountConfigDeleted(i32), // mt5账户配置已删除

}