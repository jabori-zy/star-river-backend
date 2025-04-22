use sea_orm::*;
use crate::entities::{mt5_account_info, mt5_account_info::Entity as Mt5AccountInfo};
use types::account::mt5_account::Mt5AccountInfo as Mt5AccountInfoType;



pub struct Mt5AccountInfoQuery;


impl Mt5AccountInfoQuery {
    pub async fn get_mt5_account_info_by_account_id(
        db: &DbConn,
        account_id: i32
    ) -> Result<Option<Mt5AccountInfoType>, DbErr> {
        let account_info = Mt5AccountInfo::find().filter(mt5_account_info::Column::AccountId.eq(account_id)).one(db).await?;
        Ok(account_info.map(|model| model.into()))
    }
}

