use async_trait::async_trait;
use event_center::communication::engine::EngineCommand;
use event_center::event::Event;
use super::ExchangeEngineContext;
use engine_core::context_trait::{EngineContextTrait, EngineEventHandler};
use event_center::communication::engine::exchange_engine::{ExchangeEngineCommand, RegisterExchangeRespPayload, RegisterExchangeResponse};
use std::sync::Arc;
use event_center::communication::Command;


#[async_trait]
impl EngineEventHandler for ExchangeEngineContext {

    async fn handle_event(&mut self, event: Event) {
        tracing::info!("[{}] received event: {:?}", self.engine_name(), event);
    }
    
    async fn handle_command(&mut self, command: EngineCommand) {
        match command {
            EngineCommand::ExchangeEngine(exchange_engine_command) => {
                match exchange_engine_command {
                    ExchangeEngineCommand::RegisterExchange(cmd) => {
                        let result = self.register_exchange(cmd.account_id).await;

                        let response = if let Ok(()) = result {
                            // success
                            let payload = RegisterExchangeRespPayload::new(cmd.account_id, cmd.exchange.clone());
                            RegisterExchangeResponse::success(Some(payload))
                        } else {
                            // 注册失败
                            let error = result.unwrap_err();
                            RegisterExchangeResponse::error(Arc::new(error))
                        };
                        // 发送响应事件
                        cmd.respond(response);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

}
