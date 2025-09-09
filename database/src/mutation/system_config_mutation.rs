use ::entity::system_config;
use chrono::Utc;
use sea_orm::*;
use types::system::system_config::{SystemConfig, SystemConfigUpdateParams};

pub struct SystemConfigMutation;

impl SystemConfigMutation {
    // 更新系统配置
    pub async fn update_system_config(
        db: &DbConn,
        new_system_config: SystemConfigUpdateParams,
    ) -> Result<SystemConfig, DbErr> {
        // 使用 find() 选择所有数据，然后 limit(1) 只取第一条
        let system_config: system_config::ActiveModel = system_config::Entity::find()
            .limit(1)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find system config.".to_owned()))
            .map(Into::into)?;

        let system_config_model = system_config::ActiveModel {
            id: system_config.id,
            localization: Set(new_system_config.localization.to_string()),
            updated_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await?;

        Ok(system_config_model.into())
    }
}
