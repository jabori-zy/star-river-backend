use crate::system_config::Model as SystemConfigModel;
use star_river_core::system::system_config::Localization;
use star_river_core::system::system_config::SystemConfig;
use std::str::FromStr;
use chrono_tz::Tz;

impl From<SystemConfigModel> for SystemConfig {
    fn from(config: SystemConfigModel) -> Self {
        SystemConfig {
            id: config.id,
            localization: Localization::from_str(&config.localization).unwrap(),
            timezone: Tz::from_str(&config.timezone).unwrap(),
            created_time: config.created_time,
            updated_time: config.updated_time,
        }
    }
}
