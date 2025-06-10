use sea_orm::*;
use ::entity::system_config::Entity as SystemConfigEntity;
use types::system::system_config::SystemConfig;


pub struct SystemConfigQuery;


impl SystemConfigQuery {
    pub async fn get_system_config(db: &DbConn) -> Result<SystemConfig, DbErr> {
        let system_config = SystemConfigEntity::find()
            .limit(1)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find system config.".to_owned()))?;

        Ok(system_config.into())
    }


}