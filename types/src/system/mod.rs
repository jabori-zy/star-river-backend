pub mod system_config;
use serde::{Serialize, Deserialize};
use strum::{EnumString, Display};
use utoipa::{IntoParams, ToSchema};




// 系统配置项
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString, Display, ToSchema)]
pub enum SystemConfigKey {
    #[serde(rename = "localization")]
    #[strum(serialize = "localization")]
    Localization, // 本地化
}