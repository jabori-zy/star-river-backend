
use async_trait::async_trait;
use super::Binance;
use exchange_core::{ExchangeLifecycle, MetadataAccessor};
use exchange_core::state_machine::ExchangeStateTransTrigger;
use crate::binance::{BinanceAction,BinanceMetadata};
use crate::binance::binance_ws_client::BinanceWsBuilder;
use super::error::BinanceError;


#[async_trait]
impl ExchangeLifecycle for Binance {
    type Error = BinanceError;


    async fn initialize(&self) -> Result<(), BinanceError> {
        let account_name = self.with_metadata_read(|metadata| {
            metadata.account_name().clone()
        }).await;
        tracing::info!("=================initialize binance exchange [{account_name}]====================");
        tracing::info!("[{account_name}] start to initialize");
        // 开始启动 created -> Start
        self.update_state(ExchangeStateTransTrigger::StartInit).await?;
        // 切换为running状态
        self.update_state(ExchangeStateTransTrigger::FinishInit).await?;
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), BinanceError> {
        Ok(())
    }

    async fn update_state(&self, trans_trigger: ExchangeStateTransTrigger) -> Result<(), BinanceError> {
        let account_name = self.with_metadata_read(|metadata| {
            metadata.account_name().clone()
        }).await;

        let state_machine = self.state_machine();
        
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
                BinanceAction::LogTransition => {
                    tracing::debug!("[{account_name}] state transition: {:?} -> {:?}", current_state, transition_result.new_state());
                }

                BinanceAction::InitHttpClient => {
                    tracing::info!("[{account_name}] starting to initialize http client");
                    self.http_client().ping().await?;
                    tracing::info!("[{account_name}] http client initialized successfully");
                    
                }

                BinanceAction::InitWsClient => {
                    tracing::info!("[{account_name}] starting to initialize websocket client");
                    let (websocket_state, _) = BinanceWsBuilder::connect_default().await.unwrap();
                    self.set_ws_client(websocket_state).await;
                    tracing::info!("[{account_name}] websocket client initialized successfully");
                }

                BinanceAction::LogExchangeState => {
                    tracing::info!("[{account_name}] current state: {:?}", current_state);
                }

                BinanceAction::LogError(error) => {
                    tracing::error!("[{account_name}] error: {:?}", error);
                }
            }
        }
        Ok(())
    }
}
