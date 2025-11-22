use ::entity::system_config::Entity as SystemConfigEntity;
use sea_orm::*;
use star_river_core::system::system_config::SystemConfig;

use crate::error::DatabaseError;

pub struct SystemConfigQuery;

impl SystemConfigQuery {
    pub async fn get_system_config(db: &DbConn) -> Result<SystemConfig, DatabaseError> {
        let system_config = SystemConfigEntity::find()
            .limit(1)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Cannot find system config.".to_owned()))?;

        Ok(SystemConfig::from_model(system_config).await)
    }
}
