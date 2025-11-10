mod event_handler;

use std::{collections::HashMap, sync::Arc};

use engine_core::{EngineBaseContext, context_trait::EngineContextTrait, state_machine::EngineRunState};
use star_river_core::{custom_type::StrategyId, engine::EngineName};
use tokio::sync::Mutex;

use crate::{
    indicator_engine_type::IndicatorSubKey,
    state_machine::{IndicatorEngineAction, IndicatorEngineStateMachine, indicator_engine_transition},
};

#[derive(Debug, Clone)]
pub struct IndicatorEngineContext {
    pub base_context: EngineBaseContext<IndicatorEngineAction>,
    pub subscribe_indicators: Arc<Mutex<HashMap<IndicatorSubKey, Vec<StrategyId>>>>, // 已订阅的指标
}

impl IndicatorEngineContext {
    pub fn new() -> Self {
        let state_machine = IndicatorEngineStateMachine::new(
            EngineName::IndicatorEngine.to_string(),
            EngineRunState::Created,
            indicator_engine_transition,
        );
        let base_context = EngineBaseContext::new(EngineName::IndicatorEngine, state_machine);
        Self {
            base_context,
            subscribe_indicators: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl EngineContextTrait for IndicatorEngineContext {
    type Action = IndicatorEngineAction;
    fn base_context(&self) -> &EngineBaseContext<IndicatorEngineAction> {
        &self.base_context
    }

    fn base_context_mut(&mut self) -> &mut EngineBaseContext<IndicatorEngineAction> {
        &mut self.base_context
    }
}
