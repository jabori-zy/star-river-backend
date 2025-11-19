use ::entity::system_config;
use chrono::Utc;
use sea_orm::*;
use star_river_core::system::system_config::{Localization, SystemConfig};
use crate::error::DatabaseError;

pub struct SystemConfigMutation;

impl SystemConfigMutation {
    // 更新系统配置
    pub async fn update_system_config(db: &DbConn, localization: Localization, timezone: String) -> Result<SystemConfig, DatabaseError> {
        // 使用 find() 选择所有数据，然后 limit(1) 只取第一条
        let system_config: system_config::ActiveModel = system_config::Entity::find()
            .limit(1)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Cannot find system config.".to_owned()))
            .map(Into::into)?;

        let system_config_model = system_config::ActiveModel {
            id: system_config.id,
            localization: Set(localization.to_string()),
            timezone: Set(timezone),
            updated_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;

        Ok(SystemConfig::from_model(system_config_model).await)
    }
}
