use std::sync::Arc;

use async_trait::async_trait;
use engine_core::context_trait::{EngineContextTrait, EngineEventHandler};
use event_center::{EngineCommand, Event};
use star_river_event::communication::market_engine::{
    GetKlineHistoryRespPayload,
    GetKlineHistoryResponse,
    GetSymbolInfoRespPayload,
    GetSymbolInfoResponse,
    MarketEngineCommand,
    // SubscribeKlineStreamRespPayload, SubscribeKlineStreamResponse, UnsubscribeKlineStreamRespPayload, UnsubscribeKlineStreamResponse,
};

use super::MarketEngineContext;

#[async_trait]
impl EngineEventHandler for MarketEngineContext {
    async fn handle_event(&mut self, event: Event) {
        tracing::info!("[{}] received event: {:?}", self.engine_name(), event);
    }

    async fn handle_command(&mut self, command: EngineCommand) {
        match command {
            // EngineCommand::MarketEngine(MarketEngineCommand::SubscribeKlineStream(cmd)) => {
            //     self.subscribe_kline_stream(
            //         cmd.strategy_id,
            //         cmd.account_id,
            //         cmd.exchange.clone(),
            //         cmd.symbol.clone(),
            //         cmd.interval.clone(),
            //         cmd.cache_size,
            //         cmd.frequency,
            //     )
            //     .await
            //     .unwrap();
            //     tracing::debug!("市场数据引擎订阅K线流成功, 请求节点: {}", cmd.node_id);

            //     let payload = SubscribeKlineStreamRespPayload::new(cmd.exchange.clone(), cmd.symbol.clone(), cmd.interval.clone());
            //     let response = SubscribeKlineStreamResponse::success(Some(payload));
            //     cmd.respond(response);
            // }

            // EngineCommand::MarketEngine(MarketEngineCommand::UnsubscribeKlineStream(cmd)) => {
            //     self.unsubscribe_kline_stream(
            //         cmd.strategy_id,
            //         cmd.account_id,
            //         cmd.exchange.clone(),
            //         cmd.symbol.clone(),
            //         cmd.interval.clone(),
            //         cmd.frequency,
            //     )
            //     .await
            //     .unwrap();
            //     let payload = UnsubscribeKlineStreamRespPayload::new(cmd.exchange.clone(), cmd.symbol.clone(), cmd.interval.clone());
            //     let response = UnsubscribeKlineStreamResponse::success(Some(payload));
            //     cmd.respond(response);
            // }
            EngineCommand::MarketEngine(MarketEngineCommand::GetKlineHistory(cmd)) => {
                let kline_history = self
                    .get_kline_history(
                        cmd.account_id,
                        cmd.exchange.clone(),
                        cmd.symbol.clone(),
                        cmd.interval.clone(),
                        cmd.time_range.clone(),
                    )
                    .await;
                match kline_history {
                    Ok(kline_history) => {
                        let payload =
                            GetKlineHistoryRespPayload::new(cmd.exchange.clone(), cmd.symbol.clone(), cmd.interval.clone(), kline_history);
                        let resp = GetKlineHistoryResponse::success(payload);
                        cmd.respond(resp);
                    }
                    Err(e) => {
                        let resp = GetKlineHistoryResponse::fail(Arc::new(e));
                        cmd.respond(resp);
                    }
                }
            }
            EngineCommand::MarketEngine(MarketEngineCommand::GetSymbolInfo(cmd)) => {
                let result = self.get_symbol(cmd.account_id, cmd.symbol.clone()).await;
                match result {
                    Ok(symbol) => {
                        let payload = GetSymbolInfoRespPayload::new(symbol);
                        let resp = GetSymbolInfoResponse::success(payload);
                        cmd.respond(resp);
                    }
                    Err(e) => {
                        let resp = GetSymbolInfoResponse::fail(Arc::new(e));
                        cmd.respond(resp);
                    }
                }
            }
            _ => {}
        }
    }
}
