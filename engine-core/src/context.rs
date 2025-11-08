use std::fmt::Debug;
use tokio_util::sync::CancellationToken;

use star_river_core::engine::EngineName;
use crate::state_machine::EngineAction;
use super::state_machine::EngineStateMachine;
use std::sync::Arc;
use tokio::sync::RwLock;




#[derive(Debug, Clone)]
pub struct EngineBaseContext<Action>
where
    Action: EngineAction,
{
    engine_name: EngineName,
    cancel_token: CancellationToken,
    state_machine: Arc<RwLock<EngineStateMachine<Action>>>,
}


impl<Action> EngineBaseContext<Action>
where
    Action: EngineAction,
{
    pub fn new(
        engine_name: EngineName,  
        state_machine: EngineStateMachine<Action>
    ) -> Self {
        Self { 
            engine_name, 
            cancel_token: CancellationToken::new(), 
            state_machine: Arc::new(RwLock::new(state_machine)) }
    }

    pub fn engine_name(&self) -> &EngineName {
        &self.engine_name
    }

    pub fn cancel_token(&self) -> &CancellationToken {
        &self.cancel_token
    }

    pub fn state_machine(&self) -> Arc<RwLock<EngineStateMachine<Action>>> {
        Arc::clone(&self.state_machine)
    }
}





