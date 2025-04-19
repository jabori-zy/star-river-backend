use types::account::AccountConfig as TypeAccountConfig;
use sea_orm::*;
use crate::entities::account_config;
use sea_orm::entity::prelude::*;
use chrono::Utc;
use types::market::Exchange;


pub struct AccountConfigMutation;


impl AccountConfigMutation {

    pub async fn insert_account_config(
        db: &DbConn,
        account_name: String,
        exchange: Exchange,
        account_config: Json
    ) -> Result<TypeAccountConfig, DbErr> {
        let account_config_model = account_config::ActiveModel {
            id: NotSet,
            account_name: Set(account_name),
            exchange: Set(exchange.to_string()),
            is_available: Set(true),
            account_config: Set(account_config),
            created_time: Set(Utc::now()),
            updated_time: Set(Utc::now()),
        }.insert(db).await.unwrap();

        Ok(account_config_model.into())
    }
}

