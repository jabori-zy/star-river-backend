use async_trait::async_trait;
use engine_core::{
    EngineContextAccessor, EngineLifecycle, EngineEventListener,
    context_trait::{EngineContextTrait, EngineStateMachineTrait}
};

use crate::{context::ExchangeEngineContext, state_machine::ExchangeEngineAction};
use crate::ExchangeEngine;
use engine_core::state_machine::EngineStateTransTrigger;
use crate::error::ExchangeEngineError;

#[async_trait]
impl EngineLifecycle for ExchangeEngine {
    type Error = ExchangeEngineError;

    async fn start(&self) -> Result<(), Self::Error> {
        let engine_name = self.with_ctx_read(|ctx| {
            ctx.engine_name().to_string()
        }).await;
        tracing::info!("=================start engine [{engine_name}]====================");
        tracing::info!("[{engine_name}] start to start");
        // 开始启动 created -> Start
        self.update_engine_state(EngineStateTransTrigger::Start).await?;
        // 切换为running状态
        self.update_engine_state(EngineStateTransTrigger::StartComplete).await?;
        Ok(())
    }

    async fn stop(&self) -> Result<(), Self::Error> {
        let engine_name = self.with_ctx_read(|ctx| {
            ctx.engine_name().to_string()
        }).await;
        tracing::info!("=================stop engine [{engine_name}]====================");
        tracing::info!("[{engine_name}] start to stop");
        // 开始停止 created -> Stop
        self.update_engine_state(EngineStateTransTrigger::Stop).await?;
        // 切换为stopped状态
        self.update_engine_state(EngineStateTransTrigger::StopComplete).await?;
        tracing::info!("[{engine_name}] stop complete");
        Ok(())
    }

    async fn update_engine_state(&self, trans_trigger: EngineStateTransTrigger) -> Result<(), Self::Error> {
        let (engine_name, state_machine) = self.with_ctx_read(|ctx| {
            let engine_name = ctx.engine_name().to_string();
            let state_machine = ctx.state_machine().clone();
            (engine_name, state_machine)
        }).await;
        
        let transition_result = {
            let mut state_machine = state_machine.write().await;
            state_machine.transition(trans_trigger)?
        };
        for action in transition_result.actions() {

            let current_state = {
                let state_machine = state_machine.read().await;
                state_machine.current_state().clone()
            };
            
            
            match action {
                ExchangeEngineAction::LogTransition => {
                    tracing::debug!("[{engine_name}] state transition: {:?} -> {:?}", current_state, transition_result.new_state());
                }

                ExchangeEngineAction::ListenAndHandleEvents => {
                    tracing::info!("[{engine_name}] starting to listen events");
                    self.listen_events().await;
                }
                ExchangeEngineAction::ListenAndHandleCommands => {
                    tracing::info!("[{engine_name}] starting to listen commands");
                    self.listen_commands().await;
                }
                ExchangeEngineAction::LogEngineState => {
                    tracing::info!("[{engine_name}] current state: {:?}", current_state);
                }
                ExchangeEngineAction::LogError(error) => {
                    tracing::error!("[{engine_name}] error: {:?}", error);
                }
                _ => {}
            }
        }
        Ok(())
    }
}