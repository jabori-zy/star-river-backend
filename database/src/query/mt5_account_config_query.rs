use sea_orm::*;
use crate::entities::{mt5_account_config, mt5_account_config::Entity as Mt5AccountConfig};
use types::account::mt5_account::Mt5AccountConfig as Mt5AccountConfigType;



pub struct Mt5AccountConfigQuery;


impl Mt5AccountConfigQuery {

    pub async fn get_mt5_account_config(
        db: &DbConn
    ) -> Result<Vec<Mt5AccountConfigType>, DbErr> {
        // 查找is_deleted为false的mt5账户配置
        let account_config = Mt5AccountConfig::find().filter(mt5_account_config::Column::IsDeleted.eq(false)).all(db).await?;
        Ok(account_config.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_mt5_account_config_by_id(
        db: &DbConn,
        id: i32
    ) -> Result<Option<Mt5AccountConfigType>, DbErr> {
        let account_config = Mt5AccountConfig::find_by_id(id).one(db).await?;
        Ok(account_config.map(|model| model.into()))
    }
}

