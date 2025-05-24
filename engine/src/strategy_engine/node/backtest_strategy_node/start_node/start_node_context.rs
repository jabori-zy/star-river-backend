use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use std::any::Any;
use event_center::{command::cache_engine_command::{GetCacheLengthMultiParams, GetCacheLengthParams, CacheEngineCommand}, Event};
use tracing::instrument;
use types::strategy::{node_command::{GetStrategyCacheKeysParams, StrategyCommand}, node_message::{NodeMessage, SignalMessage, SignalType}};
use async_trait::async_trait;
use types::strategy::BacktestStrategyConfig;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use std::sync::Arc;
use tokio::sync::RwLock;
use heartbeat::Heartbeat;
use tokio::sync::Mutex;
use tokio::sync::oneshot;
use types::cache::CacheKey;
use utils::get_utc8_timestamp_millis;
use std::collections::HashMap;
use event_center::response::cache_engine_response::CacheEngineResponse;
use types::strategy::node_response::{NodeResponse, StrategyResponse};

#[derive(Debug, Clone)]
pub struct StartNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub backtest_config: Arc<RwLock<BacktestStrategyConfig>>,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub strategy_cache_keys: Vec<CacheKey>,
    pub cache_lengths: HashMap<CacheKey, u32>,
    
}


#[async_trait]
impl BacktestNodeContextTrait for StartNodeContext {

    fn clone_box(&self) -> Box<dyn BacktestNodeContextTrait> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn get_base_context(&self) -> &BacktestBaseNodeContext {
        &self.base_context
    }
    
    fn get_base_context_mut(&mut self) -> &mut BacktestBaseNodeContext {
        &mut self.base_context
    }

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        self.base_context.output_handle.get(&format!("start_node_output")).unwrap().clone()
    }


    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        tracing::info!("{}: 收到事件: {:?}", self.base_context.node_id, event);
        Ok(())
    }
    async fn handle_message(&mut self, message: NodeMessage) -> Result<(), String> {
        tracing::info!("{}: 收到消息: {:?}", self.base_context.node_id, message);
        Ok(())
    }
    
}

impl StartNodeContext {
    pub async fn send_fetch_kline_data_signal(&self, signal_count : u32) {
        let fetch_kline_message = SignalMessage {
            from_node_id: self.base_context.node_id.clone(),
            from_node_name: self.base_context.node_name.clone(),
            from_node_handle_id: self.base_context.output_handle.get(&format!("start_node_output")).unwrap().output_handle_id.clone(),
            signal_type: SignalType::FetchKlineData(signal_count),
            message_timestamp: chrono::Utc::now().timestamp_millis(),
        };
        
        let signal = NodeMessage::Signal(fetch_kline_message.clone());
        self.get_default_output_handle().send(signal).unwrap();

    }

    pub async fn send_finish_signal(&self) {
        let finish_signal = SignalMessage {
            from_node_id: self.base_context.node_id.clone(),
            from_node_name: self.base_context.node_name.clone(),
            from_node_handle_id: self.base_context.output_handle.get(&format!("start_node_output")).unwrap().output_handle_id.clone(),
            signal_type: SignalType::KlinePlayFinished,
            message_timestamp: chrono::Utc::now().timestamp_millis(),
        };

        let signal = NodeMessage::Signal(finish_signal.clone());
        // tracing::info!("{}: 发送信号: {:?}", node_id, signal);
        self.get_default_output_handle().send(signal).unwrap();

    }
}
