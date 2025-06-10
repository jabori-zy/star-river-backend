use types::system::system_config::SystemConfig;
use crate::system_config::Model as SystemConfigModel;
use types::system::system_config::Localization;
use std::str::FromStr;

impl From<SystemConfigModel> for SystemConfig {
    fn from(config: SystemConfigModel) -> Self {
        SystemConfig {
            id: config.id,
            localization: Localization::from_str(&config.localization).unwrap(),
            created_time: config.created_time,
            updated_time: config.updated_time,
        }
    }
}