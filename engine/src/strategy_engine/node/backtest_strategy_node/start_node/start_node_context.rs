use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use std::any::Any;
use event_center::{command::cache_engine_command::{GetCacheLengthMultiParams, GetCacheLengthParams, CacheEngineCommand}, Event};
use tracing::instrument;
use types::strategy::{node_command::{GetStrategyCacheKeysParams}, node_event::{NodeEvent}};
use async_trait::async_trait;
use types::strategy::BacktestStrategyConfig;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use std::sync::Arc;
use tokio::sync::RwLock;
use heartbeat::Heartbeat;
use tokio::sync::Mutex;
use utils::get_utc8_timestamp_millis;
use types::strategy::node_event::{KlineTickEvent, KlinePlayFinishedEvent, SignalEvent};
use types::strategy::strategy_inner_event::StrategyInnerEvent;

#[derive(Debug, Clone)]
pub struct StartNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub backtest_config: Arc<RwLock<BacktestStrategyConfig>>,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub played_index: Arc<RwLock<u32>>,
    
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
    async fn handle_node_event(&mut self, message: NodeEvent) -> Result<(), String> {
        tracing::info!("{}: 收到消息: {:?}", self.base_context.node_id, message);
        Ok(())
    }

    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent) -> Result<(), String> {
        match strategy_inner_event {
            StrategyInnerEvent::PlayIndexUpdate(play_index_update_event) => {
                // 更新播放索引
                *self.played_index.write().await = play_index_update_event.played_index;
                tracing::debug!("{}: 更新播放索引: {}", self.get_node_id(), play_index_update_event.played_index);
                if play_index_update_event.played_index == play_index_update_event.total_signal_count {
                    // 发送k线播放完毕信号
                    self.send_finish_signal(play_index_update_event.played_index).await;
                }
                else {
                    // 发送k线跳动信号
                    self.send_kline_tick_signal(play_index_update_event.played_index).await;
                }
            }
        }
        Ok(())
    }
    
}

impl StartNodeContext {
    // 发送k线跳动信号
    pub async fn send_kline_tick_signal(&self, signal_index : u32) {
        let kline_tick_event = KlineTickEvent {
            from_node_id: self.base_context.node_id.clone(),
            from_node_name: self.base_context.node_name.clone(),
            from_node_handle_id: self.base_context.output_handle.get(&format!("start_node_output")).unwrap().output_handle_id.clone(),
            signal_index,
            message_timestamp: chrono::Utc::now().timestamp_millis(),
        };
        
        let signal = NodeEvent::Signal(SignalEvent::KlineTick(kline_tick_event.clone()));
        self.get_default_output_handle().send(signal).unwrap();

    }

    // 发送k线播放完毕信号
    pub async fn send_finish_signal(&self, signal_index : u32) {
        let finish_signal = KlinePlayFinishedEvent {
            from_node_id: self.get_node_id().clone(),
            from_node_name: self.get_node_name().clone(),
            from_node_handle_id: self.get_default_output_handle().output_handle_id.clone(),
            signal_index,
            message_timestamp: get_utc8_timestamp_millis(),
        };

        let signal = NodeEvent::Signal(SignalEvent::KlinePlayFinished(finish_signal));
        // tracing::info!("{}: 发送信号: {:?}", node_id, signal);
        self.get_default_output_handle().send(signal).unwrap();

    }
}
