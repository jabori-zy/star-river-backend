use sea_orm::*;
use crate::entities::{account_config, account_config::Entity as AccountConfig};
use types::account::AccountConfig as AccountConfigType;



pub struct AccountConfigQuery;


impl AccountConfigQuery {

    pub async fn get_account_config_by_exchange(
        db: &DbConn,
        exchange: String
    ) -> Result<Vec<AccountConfigType>, DbErr> {
        let account_config = AccountConfig::find().filter(account_config::Column::Exchange.eq(exchange)).all(db).await?;
        Ok(account_config.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_all_account_config(
        db: &DbConn,
    ) -> Result<Vec<AccountConfigType>, DbErr> {
        let account_config = AccountConfig::find().all(db).await?;
        Ok(account_config.into_iter().map(|model| model.into()).collect())
    }
}

