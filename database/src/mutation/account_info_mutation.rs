use sea_orm::*;
use ::entity::account_info;
use chrono::Utc;
use types::account::AccountInfo;



pub struct AccountInfoMutation;


impl AccountInfoMutation {
    pub async fn insert_account_info(
        db: &DbConn,
        account_id: i32,
        info: serde_json::Value,
    ) -> Result<AccountInfo, DbErr> {
        let account_info = account_info::ActiveModel {
            id: NotSet,
            account_id: Set(account_id),
            info: Set(Some(info)),
            create_time: Set(Utc::now()),
            update_time: Set(Utc::now()),
        };
        let account_info = account_info.insert(db).await?;
        Ok(account_info.into())
    }

    pub async fn update_account_info(
        db: &DbConn,
        account_id: i32,
        info: serde_json::Value,
    ) -> Result<AccountInfo, DbErr> {
        // 先查询，如果存在则更新，否则插入
        let account_info_model_option = account_info::Entity::find()
            .filter(account_info::Column::AccountId.eq(account_id))
            .one(db)
            .await?;
        match account_info_model_option {
            Some(account_info_model) => {
                let active_model = account_info::ActiveModel {
                    id: Set(account_info_model.id),
                    info: Set(Some(info)),
                    update_time: Set(Utc::now()),
                    ..Default::default()
                };
                let account_info_model = active_model.update(db).await?;
                Ok(account_info_model.into())
            }
            None => {
                let account_info_model = AccountInfoMutation::insert_account_info(db, account_id, info).await?;
                Ok(account_info_model)
            }
        }
    }

}

