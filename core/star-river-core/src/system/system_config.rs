use std::{
    str::FromStr,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicPtr, Ordering},
    },
};

use chrono::Utc;
use chrono_tz::{Asia::Shanghai, Tz};
use entity::system_config::Model as SystemConfigModel;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use utoipa::ToSchema;

use crate::system::DateTimeUtc;

// Localization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString, Display, ToSchema)]
pub enum Localization {
    #[serde(rename = "zh-CN")]
    #[strum(serialize = "zh-CN")]
    Chinese, // Chinese
    #[serde(rename = "en-US")]
    #[strum(serialize = "en-US")]
    English, // English
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SystemConfig {
    pub id: i32,
    /// Localization
    pub localization: Localization,

    /// Timezone
    #[schema(value_type = String, example = "Asia/Shanghai")]
    pub timezone: Tz,

    /// Created time
    #[schema(value_type = String, example = "2021-01-01 00:00:00")]
    pub created_time: DateTimeUtc,
    /// Updated time
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

/// Global system configuration state, using atomic pointer
static SYSTEM_CONFIG: LazyLock<AtomicPtr<SystemConfig>> = LazyLock::new(|| {
    let config = Box::new(SystemConfig::default());
    AtomicPtr::new(Box::into_raw(config))
});

/// System configuration manager
pub struct SystemConfigManager;

impl SystemConfigManager {
    /// Get Arc reference to current global system configuration
    pub fn get_config() -> Arc<SystemConfig> {
        let ptr = SYSTEM_CONFIG.load(Ordering::Acquire);
        // Safe: we never free this pointer, only replace it
        let config = unsafe { &*ptr };
        Arc::new(config.clone())
    }

    /// Update global system configuration
    pub fn update_config(new_config: SystemConfig) {
        let new_ptr = Box::into_raw(Box::new(new_config));
        let old_ptr = SYSTEM_CONFIG.swap(new_ptr, Ordering::AcqRel);

        // Free the old configuration (but not at program exit to avoid memory leaks)
        if !old_ptr.is_null() {
            unsafe {
                let _old_box = Box::from_raw(old_ptr);
                // Box will automatically free the memory
            }
        }
    }

    /// Load system configuration from database and initialize (should be called at system startup)
    pub fn initialize_from_db(config: SystemConfig) {
        tracing::debug!("Initializing system config from DB: {:?}", config);
        Self::update_config(config);
        tracing::debug!("System config initialized successfully");
    }

    /// Get current system timezone
    pub fn get_timezone() -> Tz {
        let ptr = SYSTEM_CONFIG.load(Ordering::Acquire);
        unsafe { (*ptr).timezone }
    }

    /// Get current system localization settings
    pub fn get_localization() -> Localization {
        let ptr = SYSTEM_CONFIG.load(Ordering::Acquire);
        unsafe { (*ptr).localization.clone() }
    }
}
