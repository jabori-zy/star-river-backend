use crate::system::DateTimeUtc;
use chrono::Utc;
use chrono_tz::{Asia::Shanghai, Tz};
use entity::system_config::Model as SystemConfigModel;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicPtr, Ordering},
};
use strum::{Display, EnumString};
use utoipa::ToSchema;

// 本地化
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString, Display, ToSchema)]
pub enum Localization {
    #[serde(rename = "zh-CN")]
    #[strum(serialize = "zh-CN")]
    Chinese, // 中文
    #[serde(rename = "en-US")]
    #[strum(serialize = "en-US")]
    English, // 英文
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SystemConfig {
    pub id: i32,
    /// 本地化
    pub localization: Localization,

    /// 时区
    #[schema(value_type = String, example = "Asia/Shanghai")]
    pub timezone: Tz,

    /// 创建时间
    #[schema(value_type = String, example = "2021-01-01 00:00:00")]
    pub created_time: DateTimeUtc,
    /// 更新时间
    #[schema(value_type = String, example = "2021-01-01 00:00:00")]
    pub updated_time: DateTimeUtc,
}

impl SystemConfig {
    pub async fn from_model(model: SystemConfigModel) -> Self {
        let timezone = model.timezone.parse::<Tz>().unwrap();
        Self {
            id: model.id,
            localization: Localization::from_str(&model.localization).unwrap(),
            timezone,
            created_time: model.created_time,
            updated_time: model.updated_time,
        }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            id: 0,
            localization: Localization::English,
            timezone: Shanghai,
            created_time: Utc::now(),
            updated_time: Utc::now(),
        }
    }
}

/// 全局系统配置状态，使用原子指针
static SYSTEM_CONFIG: LazyLock<AtomicPtr<SystemConfig>> = LazyLock::new(|| {
    let config = Box::new(SystemConfig::default());
    AtomicPtr::new(Box::into_raw(config))
});

/// 系统配置管理器
pub struct SystemConfigManager;

impl SystemConfigManager {
    /// 获取当前全局系统配置的 Arc 引用
    pub fn get_config() -> Arc<SystemConfig> {
        let ptr = SYSTEM_CONFIG.load(Ordering::Acquire);
        // 安全：我们永远不会释放这个指针，只会替换它
        let config = unsafe { &*ptr };
        Arc::new(config.clone())
    }

    /// 更新全局系统配置
    pub fn update_config(new_config: SystemConfig) {
        let new_ptr = Box::into_raw(Box::new(new_config));
        let old_ptr = SYSTEM_CONFIG.swap(new_ptr, Ordering::AcqRel);

        // 释放旧的配置（但不在程序结束时，避免内存泄漏）
        if !old_ptr.is_null() {
            unsafe {
                let _old_box = Box::from_raw(old_ptr);
                // Box 会自动释放内存
            }
        }
    }

    /// 从数据库加载系统配置并初始化（应该在系统启动时调用）
    pub fn initialize_from_db(config: SystemConfig) {
        tracing::debug!("Initializing system config from DB: {:?}", config);
        Self::update_config(config);
        tracing::debug!("System config initialized successfully");
    }

    /// 获取当前系统时区
    pub fn get_timezone() -> Tz {
        let ptr = SYSTEM_CONFIG.load(Ordering::Acquire);
        unsafe { (*ptr).timezone }
    }

    /// 获取当前系统本地化设置
    pub fn get_localization() -> Localization {
        let ptr = SYSTEM_CONFIG.load(Ordering::Acquire);
        unsafe { (*ptr).localization.clone() }
    }
}
