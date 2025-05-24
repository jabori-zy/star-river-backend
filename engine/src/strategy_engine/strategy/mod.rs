
pub mod strategy_state_machine;
pub mod strategy_functions;
pub mod strategy_context;
pub mod live_strategy;
pub mod backtest_strategy;


use std::sync::Arc;
use tokio::sync::RwLock;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use strategy_context::StrategyContext;
use strategy_state_machine::LiveStrategyStateTransitionEvent;
use strategy_functions::StrategyFunction;
use strategy_state_machine::LiveStrategyStateMachineTrait;
use types::cache::CacheKey;

#[async_trait]
pub trait LiveStrategyTrait: Debug + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_box(&self) -> Box<dyn LiveStrategyTrait>;
    fn get_context(&self) -> Arc<RwLock<Box<dyn StrategyContext>>>;
    async fn get_strategy_id(&self) -> i32;
    async fn get_strategy_name(&self) -> String;
    async fn get_state_machine(&self) -> Box<dyn LiveStrategyStateMachineTrait> {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_state_machine()
    }
    async fn set_state_machine(&mut self, state_machine: Box<dyn LiveStrategyStateMachineTrait>) {
        let context = self.get_context();
        let mut context_guard = context.write().await;
        context_guard.set_state_machine(state_machine.clone_box());
    }
    async fn update_strategy_state(&mut self, event: LiveStrategyStateTransitionEvent) -> Result<(), String>;
    async fn listen_node_message(&self) -> Result<(), String> {
        let context = self.get_context();
        StrategyFunction::listen_node_message(context).await;
        Ok(())
    }
    async fn listen_event(&self) -> Result<(), String> {
        let context = self.get_context();
        StrategyFunction::listen_event(context).await;
        Ok(())
    }
    
    async fn listen_command(&self) -> Result<(), String> {
        let context = self.get_context();
        StrategyFunction::listen_command(context).await;
        Ok(())
    }

    async fn get_strategy_cache_keys(&self) -> Vec<CacheKey> {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_cache_keys().await
    }
    async fn enable_strategy_data_push(&mut self) -> Result<(), String>;
    async fn disable_strategy_data_push(&mut self) -> Result<(), String>;
    
    async fn init_strategy(&mut self) -> Result<(), String>;
    async fn start_strategy(&mut self) -> Result<(), String>;
    async fn stop_strategy(&mut self) -> Result<(), String>;
}


impl Clone for Box<dyn LiveStrategyTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}



    




