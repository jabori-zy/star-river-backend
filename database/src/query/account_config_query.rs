use sea_orm::*;
use ::entity::{account_config, account_config::Entity as AccountConfigEntity};
use types::{account::AccountConfig, custom_type::AccountId, market::Exchange};


pub struct AccountConfigQuery;


impl AccountConfigQuery {

    pub async fn get_account_config_by_exchange(
        db: &DbConn,
        exchange: String
    ) -> Result<Vec<AccountConfig>, DbErr> {
        let account_config = AccountConfigEntity::find()
        .filter(account_config::Column::Exchange.eq(exchange))
        .filter(account_config::Column::IsDelete.eq(false))
        .all(db).await?;
        Ok(account_config.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_account_config_by_id(
        db: &DbConn,
        account_id: AccountId
    ) -> Result<AccountConfig, DbErr> {
        let account_config = AccountConfigEntity::find_by_id(account_id)
            .filter(account_config::Column::IsDelete.eq(false))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find account config.".to_owned()))
            .map(Into::into)?;
        Ok(account_config)
    }

    pub async fn get_all_account_config(
        db: &DbConn,
    ) -> Result<Vec<AccountConfig>, DbErr> {
        let account_config = AccountConfigEntity::find()
        .filter(account_config::Column::IsDelete.eq(false))
        .all(db).await?;
        Ok(account_config.into_iter().map(|model| model.into()).collect())
    }
}

