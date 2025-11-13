// External crate imports
use async_trait::async_trait;
// Workspace crate imports
use engine_core::{
    EngineContextAccessor, EngineEventListener, EngineLifecycle,
    context_trait::{EngineContextTrait, EngineStateMachineTrait},
    state_machine::EngineStateTransTrigger,
};

// Current crate imports
use crate::{BacktestEngine, engine_error::BacktestEngineError, engine_state_machine::BacktestEngineAction};

#[async_trait]
impl EngineLifecycle for BacktestEngine {
    type Error = BacktestEngineError;
    async fn start(&self) -> Result<(), Self::Error> {
        let engine_name = self.with_ctx_read(|ctx| ctx.engine_name().to_string()).await;
        tracing::info!("=================start engine [{engine_name}]====================");
        tracing::info!("[{engine_name}] start to start");

        // Start transition: created -> Launching
        self.update_engine_state(EngineStateTransTrigger::Start).await?;

        // Complete startup: Launching -> Running
        self.update_engine_state(EngineStateTransTrigger::StartComplete).await?;

        Ok(())
    }

    async fn stop(&self) -> Result<(), Self::Error> {
        let engine_name = self.with_ctx_read(|ctx| ctx.engine_name().to_string()).await;
        tracing::info!("=================stop engine [{engine_name}]====================");
        tracing::info!("[{engine_name}] start to stop");

        // Stop transition: running -> Stopping
        self.update_engine_state(EngineStateTransTrigger::Stop).await?;

        // Complete stop: Stopping -> Stopped
        self.update_engine_state(EngineStateTransTrigger::StopComplete).await?;

        tracing::info!("[{engine_name}] stop complete");
        Ok(())
    }

    async fn update_engine_state(&self, trans_trigger: EngineStateTransTrigger) -> Result<(), Self::Error> {
        let (engine_name, state_machine) = self
            .with_ctx_read(|ctx| {
                let engine_name = ctx.engine_name().to_string();
                let state_machine = ctx.state_machine().clone();
                (engine_name, state_machine)
            })
            .await;

        let transition_result = {
            let mut state_machine = state_machine.write().await;
            state_machine.transition(trans_trigger)?
        };

        for action in transition_result.actions() {
            let (previous_state, current_state) = {
                let state_machine = state_machine.read().await;
                (state_machine.previous_state().clone(), state_machine.current_state().clone())
            };

            match action {
                BacktestEngineAction::LogTransition => {
                    tracing::debug!("[{engine_name}] state transition: {:?} -> {:?}", previous_state, current_state);
                }

                BacktestEngineAction::ListenAndHandleEvents => {
                    tracing::info!("[{engine_name}] starting to listen events");
                    self.listen_events().await;
                }

                BacktestEngineAction::ListenAndHandleCommands => {
                    tracing::info!("[{engine_name}] starting to listen commands");
                    self.listen_commands().await;
                }

                BacktestEngineAction::LogEngineState => {
                    tracing::info!("[{engine_name}] current state: {:?}", current_state);
                }

                BacktestEngineAction::LogError(error) => {
                    tracing::error!("[{engine_name}] error: {:?}", error);
                }
            }
        }

        Ok(())
    }
}
