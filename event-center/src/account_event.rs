use serde::{Deserialize, Serialize};
use strum::Display;
use crate::Event;
use types::account::Account;
use types::account::AccountConfig;


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event_name")]
pub enum AccountEvent {
    #[strum(serialize = "account-config-added")]
    #[serde(rename = "account-config-added")]
    AccountConfigAdded(i32), // 账户配置已添加

    #[strum(serialize = "account-config-deleted")]
    #[serde(rename = "account-config-deleted")]
    AccountConfigDeleted(i32), // 账户配置已删除

    #[strum(serialize = "account-config-updated")]
    #[serde(rename = "account-config-updated")]
    AccountConfigUpdated(AccountConfig), // 账户配置已更新


    #[strum(serialize = "account-updated")]
    #[serde(rename = "account-updated")]
    AccountUpdated(Account), // 账户信息更新


}

impl From<AccountEvent> for Event {
    fn from(event: AccountEvent) -> Self {
        Event::Account(event)
    }
}






