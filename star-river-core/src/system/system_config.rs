use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use utoipa::ToSchema;

// 本地化
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString, Display, ToSchema,
)]
pub enum Localization {
    #[serde(rename = "zh-CN")]
    #[strum(serialize = "zh-CN")]
    Chinese, // 中文
    #[serde(rename = "en-US")]
    #[strum(serialize = "en-US")]
    English, // 英文
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemConfigUpdateParams {
    /// 本地化
    pub localization: Localization,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemConfig {
    pub id: i32,
    /// 本地化
    pub localization: Localization,
    /// 创建时间
    pub created_time: DateTime<Utc>,
    /// 更新时间
    pub updated_time: DateTime<Utc>,
}
