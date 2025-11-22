// ============================================================================
// 模块声明
// ============================================================================
pub mod context;
pub mod context_trait;
pub mod engine;
pub mod engine_trait;
pub mod state_machine;
pub mod state_machine_error;

// ============================================================================
// 重导出
// ============================================================================
use std::{collections::HashMap, sync::LazyLock};

pub use context::EngineBaseContext;
pub use engine::EngineBase;
pub use engine_trait::{EngineContextAccessor, EngineEventListener, EngineLifecycle};
use event_center::event::Channel;
use star_river_core::engine::EngineName;

// 引擎事件接收器, 定义每个引擎应该接收哪些引擎的事件
static ENGINE_EVENT_RECEIVERS: LazyLock<HashMap<EngineName, Vec<Channel>>> = LazyLock::new(|| {
    HashMap::from([
        // (EngineName::CacheEngine, vec![Channel::Exchange]),
        (EngineName::ExchangeEngine, vec![]),
        (EngineName::MarketEngine, vec![]),
        (EngineName::IndicatorEngine, vec![Channel::Exchange]),
        (EngineName::BacktestEngine, vec![Channel::Market]),
        // (EngineName::AccountEngine, vec![Channel::Account]),
    ])
});

pub struct EngineEventReceiver;

impl EngineEventReceiver {
    pub fn get_event_receivers(engine_name: &EngineName) -> Vec<Channel> {
        ENGINE_EVENT_RECEIVERS.get(engine_name).cloned().unwrap_or_default()
    }
}
