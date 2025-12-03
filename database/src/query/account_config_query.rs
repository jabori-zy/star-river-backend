use ::entity::{account_config, account_config::Entity as AccountConfigEntity};
use sea_orm::*;
use star_river_core::{account::AccountConfig, custom_type::AccountId};

use crate::error::DatabaseError;

pub struct AccountConfigQuery;

impl AccountConfigQuery {
    pub async fn get_account_config_list_by_exchange(db: &DbConn, exchange: String) -> Result<Vec<AccountConfig>, DatabaseError> {
        let account_config_models = AccountConfigEntity::find()
            .filter(account_config::Column::Exchange.eq(exchange))
            .filter(account_config::Column::IsDelete.eq(false))
            .all(db)
            .await?;

        let mut account_configs = Vec::new();
        for model in account_config_models {
            let account_config = model.into();
            account_configs.push(account_config);
        }

        Ok(account_configs)
    }

    pub async fn get_account_config_by_id(db: &DbConn, account_id: AccountId) -> Result<AccountConfig, DatabaseError> {
        let account_config_model = AccountConfigEntity::find_by_id(account_id)
            .filter(account_config::Column::IsDelete.eq(false))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(String::from(format!(
                "account {} config not found.",
                account_id
            ))))?;

        Ok(account_config_model.into())
    }

    pub async fn get_all_account_config(db: &DbConn) -> Result<Vec<AccountConfig>, DatabaseError> {
        let account_config_models = AccountConfigEntity::find()
            .filter(account_config::Column::IsDelete.eq(false))
            .all(db)
            .await?;

        let mut account_configs = Vec::new();
        for model in account_config_models {
            let account_config = model.into();
            account_configs.push(account_config);
        }

        Ok(account_configs)
    }
}
